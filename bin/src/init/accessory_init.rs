use std::sync::Arc;
use anyhow::anyhow;
use log::info;
use sea_orm::*;
use tokio::sync::{Mutex, RwLock};
use hap::accessory::{AccessoryInformation, HapAccessory};
use hap::service::HapService;
use crate::db::entity::prelude::{HapAccessoryModel, HapCharacteristicEntity, HapServiceColumn, HapServiceEntity};
use crate::db::SNOWFLAKE;
use crate::hap::iot::iot_hap_accessory::IotHapAccessory;
use crate::hap::models::{AccessoryModel, AccessoryModelContext};
use crate::init::{DevicePointer, HapAccessoryPointer};
use crate::init::hap_init::InitServiceContext;
use crate::init::manager::hap_manager::HapManage;
#[cfg(feature = "deno")]
use crate::js_engine::init_hap_accessory_module::init_hap_accessory_module;

/// 初始化配件的设备
/// 需要建立配件与设备的关系,处理设备情况
/// 像设备注册监听属性,销毁配件时候需要移除监听
pub(crate) async fn init_hap_accessory<'a, C: ConnectionTrait>(conn: &C, hap_manage: HapManage,
                                                               device: DevicePointer, hap_accessory: HapAccessoryModel) -> anyhow::Result<HapAccessoryPointer> {
    let aid = hap_accessory.aid as u64;
    let mut hss: Vec<Box<dyn HapService>> = vec![];
    // let device = device.value().read().await.device.clone();
    let device = device.clone();
    // 初始化配件服务

    let dev_info = device.get_info().clone();
    let name = hap_accessory.name.unwrap_or(dev_info.name.clone());
    let name_c = name.clone();
    let software_revision = dev_info
        .extra
        .and_then(|i| i.fw_version);
    let parts: Vec<&str> = dev_info.model.split('.').collect();
    let manufacturer = parts.first()
        .map(|f| f.to_string())
        .unwrap_or("未知制造商".to_string());
    // 可以从设备信息中获取
    let info = AccessoryInformation {
        name,
        model: dev_info.model.clone(),
        firmware_revision: dev_info.firmware_revision.clone(),
        software_revision,
        serial_number: dev_info.did,
        manufacturer,
        // configured_name: Some(dev_info.model.clone()),
        ..Default::default()
    };
    // SwitchAccessory::new(1, info.clone())?;
    let mut service = info.to_service(1, aid)?;
    let ids: Vec<u64> = service.get_characteristics()
        .into_iter()
        .map(|e| e.get_id())
        .collect();
    info!("特征id:{:?}", ids);
    let mut cid = ids.last().unwrap_or(&0) + 1;
    // cid += service.get_characteristics().len() as u64 + 1;
    hss.push(Box::new(service));
    // 初始化子服务
    let services = HapServiceEntity::find()
        .filter(HapServiceColumn::AccessoryId.eq(hap_accessory.aid).and(HapServiceColumn::Disabled.eq(false)))
        .find_with_related(HapCharacteristicEntity)
        .all(conn)
        .await?;
    if services.is_empty() {
        return Err(anyhow!("配件:{},无服务",name_c));
    };

    // 初始化model
    let model = if let Some(model) = hap_accessory.model {
        let ctx = AccessoryModelContext {
            aid,
            dev: device.device.clone(),
            hap_manager: hap_manage.clone(),
        };
        Some(AccessoryModel::new(ctx, model.as_str())?)
    } else {
        None
    };

    let accessory = Arc::new(RwLock::new(Box::new(IotHapAccessory::new(aid, hss, model)) as Box<dyn HapAccessory>));
    let ch_id = SNOWFLAKE.next_id();
    // 注册到manage 上
    hap_manage.put_accessory_ch(aid, ch_id, false).await;

    for service in services.into_iter() {
        let ctx = InitServiceContext {
            aid,
            sid: cid,
            stag: service.0.tag.clone(),
            device: device.clone(),
            accessory: accessory.clone(),
            hap_manage: hap_manage.clone(),
        };
        let len = crate::init::hap_init::add_service(ctx, service).await?;
        cid += len as u64 + 1;
        // 转成服务, 服务需要服务类型和服务的必填特征
    }
    //检测特征id 是否重复
    check_ids(name_c, &accessory).await?;


    // 查询script
    #[cfg(feature = "deno")]
    if let Some(script) = hap_accessory.script {
        if !script.is_empty() {
            // 初始化hap js模块,
            init_hap_accessory_module(hap_manage, ch_id, aid, script.as_str()).await?;
        };
    };


    Ok(accessory.clone())
}

async fn check_ids(name_c: String, accessory: &Arc<RwLock<Box<dyn HapAccessory>>>) -> anyhow::Result<()> {
    let mut ids = vec![];
    for ch in accessory.read().await.get_services() {
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