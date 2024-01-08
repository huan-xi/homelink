use std::sync::Arc;
use axum::body::HttpBody;
use futures_util::FutureExt;
use hap::{Config, HapType, MacAddress, Pin};
use hap::accessory::{AccessoryCategory, AccessoryInformation, HapAccessory};
use hap::accessory::bridge::BridgeAccessory;
use hap::characteristic::{AsyncCharacteristicCallbacks, Characteristic, CharacteristicCallbacks, Format, HapCharacteristic, Perm};
use hap::characteristic::color_temperature::ColorTemperatureCharacteristic;
use hap::characteristic::power_state::PowerStateCharacteristic;
use hap::server::{IpServer, Server};
use hap::service::HapService;
use hap::service::outlet::OutletService;
use hap::service::switch::SwitchService;
use hap::service::temperature_sensor::TemperatureSensorService;
use hap::storage::{FileStorage, Storage};
use log::{debug, error, info};
use rand::Rng;
use sea_orm::{ColIdx, ColumnTrait, DatabaseConnection, EntityTrait, ModelTrait, QueryFilter};
use tap::TapFallible;
use tokio::sync::Mutex;
use miot_spec::device::miot_spec_device::{MiotDeviceType, MiotSpecDevice};
use crate::config::cfgs::Configs;
use crate::convertor::iot_hap_accessory::{IotDeviceAccessory, IotHapAccessory};
use crate::convertor::miot2hap::{ServiceSetter, Utils};
use crate::db::entity::prelude::{HapAccessoryColumn, HapAccessoryEntity, HapAccessoryModel, HapBridge, HapBridgeColumn, HapCharacteristicEntity, HapCharacteristicModel, HapServiceColumn, HapServiceEntity, HapServiceModel};
use crate::init::{AFuturesMutex, DeviceMap, DevicePointer, FuturesMutex, HapAccessoryPointer};
use crate::convertor::iot_hap_service::IotHapService;
use crate::init::mapping_characteristic::{to_characteristic, ToChUtils};

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

pub async fn init_hap(conn: &DatabaseConnection, config: &Configs, iot_device_map: DeviceMap) -> anyhow::Result<Vec<IpServer>> {
    let bridges = HapBridge::find()
        .filter(HapBridgeColumn::Disabled.eq(false))
        .all(conn).await?;
    let mut servers = Vec::new();
    for bridge in bridges.into_iter() {
        let hex = format!("{:x}", bridge.bridge_id);
        let str = format!("{}/{}_{}", config.server.data_dir, "hap", hex);
        let mut storage = FileStorage::new(str.as_str()).await?;
        let bid = bridge.bridge_id;
        let pin = pin_from_str(bridge.bridge_id.to_string().as_str());
        let name = bridge.name.clone();
        //config
        let hap_config = match storage.load_config().await {
            Ok(mut config) => {
                config.redetermine_local_ip();
                storage.save_config(&config).await?;
                config
            }
            Err(_) => {
                let pin = config.hap_config.pin();
                let name = config.hap_config.name();
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
        let accessories = init_hap_accessories(conn, bid, iot_device_map.clone()).await?;
        for accessory in accessories.into_iter() {
            server.add_arc_accessory(accessory).await?;
        }
        server.configuration_number_incr().await;
        servers.push(server);
    }

    Ok(servers)
}

/// 配件是基于设备的
async fn init_hap_accessories(conn: &DatabaseConnection, bridge_id: i64, iot_device_map: DeviceMap) -> anyhow::Result<Vec<HapAccessoryPointer>> {
    let hap_accessories = HapAccessoryEntity::find()
        .filter(HapAccessoryColumn::BridgeId.eq(bridge_id)
            .and(HapAccessoryColumn::Disabled.eq(false)))
        .all(conn)
        .await?;
    let mut list = vec![];
    for hap_accessory in hap_accessories.into_iter() {
        match init_hap_accessory(conn, &iot_device_map, hap_accessory).await {
            Ok(ok) => {
                list.push(ok);
            }
            Err(e) => {
                error!("初始化配件失败:{:?}", e);
            }
        }
    }
    Ok(list)
}

/// 初始化配件的设备
/// 需要建立配件与设备的关系,处理设备情况
async fn init_hap_accessory<'a>(conn: &DatabaseConnection, iot_device_map: &DeviceMap, hap_accessory: HapAccessoryModel) -> anyhow::Result<HapAccessoryPointer> {
    let device = match iot_device_map.get(&hap_accessory.device_id) {
        None => {
            return Err(anyhow::anyhow!("未找到设备:{:?}", hap_accessory.device_id));
        }
        Some(d) => { d }
    };
    let aid = hap_accessory.aid as u64;
    let mut hss: Vec<Box<dyn HapService>> = vec![];
    // let device = device.value().read().await.device.clone();
    let device = device.clone();
    // 初始化配件服务

    let dev_info = device.get_info().clone();

    // 可以从设备信息中获取
    let mut info = AccessoryInformation {
        name: dev_info.name.clone(),
        model: dev_info.model.clone(),
        firmware_revision: dev_info.firmware_revision.clone(),
        software_revision: dev_info.software_revision.clone(),
        serial_number: dev_info.serial_number.clone().unwrap_or("undefined".to_string()),
        manufacturer: dev_info.manufacturer.clone().unwrap_or("undefined".to_string()),
        ..Default::default()
    };
    // SwitchAccessory::new(1, info.clone())?;
    let mut cid = 1;
    let service = info.to_service(cid, aid)?;
    cid += service.get_characteristics().len() as u64 + 1;
    hss.push(Box::new(service));
    // 初始化子服务
    let services = HapServiceEntity::find()
        .filter(HapServiceColumn::AccessoryId.eq(hap_accessory.id).and(HapServiceColumn::Disabled.eq(false)))
        .find_with_related(HapCharacteristicEntity)
        .all(conn)
        .await?;
    let mut ha = IotHapAccessory::new(aid, hss);

    let accessory = Arc::new(FuturesMutex::new(Box::new(ha) as Box<dyn HapAccessory>));
    for service in services.into_iter() {
        let len = add_service(aid, cid, service, device.clone(), accessory.clone()).await?;
        cid += len as u64 + 1;

        // 转成服务, 服务需要服务类型和服务的必填特征
        /* match add_service(aid, cid, service, device.clone()).await {
             Ok(hs) => {
                 cid += hs.get_characteristics().len() as u64 + 1;
                 hss.push(hs);
                 // ha.inner.services.push(hs);
             }
             Err(e) => {}
         };*/
    }


    Ok(accessory.clone())
}


/// 服务映射
async fn add_service(aid: u64, sid: u64, service_chs: (HapServiceModel, Vec<HapCharacteristicModel>), device: Arc<dyn MiotSpecDevice + Send + Sync>,
                     accessory: HapAccessoryPointer) -> anyhow::Result<usize> {
    let service = service_chs.0;
    let chs = service_chs.1;
    let mut hap_service = IotHapService::new(sid, aid, service.hap_type.into());
    for (index, ch) in chs.into_iter().enumerate() {
        if ch.disabled {
            continue;
        };
        let cid = ch.cid as u64;
        match to_characteristic(sid, aid, index, ch, device.clone(), accessory.clone()).await {
            Ok(cts) => {
                debug!("cts 创建成功:{:?}", cts);
                hap_service.push_characteristic(Box::new(cts));
            }
            Err(e) => {
                error!("cid:{},转换失败:{:?}",  cid,e);
                continue;
            }
        };
    }
    let len = hap_service.get_characteristics().len();
    accessory.lock().await.push_service(Box::new(hap_service));
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