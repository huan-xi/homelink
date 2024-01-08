use std::fs;
use std::net::SocketAddr;
use std::sync::{Arc};
use std::time::Duration;
use axum::body::HttpBody;
use axum::Router;
use futures_util::FutureExt;
use dashmap::DashMap;
use futures_util::future::{BoxFuture, join_all};
use futures_util::StreamExt;
use hap::{Config, MacAddress, Pin};
use hap::accessory::{AccessoryCategory, AccessoryInformation, HapAccessory};
use hap::accessory::bridge::BridgeAccessory;
use hap::accessory::lightbulb::LightbulbAccessory;
use hap::accessory::switch::SwitchAccessory;
use hap::futures::future::{join, join3};
use hap::server::{IpServer, Server};
use hap::service::HapService;
use hap::service::switch::SwitchService;
use hap::storage::{FileStorage, Storage};
use log::{error, info};
use sea_orm::{ColumnTrait, ConnectOptions, Database, DatabaseConnection, EntityTrait, QueryFilter};
use sea_orm::ActiveValue::Set;
use tokio::sync::{Mutex, RwLock};
use bin::api::router;
use bin::api::state::{AppState, AppStateInner};
use bin::config::cfgs::Configs;
use bin::convertor::hap_type::{MappingHapType};
use bin::db::entity::hap_service::{ Property};
use bin::db::entity::iot_device::{DeviceParam, IotDeviceType};
use bin::db::entity::prelude::{DeviceMapping, DeviceMappingColumn, HapServiceActiveModel, HapServiceEntity, HapServiceModel, IotDevice, IotDeviceActiveModel, IotDeviceColumn};
use bin::init;
use bin::init::{DeviceMap, DevicePointer};
use miot_spec::device::ble::ble_device::BleDevice;
use miot_spec::device::gateway::gateway::OpenMiioGatewayDevice;
use miot_spec::device::miot_spec_device::{DeviceInfo, MiotSpecDevice};
use miot_spec::device::wifi_device::{ExitCode, WifiDevice};


pub async fn db_conn() -> DatabaseConnection {
    /*   let link = match CFG.database.link.clone() {
           None => {
               let p = CFG.server.data_dir.as_str();
               match fs::metadata(p) {
                   Ok(f) => {
                       assert!(f.is_dir(), "数据目录不是目录");
                   }
                   Err(e) => {
                       fs::create_dir_all(p).expect(format!("创建数据目录:{:?}失败", p).as_str());
                   }
               }
               let path = std::fs::canonicalize(CFG.server.data_dir.as_str()).unwrap();
               let schema = format!("sqlite://{}/data.db?mode=rwc", path.to_str().unwrap());
               println!("设置默认数据:{}", schema);
               schema
           }
           Some(str) => str.clone(),
       };

       open_db(link).await*/
    let path = std::fs::canonicalize("/Users/huanxi/project/home-gateway/data").unwrap();
    let schema = format!("sqlite://{}/data.db?mode=rwc", path.to_str().unwrap());
    open_db(schema).await
}

pub async fn open_db(schema: String) -> DatabaseConnection {
    let mut opt = ConnectOptions::new(schema);
    opt.max_connections(1000)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .sqlx_logging(false);
    let db = Database::connect(opt).await.expect("数据库打开失败");
    info!("Database connected");
    db
}

