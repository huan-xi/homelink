use std::sync::Arc;
use axum::body::HttpBody;
use futures_util::future::{BoxFuture, join_all};
use futures_util::FutureExt;
use hap::{Config, HapType, MacAddress, Pin};
use hap::accessory::{AccessoryCategory, AccessoryInformation, HapAccessory};
use hap::accessory::bridge::BridgeAccessory;
use hap::server::{IpServer, Server};
use hap::service::HapService;
use hap::storage::{FileStorage, Storage};
use log::{debug, error, info};
use rand::Rng;
use sea_orm::{ColIdx, ColumnTrait, DatabaseConnection, EntityTrait, JsonValue, ModelTrait, QueryFilter};
use hap::characteristic::{HapCharacteristic};
use hap::characteristic::configured_name::ConfiguredNameCharacteristic;
use miot_spec::device::miot_spec_device::{MiotSpecDevice};
use crate::config::cfgs::Configs;
use crate::config::context::get_app_context;
use crate::hap::iot_characteristic::IotCharacteristic;
use crate::hap::iot_hap_accessory::{IotDeviceAccessory, IotHapAccessory};
use crate::db::entity::prelude::{HapAccessoryColumn, HapAccessoryEntity, HapAccessoryModel, HapBridgeEntity, HapBridgeColumn, HapCharacteristicEntity, HapCharacteristicModel, HapServiceColumn, HapServiceEntity, HapServiceModel};
use crate::init::{DevicePointer, FuturesMutex, HapAccessoryPointer};
use crate::hap::iot_hap_service::IotHapService;
use crate::init::device_manage::{IotDeviceManager, IotDeviceManagerInner};
use crate::init::hap_manage::HapManage;
use crate::init::mapping_characteristic::{to_characteristic, ToChUtils};
use crate::js_engine::init_hap_accessory_module::init_hap_accessory_module;

pub fn rand_num() -> u32 {
    let mut rng = rand::thread_rng();
    let invalid_numbers: [u32; 12] = [0, 11111111, 22222222, 33333333, 44444444, 55555555, 66666666, 77777777, 88888888, 99999999, 12345678, 87654321];

    let random_number = loop {
        let number = rng.gen_range(100_000_00, 999_999_99);
        if !invalid_numbers.contains(&number) {
            break number;
        }
    };
    random_number
}

pub async fn init_hap(conn: &DatabaseConnection, manage: HapManage, iot_device_map: IotDeviceManager) -> anyhow::Result<()> {
    let bridges = HapBridgeEntity::find()
        .filter(HapBridgeColumn::Disabled.eq(false))
        .all(conn).await?;
    // let mut servers = Vec::new();
    for bridge in bridges.into_iter() {
        let config = &get_app_context().config;

        let hex = format!("{:x}", bridge.bridge_id);
        let str = format!("{}/{}_{}", config.server.data_dir, "hap", hex);
        let mut storage = FileStorage::new(str.as_str()).await?;
        let bid = bridge.bridge_id;

        //config
        let hap_config = match storage.load_config().await {
            Ok(mut config) => {
                config.redetermine_local_ip();
                storage.save_config(&config).await?;
                config
            }
            Err(_) => {
                let pin = pin_from_str(bridge.pin_code.to_string().as_str());
                let name = bridge.name.clone();
                //todo 分类
                let config = Config {
                    pin,
                    name,
                    //mac 地址配置
                    device_id: MacAddress::from([10, 21, 30, 40, 50, 60]),
                    category: AccessoryCategory::Bridge,
                    ..Default::default()
                };
                storage.save_config(&config).await?;
                config
            }
        };

        let server = IpServer::new(hap_config, storage).await?;
        // 初始化bridge 设备 //todo 生成随机端口
        let bridge = BridgeAccessory::new(1, AccessoryInformation {
            name: bridge.name.clone(),
            serial_number: "00000".into(),
            software_revision: Some(config.server.version.clone()),
            ..Default::default()
        })?;
        server.add_accessory(bridge).await?;

        // 初始化其他配件
        let accessories = init_hap_accessories(conn,
                                               manage.clone(),
                                               bid, iot_device_map.clone()).await?;

        for accessory in accessories.iter() {
            server.add_arc_accessory(accessory.accessory.clone()).await?;
        }
        server.configuration_number_incr().await;
        manage.push_server(bid, server, accessories);

    }
    Ok(())
}

pub struct AccessoryRelation {
    pub aid: u64,
    pub device_id: i64,
    pub accessory: HapAccessoryPointer,
}

/// 配件是基于设备的
async fn init_hap_accessories(conn: &DatabaseConnection,
                              hap_manage: HapManage,
                              bridge_id: i64, iot_device_map: IotDeviceManager) -> anyhow::Result<Vec<AccessoryRelation>> {
    let hap_accessories = HapAccessoryEntity::find()
        .filter(HapAccessoryColumn::BridgeId.eq(bridge_id)
            .and(HapAccessoryColumn::Disabled.eq(false)))
        .all(conn)
        .await?;
    let mut list = vec![];
    for hap_accessory in hap_accessories.into_iter() {
        let aid = hap_accessory.aid;
        let device_id = hap_accessory.device_id;
        let result = match iot_device_map.get_device(device_id) {
            None => {
                Err(anyhow::anyhow!("未找到设备:{:?}", hap_accessory.device_id))
            }
            Some(device) => {
                init_hap_accessory(conn, hap_manage.clone(), device, hap_accessory).await
            }
        };
        match result {
            Ok(ptr) => {
                list.push(AccessoryRelation {
                    aid: aid as u64,
                    device_id,
                    accessory: ptr,
                });
            }
            Err(e) => {
                error!("初始化配件:{aid}失败:{:?}", e);
            }
        }
    }
    Ok(list)
}

