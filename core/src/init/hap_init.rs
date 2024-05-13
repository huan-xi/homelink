use std::str::FromStr;
use anyhow::anyhow;
use axum::body::HttpBody;
use log::error;
use sea_orm::{ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait, JsonValue, QueryFilter};
use tokio::sync::RwLock;

use hap::accessory::{AccessoryInformation, HapAccessory};
use hap::accessory::bridge::BridgeAccessory;
use hap::characteristic::configured_name::ConfiguredNameCharacteristic;
use hap::characteristic::{CharacteristicCallbacks, HapCharacteristic};
use hap::HapType;
use hap::server::{IpServer, Server};
use hap::service::HapService;
use hap::storage::Storage;

use crate::config::context::get_app_context;
use crate::db::entity::prelude::{HapAccessoryColumn, HapAccessoryEntity, HapBridgeColumn, HapBridgeEntity, HapBridgeModel, HapCharacteristicModel, HapServiceModel};
use crate::hap::db_bridge_storage::DbBridgesStorage;
use crate::init::{DevicePointer, HapAccessoryPointer};
use crate::init::characteristic_init::to_characteristic;
use crate::init::manager::device_manager::IotDeviceManager;
use target_hap::hap_manager::{AccessoryRelation, HapManage};
use target_hap::hap_type_wrapper::HapTypeWrapper;
use target_hap::iot::iot_hap_service::IotHapService;

pub async fn init_hap_list(conn: &DatabaseConnection, manage: HapManage, iot_device_map: IotDeviceManager) -> anyhow::Result<()> {
    let bridges = HapBridgeEntity::find()
        .filter(HapBridgeColumn::Disabled.eq(false))
        .all(conn)
        .await?;
    // let mut servers = Vec::new();
    for bridge in bridges.into_iter() {
        if let Err(e) = add_hap_bridge(conn, bridge, manage.clone(), iot_device_map.clone()).await {
            error!("初始化桥接器失败:{}",e);
        }
    }
    Ok(())
}


/// 对于设备的依赖就是特征读写值,平台设备要转成 hap_platform 设备就必须实现,特征读写方法
/// 添加单个bridge 到管理器中
pub async fn add_hap_bridge(conn: &DatabaseConnection, bridge: HapBridgeModel, manage: HapManage, iot_device_map: IotDeviceManager) -> anyhow::Result<()> {
    let config = &get_app_context().config;

    // let str = format!("{}/{}_{}", config.server.data_dir, "hap_platform", bridge.pin_code);
    // let mut storage = FileStorage::new(str.as_str()).await?;
    let bid = bridge.bridge_id;
    let bridge_model = HapBridgeEntity::find_by_id(bid)
        .one(conn)
        .await?
        .ok_or(anyhow!("未找到桥接器:{:?}", bid))?;
    let is_single_accessory = bridge_model.single_accessory;

    let mut storage = DbBridgesStorage::new(bid, conn.clone(),RwLock::new(None));
    //config
    let mut hap_config = storage.load_config().await?;
    hap_config.redetermine_local_ip();
    storage.save_config(&hap_config).await?;

    let mdns = manage.get_mdns().await?;

    let server = IpServer::new(hap_config, storage, mdns).await?;
    if is_single_accessory {
        //取设备上的属性覆盖
        let accessories = HapAccessoryEntity::find()
            .filter(HapAccessoryColumn::BridgeId.eq(bridge_model.bridge_id))
            .all(conn)
            .await?;
        if accessories.len() == 0 {
            return Err(anyhow!("单配件桥接器没有配件"));
        }
        if accessories.len() > 1 {
            return Err(anyhow!("单配件桥接器配件大于1"));
        }
    } else {
        // 初始化bridge 设备
        let bridge = BridgeAccessory::new(1, AccessoryInformation {
            name: bridge_model.name.clone(),
            serial_number: bridge_model.info.serial_number,
            software_revision: bridge_model.info.software_revision,
            manufacturer: bridge_model.info.manufacturer,
            model: bridge_model.info.model,
            ..Default::default()
        })?;
        server.add_accessory(bridge).await?;
    }

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


/// 基于设备初始化配件列表
async fn init_hap_accessories<C: ConnectionTrait>(conn: &C, hap_manage: HapManage, bridge_id: i64, iot_device_map: IotDeviceManager) -> anyhow::Result<Vec<AccessoryRelation>> {
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
    let hap_type: HapType = HapTypeWrapper::from_str(service.service_type.as_str())?.into();
    let mut hap_service = IotHapService::new(ctx.sid, ctx.aid, hap_type, service.tag.clone());
    hap_service.set_primary(service.primary);
    let stag = service.tag.clone();
    let mut success = false;
    for (index, ch) in chs.into_iter()
        .filter(|ch| !ch.disabled).enumerate().into_iter() {
        let ctx = ctx.clone();
        match to_characteristic(ctx, index, ch).await {
            Ok(cts) => {
                hap_service.push_characteristic(Box::new(cts));
                success = true;
            }
            Err(e) => {
                error!("特征映射失败:{:?}", e);
            }
        };
    }
    // hap_service.set_primary(true);
    // for (index, ch) in chs.into_iter().enumerate() {}
    let mut len = hap_service.get_characteristics().len();
    // 设置名称
    if let Some(n) = service.configured_name.clone() {
        if !n.is_empty() {
            let id = ctx.sid + len as u64 + 1;
            len += 1;
            let mut name = ConfiguredNameCharacteristic::new(id, ctx.aid);
            name.set_value(JsonValue::String(n.clone())).await?;
            name.on_read(Some(move || {
                let name = n.clone();
                {
                    Ok(Some(name))
                }
            }));
            hap_service.push_characteristic(Box::new(name));
        }
    };

    if success {
        ctx.accessory.write().await.push_service(service.tag, Box::new(hap_service));
    } else {
        error!("服务没有可用的特征:{:?}", service);
    }
    Ok(len)
}


#[test]
pub fn test_format() {
    /* let f = Format::from_str(Default::default());
     println!("{:?}", f);*/
}