/// 先创建http服务
/// 创建homekit 服务
/// 创建米家设备
/// 映射米家设备到homekit设备
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();
    //读取配置
    let config = Configs::init();
    let addr = SocketAddr::new([0, 0, 0, 0].into(), 5514);
    let conn = db_conn().await;
    // 初始化hap 服务器

    // 初始化iot设备
    let iot_device_map = init_iot_device(&conn).await?;

    // 添加桥接设备
    // let mut hap_servers = init_hap(&conn, &config).await?;
    info!("初始化设备成功:{:?}",iot_device_map.iter().map(|i| i.key().clone()).collect::<Vec<_>>() );

    let mut hap_servers = init::hap_init::init_hap(&conn, &config, iot_device_map.clone()).await?;
    let hap_server = Arc::new(hap_servers.remove(0));
    let app_state = AppState {
        inner: Arc::new(AppStateInner::new(conn.clone(), hap_server.clone()))
    };
    let app = Router::new()
        .nest("/api",router::api())
        .with_state(app_state);

    let api_server = axum_server::Server::bind(addr.clone())
        .serve(app.into_make_service());

    info!("api_server start at :{:?}", addr);
    // api_server.await?;
    //启动device
    /*    let mut list = vec![];
        for v in iot_device_map.iter() {
            let dev = v.value().clone();
            list.push(async move {
                let a = dev.clone();
                /* match dev.lock().await.run().await {
                     Ok(_) => {}
                     Err(_) => {}
                 }*/
            }.boxed());
        }*/

    let device_handlers: Vec<BoxFuture<()>> = iot_device_map.iter().map(|i| {
        let dev = i.value().clone();
        async move {
            match dev.run().await {
                Ok(_) => {}
                Err(_) => {}
            }
            error!("设备连接断开:{:?}", dev.get_info().did);
        }.boxed()
    }).collect();
    // join_all(device_handlers);
    join3(join_all(device_handlers), api_server, hap_server.run_handle()).await;
    // api_server.await?;
    Ok(())
}


/// 初始化hap 设备 init_iot_device
async fn init_iot_device(conn: &DatabaseConnection) -> anyhow::Result<DeviceMap> {
    //读取设备
    let devices = IotDevice::find()
        .filter(IotDeviceColumn::GatewayId.is_null().and(
            IotDeviceColumn::Disabled.eq(false)
        ))
        .all(conn).await?;
    let mut map: DeviceMap = dashmap::DashMap::new();
    let mut gateway_map = dashmap::DashMap::new();
    for dev in devices.into_iter() {
        let dev_id = dev.device_id;
        let dev: DevicePointer = match dev.device_type {
            IotDeviceType::MiWifiDevice => {
                if let Some(DeviceParam::WifiDeviceParam(param)) = dev.params {
                    match WifiDevice::new(param).await {
                        Ok(dev) => Arc::new(dev),
                        Err(err) => {
                            error!("初始化设备失败，{}", err);
                            continue;
                        }
                    }
                } else {
                    error!("初始化设备失败，参数类型错误");
                    continue;
                }
            }
            IotDeviceType::MiGatewayDevice => {
                if let Some(DeviceParam::MiGatewayParam(param)) = dev.params {
                    match OpenMiioGatewayDevice::new(param).await {
                        Ok(dev) => {
                            //as Box<dyn MiotSpecDevice>
                            let dev = Arc::new(dev);
                            gateway_map.insert(dev_id, dev.clone());
                            dev
                        }
                        Err(err) => {
                            error!("初始化设备失败，{}", err);
                            continue;
                        }
                    }
                } else {
                    error!("初始化设备失败，参数类型错误");
                    continue;
                }
            }
            _ => {
                error!("初始化设备失败，设备类型错误");
                continue;
            }
        };
        map.insert(dev_id, dev);
        //list.push(dev);
    }
    // 处理需要网关的设备
    let children_devices = IotDevice::find()
        .filter(IotDeviceColumn::GatewayId.is_not_null().and(
            IotDeviceColumn::Disabled.eq(false)
        ))
        .all(conn).await?;

    for dev in children_devices.into_iter() {
        match dev.device_type {
            IotDeviceType::BleDevice => {
                let gateway = gateway_map.get(dev.gateway_id.as_ref().unwrap());
                if gateway.is_none() {
                    error!("初始化设备失败，网关不存在");
                    continue;
                }
                let gateway = gateway.unwrap();
                if let Some(DeviceParam::BleParam(param)) = dev.params {
                    let ble_dev = BleDevice::new(param, gateway.clone());
                    map.insert(dev.device_id, Arc::new(ble_dev));
                }
            }
            _ => {}
        }
    }


    Ok(map)
}