/// 初始化配件的设备
/// 需要建立配件与设备的关系,处理设备情况
async fn init_hap_accessory<'a>(conn: &DatabaseConnection,
                                hap_manage: HapManage,
                                device: DevicePointer, hap_accessory: HapAccessoryModel) -> anyhow::Result<HapAccessoryPointer> {
    let aid = hap_accessory.aid as u64;
    let mut hss: Vec<Box<dyn HapService>> = vec![];
    // let device = device.value().read().await.device.clone();
    let device = device.clone();
    // 初始化配件服务

    let dev_info = device.get_info().clone();
    let name = hap_accessory.name.unwrap_or(dev_info.name.clone());
    // 可以从设备信息中获取
    let mut info = AccessoryInformation {
        name,
        model: dev_info.model.clone(),
        firmware_revision: dev_info.firmware_revision.clone(),
        software_revision: dev_info.software_revision.clone(),
        serial_number: dev_info.serial_number.clone().unwrap_or("undefined".to_string()),
        manufacturer: dev_info.manufacturer.clone().unwrap_or("undefined".to_string()),
        // configured_name: Some(dev_info.model.clone()),
        ..Default::default()
    };
    // SwitchAccessory::new(1, info.clone())?;
    let mut cid = 1;
    let mut service = info.to_service(cid, aid)?;
    cid += service.get_characteristics().len() as u64 + 1;
    hss.push(Box::new(service));
    // 初始化子服务
    let services = HapServiceEntity::find()
        .filter(HapServiceColumn::AccessoryId.eq(hap_accessory.aid).and(HapServiceColumn::Disabled.eq(false)))
        .find_with_related(HapCharacteristicEntity)
        .all(conn)
        .await?;
    let mut ha = IotHapAccessory::new(aid, hss);
/*    for (svc_model, _) in &services {
        if let Some(tag) = svc_model.tag.clone(){
            ha.tag_ids_map.entry(tag.clone()).or_insert(vec![]).push(svc_model.id as u64);
        }
    }*/
    let accessory = Arc::new(FuturesMutex::new(Box::new(ha) as Box<dyn HapAccessory>));

    for service in services.into_iter() {
        let len = add_service(aid, cid, service, device.clone(), accessory.clone()).await?;
        cid += len as u64 + 1;
        // 转成服务, 服务需要服务类型和服务的必填特征
    }
    // 查询script
    if let Some(script) = hap_accessory.script {
        // 初始化hap js模块,
        init_hap_accessory_module(hap_manage, aid, script.as_str()).await?;
    };

    Ok(accessory.clone())
}


/// 服务映射
/// cid 自增的 每次+特征的长度
async fn add_service(aid: u64, cid: u64, service_chs: (HapServiceModel, Vec<HapCharacteristicModel>), device: DevicePointer,
                     accessory: HapAccessoryPointer) -> anyhow::Result<usize> {
    let service = service_chs.0;
    let chs = service_chs.1;
    let mut hap_service = IotHapService::new(cid, aid, service.service_type.into());

    let chs: Vec<BoxFuture<anyhow::Result<(Option<String>, IotCharacteristic)>>> = chs.into_iter()
        .filter(|ch| !ch.disabled)
        .enumerate()
        .into_iter()
        .map(|(index, ch)| {
            let dev = device.clone();
            let acc = accessory.clone();
            async move {
                let tag = ch.tag.clone();
                let cts = to_characteristic(cid, aid, index, ch, dev, acc).await?;
                Ok((tag, cts))
            }.boxed()
        }).collect();

    let chs_list = join_all(chs).await;
    let mut success = false;
    for ch in chs_list.into_iter() {
        match ch {
            Ok((tag, cts)) => {
                hap_service.push_characteristic(tag, Box::new(cts));
                success = true;
            }
            Err(e) => {
                error!("转换失败特征:{:?}", e);
            }
        }
    }
    // for (index, ch) in chs.into_iter().enumerate() {}
    let len = hap_service.get_characteristics().len();
    // 设置名称
    if let Some(n) = service.name.clone() {
        if !n.is_empty() {
            let id = cid + len as u64 + 1;
            let mut name = ConfiguredNameCharacteristic::new(id, aid);
            name.set_value(JsonValue::String(n)).await?;
            hap_service.push_characteristic(Some("configured-name".to_string()), Box::new(name));
        }
    };

    if success {
        accessory.lock().await.push_service(service.tag, Box::new(hap_service));
    } else {
        error!("服务:{:?}没有可用的特征", service);
    }
    Ok(len)
}


pub fn pin_from_str(pin: &str) -> Pin {
    let mut arr: [u8; 8] = [0; 8]; // 初始化一个长度为8的u8数组
    for (i, c) in pin.chars().enumerate() {
        if i < 8 {
            arr[i] = c.to_digit(10).unwrap() as u8
        }
    }
    Pin::new(arr).unwrap()
}

#[test]
pub fn test_format() {
    /* let f = Format::from_str(Default::default());
     println!("{:?}", f);*/
}