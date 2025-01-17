use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use anyhow::anyhow;
use log::{error, info};
use sea_orm::*;
use tap::TapFallible;
use tokio::sync::RwLock;

use hap::accessory::{AccessoryInformation, HapAccessory};
use hap::service::HapService;
use hl_integration::convertor::ext_factory::get_unit_convertor_factory;
use hl_integration::convertor::UnitConvertor;
use target_hap::delegate::model::{AccessoryModelContext, HapAccessoryDelegateModel};
use hl_integration::platform::hap::hap_device::{AsHapDevice, HapDevice};
use target_hap::hap_manager::HapManage;
use target_hap::hap_type_wrapper::HapTypeWrapper;
use target_hap::iot::iot_hap_accessory::IotHapAccessory;
use target_hap::types::CharIdentifier;

use crate::db::entity::prelude::{HapAccessoryModel, HapCharacteristicEntity, HapServiceColumn, HapServiceEntity};
use crate::init::{DevicePointer, HapAccessoryPointer};
use crate::init::hap_init::InitServiceContext;

/// 初始化配件的设备
/// 需要建立配件与设备的关系,处理设备情况
/// 像设备注册监听属性,销毁配件时候需要移除监听
pub(crate) async fn init_hap_accessory<'a, C: ConnectionTrait>(conn: &C, hap_manage: HapManage,
                                                               device: DevicePointer, hap_accessory: HapAccessoryModel) -> anyhow::Result<HapAccessoryPointer> {
    let aid = hap_accessory.aid as u64;
    let mut hss: Vec<Box<dyn HapService>> = vec![];
    let dev_c = device.clone();
    let hap_device = dev_c
        .as_hap_device()
        .ok_or(anyhow!("设备不支持hap"))?;

    // 初始化配件服务
    let dev_info = hap_device.get_hap_info();
    let name = hap_accessory.name.clone();

    // 可以从设备信息中获取
    let info = AccessoryInformation {
        name: name.clone(),
        model: dev_info.model,
        firmware_revision: dev_info.firmware_revision,
        software_revision: dev_info.software_revision,
        serial_number: dev_info.serial_number,
        manufacturer: dev_info.manufacturer,
        ..Default::default()
    };

    let mut service = info.to_service(1, aid)?;


    let ids: Vec<u64> = service.get_characteristics()
        .into_iter()
        .map(|e| e.get_id())
        .collect();
    let mut cid = ids.last().unwrap_or(&0) + 1;
    hss.push(Box::new(service));
    // 初始化子服务
    let services = HapServiceEntity::find()
        .filter(HapServiceColumn::AccessoryId.eq(hap_accessory.aid)
            .and(HapServiceColumn::Disabled.eq(false)))
        .find_with_related(HapCharacteristicEntity)
        .all(conn)
        .await?;
    if services.is_empty() {
        return Err(anyhow!("配件:{},无服务",name));
    };
    let mut convertor_map = HashMap::new();

    // 处理chars 上的类型转换器
    for (svc, chars) in services.iter() {
        for char in chars.iter() {
            if let Some(conv) = char.convertor.clone() {
                let conv = get_unit_convertor_factory()
                    .get_convertor(conv.as_str(), char.convertor_param.clone())
                    .tap_err(|_| error!("单位转换器:{conv}不存在"))?;
                let t = HapTypeWrapper::from_str(&char.characteristic_type)?;
                let cid = CharIdentifier::new(svc.tag.clone().unwrap_or("default".to_string()), t);
                convertor_map.insert(cid, UnitConvertor::new(conv));
            };
        }
    }
    info!("aid:{aid}:convertor_map:{:?}",convertor_map.keys());


    // 初始化配件 委托 model
    let model = init_hap_delegate(aid, hap_accessory.clone(), &device, hap_manage.clone(), convertor_map).await?;
    // 初始化属性映射
    let accessory = Arc::new(RwLock::new(Box::new(IotHapAccessory::new(aid, hss, model)) as Box<dyn HapAccessory>));


    // 属性映射注册器,读取cid:1-> 读设备的xx
    // prop_mapping_register.push(cid,params,conv)
    // let prop_mapping_register: Mutex<Vec<PropMappingParam>> = Mutex::new(vec![]);


    // 设置读写值,监听器
    for svc_chs in services.into_iter() {
        let ctx = InitServiceContext {
            aid,
            sid: cid,
            stag: svc_chs.0.tag.clone(),
            device: device.clone(),
            accessory: accessory.clone(),
            hap_manage: hap_manage.clone(),
        };
        //检测是否存在未填的特征
        let required_characteristics = hap_manage.hap_mata.services
            .get(svc_chs.0.service_type.as_str())
            .ok_or(anyhow!("服务类型:{}不存在",svc_chs.0.service_type))?
            .characteristics
            .required_characteristics
            .clone();
        let mut required_types = vec![];
        for char_name in required_characteristics.into_iter() {
            let ch = hap_manage.hap_mata.characteristics.get(char_name.as_str());
            if let Some(c)= ch {
                let char_type = hap_metadata::utils::pascal_case(c.name.as_str());
                required_types.push(char_type);
            }
        }


        for char_type in required_types {
            if !svc_chs.1.iter().any(|ch| ch.characteristic_type == char_type) {
                return Err(anyhow!("配件:{},服务:{}未找到必填特征:{}",name,svc_chs.0.service_type,char_type));
            }
        }


        // let req = svc_meta.characteristics.required_characteristics;


        let len = crate::init::hap_init::add_service(ctx, svc_chs).await?;
        cid += len as u64 + 1;
        // 转成服务, 服务需要服务类型和服务的必填特征
    }
    //检测特征id 是否重复
    check_ids(name, &accessory).await?;

    Ok(accessory.clone())
}


async fn init_hap_delegate(aid: u64, hap_accessory: HapAccessoryModel, device: &DevicePointer, hap_manager: HapManage, convertor_map: HashMap<CharIdentifier, UnitConvertor>) -> anyhow::Result<Option<HapAccessoryDelegateModel>> {
    if hap_accessory.hap_model_delegates.0.is_empty() {
        return Ok(None);
    };

    Ok({
        let ctx = AccessoryModelContext {
            aid,
            dev: device.clone(),
            hap_manager,
            resource_table: Default::default(),
            convertor_map,
        };
        // 事件订阅
        let model = HapAccessoryDelegateModel::new(ctx, hap_accessory.hap_model_delegates.0).await?;
        model.init().await?;


        if model.model_ext.is_subscribe_event() {
            let model_c = model.clone();
            device.add_listener(Box::new(move |data| {
                let model_c = model_c.clone();
                Box::pin(async move {
                    model_c.on_event(data).await;
                })
            })).await;
        };
        Some(model)
    }
    )
}

async fn check_ids(name_c: String, accessory: &Arc<RwLock<Box<dyn HapAccessory>>>) -> anyhow::Result<()> {
    let mut ids = vec![];
    for ch in accessory.read().await.get_services() {
        let sid = ch.get_id();
        if ids.contains(&sid) {
            return Err(anyhow!("配件:{},服务id:{}重复",name_c,sid));
        }
        ids.push(sid);
        for ch in ch.get_characteristics() {
            let id = ch.get_id();
            if ids.contains(&id) {
                return Err(anyhow!("配件:{},特征id:{}重复",name_c,id));
            }
            ids.push(id);
        }
    }
    Ok(())
}