#[tokio::test]
pub async fn test2() -> anyhow::Result<()> {
    let conn = db_conn().await;
    SwitchService::new(1, 1);
    let a = HapServiceActiveModel {
        id: Set(4),
        accessory_id: Set(2),
        name: Default::default(),
        hap_type: Set(MappingHapType::Switch),
        disabled: Set(false),
        mapping_method: Set(MappingMethod::WifiToContr),
        param: Set(Some(MappingParam {
            characteristic: vec![
                MappingCharacteristic {
                    characteristic_type: MappingHapType::PowerState,
                    fixed_value: None,
                    property: Some(Property {
                        siid: 2,
                        piid: 1,
                        min_value: None,
                        max_value: None,
                        step: None,
                    }),
                }
            ],
            power_state: None,
            brightness: None,
            color_temperature: None,
            current_temperature: None,
        })),
    };

    let b = HapServiceEntity::insert(a).exec(&conn).await.unwrap();
    Ok(())
}

#[tokio::test]
pub async fn test1() -> anyhow::Result<()> {
    let conn = db_conn().await;
    /* let a = IotDeviceActiveModel {
         device_id: Set(3),
         device_type: Set(IotDeviceType::MiWifiDevice),
         params: Set(Some(DeviceParam::WifiDeviceParam(
             DeviceInfo {
                 did: "553822207".to_string(),
                 token: "6b95c9eda4e89e7006908f3757b655b7".to_string(),
                 model: "yeelink.light.lamp22".to_string(),
                 firmware_revision: None,
                 software_revision: None,
                 name: "米家智能显示器挂灯1s".to_string(),
                 mac: Some("68:ab:bc:73:05:11".to_string()),
                 serial_number: None,
                 manufacturer: None,
                 localip: Some("192.168.68.25".to_string()),
             }
         ))),
         gateway_id: Default::default(),
         name: Set(Some("test".to_string())),
         memo: Default::default(),
     };*/

    /*  let a = IotDeviceActiveModel {
          device_id: Set(4),
          device_type: Set(IotDeviceType::MiGatewayDevice),
          params: Set(Some(DeviceParam::MiGatewayParam(
              DeviceInfo {
                  did: "553822207".to_string(),
                  token: "6b95c9eda4e89e7006908f3757b655b7".to_string(),
                  model: "yeelink.light.lamp22".to_string(),
                  firmware_revision: None,
                  software_revision: None,
                  name: "网关".to_string(),
                  mac: Some("".to_string()),
                  serial_number: None,
                  manufacturer: None,
                  localip: Some("192.168.68.24".to_string()),
              }
          ))),
          gateway_id: Default::default(),
          name: Set(Some("test".to_string())),
          memo: Default::default(),
      };*/
    let ble = DeviceInfo {
        did: "blt.3.1g8f9gmps4o02".to_string(),
        token: "".to_string(),
        model: "miaomiaoce.sensor_ht.t1".to_string(),
        firmware_revision: None,
        software_revision: None,
        name: "温湿度传感器".to_string(),
        mac: Some("".to_string()),
        serial_number: None,
        manufacturer: None,
        localip: None,
    };
    let a = IotDeviceActiveModel {
        device_id: Set(4),
        device_type: Set(IotDeviceType::MiGatewayDevice),
        params: Set(Some(DeviceParam::BleParam(ble))),
        gateway_id: Set(Some(3)),
        name: Set(Some("test".to_string())),
        memo: Default::default(),
    };
    let b = IotDevice::insert(a).exec(&conn).await.unwrap();

    let devices = IotDevice::find()
        .filter(IotDeviceColumn::GatewayId.is_null())
        .all(&conn).await?;
    println!("{:?}", devices);
    Ok(())
}