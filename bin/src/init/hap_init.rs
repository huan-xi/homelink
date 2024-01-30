use std::str::FromStr;
use std::sync::Arc;

use anyhow::anyhow;
use axum::body::HttpBody;
use futures_util::FutureExt;
use log::{error, info};
use rand::Rng;
use sea_orm::{ColIdx, ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait, JsonValue, QueryFilter};
use tokio::sync::Mutex;

use hap::{Config, MacAddress};
use hap::accessory::{AccessoryCategory, AccessoryInformation, HapAccessory};
use hap::accessory::bridge::BridgeAccessory;
use hap::characteristic::configured_name::ConfiguredNameCharacteristic;
use hap::characteristic::HapCharacteristic;
use hap::server::{IpServer, Server};
use hap::service::HapService;
use hap::storage::{FileStorage, Storage};
use miot_spec::device::miot_spec_device::MiotSpecDevice;

use crate::config::context::get_app_context;
use crate::db::entity::prelude::{HapAccessoryColumn, HapAccessoryEntity, HapAccessoryModel, HapBridgeColumn, HapBridgeEntity, HapBridgeModel, HapCharacteristicEntity, HapCharacteristicModel, HapServiceColumn, HapServiceEntity, HapServiceModel};
use crate::db::SNOWFLAKE;
use crate::hap::iot::iot_hap_accessory::IotHapAccessory;
use crate::hap::iot::iot_hap_service::IotHapService;
use crate::hap::models::{AccessoryModel, AccessoryModelContext};
use crate::hap::rand_utils::{compute_setup_hash, pin_code_from_str};
use crate::init::{DevicePointer, HapAccessoryPointer};
use crate::init::manager::device_manager::IotDeviceManager;
use crate::init::manager::hap_manager::HapManage;
use crate::init::characteristic_init::to_characteristic;


pub async fn init_hap_list(conn: &DatabaseConnection, manage: HapManage, iot_device_map: IotDeviceManager) -> anyhow::Result<()> {
    let bridges = HapBridgeEntity::find()
        .filter(HapBridgeColumn::Disabled.eq(false))
        .all(conn)
        .await?;
    // let mut servers = Vec::new();
    for bridge in bridges.into_iter() {
        add_hap_bridge(conn, bridge, manage.clone(), iot_device_map.clone()).await?;
    }
    Ok(())
}

#[test]
pub fn print_dir() {
    let u64: u64 = 1201411032090673152;
    let hex = format!("{:x}", u64);
    println!("{:?}", hex);
}

/// 添加单个bridge 到管理器中
pub async fn add_hap_bridge<C>(conn: &C, bridge: HapBridgeModel, manage: HapManage, iot_device_map: IotDeviceManager) -> anyhow::Result<()>
    where C: ConnectionTrait, {
    let config = &get_app_context().config;

    let str = format!("{}/{}_{}", config.server.data_dir, "hap", bridge.pin_code);
    let mut storage = FileStorage::new(str.as_str()).await?;
    let bid = bridge.bridge_id;

    //config
    let hap_config = match storage.load_config().await {
        Ok(mut config) => {
            config.redetermine_local_ip();
            config.port = bridge.port as u16;
            storage.save_config(&config).await?;
            config
        }
        Err(_) => {
            let pin = pin_code_from_str(bridge.pin_code.to_string().as_str());
            let name = bridge.name.clone();
            let setup_hash = compute_setup_hash(bridge.setup_id.as_str(), bridge.mac.as_str());
            // let setup_hash=
            let config = Config {
                pin,
                name,
                //mac 地址配置
                device_id: MacAddress::from_str(bridge.mac.as_str())?,
                port: bridge.port as u16,
                category: AccessoryCategory::Bridge,
                setup_id: bridge.setup_id.clone(),
                setup_hash,
                ..Default::default()
            };
            storage.save_config(&config).await?;
            config
        }
    };

    let serial_number = {
        let mut rng = rand::thread_rng();
        let random_bytes: [u8; 16] = rng.gen();
        hex::encode(random_bytes)
    };


    let server = IpServer::new(hap_config, storage).await?;
    // 初始化bridge 设备
    let bridge = BridgeAccessory::new(1, AccessoryInformation {
        name: bridge.name.clone(),
        serial_number,
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
    Ok(())
}

pub struct AccessoryRelation {
    pub aid: u64,
    pub device_id: i64,
    pub accessory: HapAccessoryPointer,
}

/// 配件是基于设备的
async fn init_hap_accessories<C: ConnectionTrait>(conn: &C,
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
                super::accessory_init::init_hap_accessory(conn, hap_manage.clone(), device, hap_accessory).await
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
                error!("初始化配件:{aid}失败:{e}");
            }
        }
    }
    Ok(list)
}



#[derive(Clone)]
pub struct InitServiceContext {
    pub aid: u64,
    /// 特征id
    pub sid: u64,
    pub stag: Option<String>,
    /// 设备指针
    pub device: DevicePointer,
    pub accessory: HapAccessoryPointer,
    pub hap_manage: HapManage,
}

/// 服务映射
/// cid 自增的 每次+特征的长度
pub(crate) async fn add_service(ctx: InitServiceContext, service_chs: (HapServiceModel, Vec<HapCharacteristicModel>)) -> anyhow::Result<usize> {
    let service = service_chs.0;
    let chs = service_chs.1;
    let mut hap_service = IotHapService::new(ctx.sid, ctx.aid, service.service_type.into());
    let stag = service.tag.clone();
    let mut success = false;
    for (index, ch) in chs.into_iter()
        .filter(|ch| !ch.disabled).enumerate().into_iter() {
        let ctx = ctx.clone();
        let c_tag = stag.clone();
        match to_characteristic(ctx, index, ch).await {
            Ok(cts) => {
                hap_service.push_characteristic(c_tag, Box::new(cts));
                success = true;
            }
            Err(e) => {
                error!("特征映射失败:{:?}", e);
            }
        };
    }

    // for (index, ch) in chs.into_iter().enumerate() {}
    let len = hap_service.get_characteristics().len();
    // 设置名称
    if let Some(n) = service.name.clone() {
        if !n.is_empty() {
            let id = ctx.sid + len as u64 + 1;
            let mut name = ConfiguredNameCharacteristic::new(id, ctx.aid);
            name.set_value(JsonValue::String(n)).await?;
            hap_service.push_characteristic(Some("configured-name".to_string()), Box::new(name));
        }
    };

    if success {
        ctx.accessory.lock().await.push_service(service.tag, Box::new(hap_service));
    } else {
        error!("服务:{:?}没有可用的特征", service);
    }
    Ok(len)
}


#[test]
pub fn test_format() {
    /* let f = Format::from_str(Default::default());
     println!("{:?}", f);*/
}