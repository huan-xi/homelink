use std::str::FromStr;
use std::sync::Arc;
use anyhow::anyhow;
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
use sea_orm::{ColIdx, ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait, JsonValue, ModelTrait, QueryFilter};
use hap::characteristic::{HapCharacteristic};
use hap::characteristic::configured_name::ConfiguredNameCharacteristic;
use miot_spec::device::miot_spec_device::{MiotSpecDevice};
use crate::config::context::get_app_context;
use crate::hap::iot_characteristic::IotCharacteristic;
use crate::hap::iot_hap_accessory::{IotDeviceAccessory, IotHapAccessory};
use crate::db::entity::prelude::{HapAccessoryColumn, HapAccessoryEntity, HapAccessoryModel, HapBridgeEntity, HapBridgeColumn, HapCharacteristicEntity, HapCharacteristicModel, HapServiceColumn, HapServiceEntity, HapServiceModel, HapBridgeModel};
use crate::db::SNOWFLAKE;
use crate::init::{DevicePointer, FuturesMutex, HapAccessoryPointer};
use crate::hap::iot_hap_service::IotHapService;
use crate::hap::rand_utils::{compute_setup_hash, pin_code_from_str, rand_pin_code};
use crate::init::manager::device_manager::{IotDeviceManager, IotDeviceManagerInner};
use crate::init::manager::hap_manager::HapManage;
use crate::init::mapping_characteristic::{to_characteristic, ToChUtils};
use crate::js_engine::init_hap_accessory_module::init_hap_accessory_module;


pub async fn init_haps(conn: &DatabaseConnection, manage: HapManage, iot_device_map: IotDeviceManager) -> anyhow::Result<()> {
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

/// 添加单个bridge 到管理器中
pub async fn add_hap_bridge<C>(conn: &C, bridge: HapBridgeModel, manage: HapManage, iot_device_map: IotDeviceManager) -> anyhow::Result<()>
    where C: ConnectionTrait, {
    let config = &get_app_context().config;

    let hex = format!("{:x}", bridge.bridge_id);
    let str = format!("{}/{}_{}", config.server.data_dir, "hap", hex);
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
                error!("初始化配件:{aid}失败:{e}");
            }
        }
    }
    Ok(list)
}

/// 初始化配件的设备
/// 需要建立配件与设备的关系,处理设备情况
async fn init_hap_accessory<'a, C: ConnectionTrait>(conn: &C,
                                                    hap_manage: HapManage,
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
    let mut info = AccessoryInformation {
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
    if services.is_empty() {
        return Err(anyhow!("配件:{},无服务",name_c));
    };
    let ha = IotHapAccessory::new(aid, hss);
    let accessory = Arc::new(FuturesMutex::new(Box::new(ha) as Box<dyn HapAccessory>));
    let ch_id = SNOWFLAKE.next_id();
    // 注册到manage 上
    hap_manage.put_accessory_ch(aid, ch_id, false).await;

    for service in services.into_iter() {
        let len = add_service(aid, cid, service, device.clone(), accessory.clone()).await?;
        cid += len as u64 + 1;
        // 转成服务, 服务需要服务类型和服务的必填特征
    }

    // 查询script
    if let Some(script) = hap_accessory.script {
        if !script.is_empty() {
            // 初始化hap js模块,
            init_hap_accessory_module(hap_manage, ch_id, aid, script.as_str()).await?;
        };
    };

    Ok(accessory.clone())
}


/// 服务映射
/// cid 自增的 每次+特征的长度
async fn add_service(aid: u64, cid: u64, service_chs: (HapServiceModel, Vec<HapCharacteristicModel>), device: DevicePointer, accessory: HapAccessoryPointer) -> anyhow::Result<usize> {
    let service = service_chs.0;
    let chs = service_chs.1;
    let mut hap_service = IotHapService::new(cid, aid, service.service_type.into());
    let stag = service.tag.clone();
    let chs: Vec<BoxFuture<anyhow::Result<(Option<String>, IotCharacteristic)>>> = chs.into_iter()
        .filter(|ch| !ch.disabled)
        .enumerate()
        .into_iter()
        .map(|(index, ch)| {
            let dev = device.clone();
            let acc = accessory.clone();
            let stag = stag.clone();
            async move {
                let c_tag = ch.tag.clone();
                let cts = to_characteristic(cid, aid, stag, index, ch, dev, acc).await?;
                Ok((c_tag, cts))
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


#[test]
pub fn test_format() {
    /* let f = Format::from_str(Default::default());
     println!("{:?}", f);*/
}