use crate::error::BleError;

pub mod parse_advertisement;
pub mod parser;
pub mod error;


pub type BltResult<T> = Result<T, BleError>;


pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use std::env;
    use btleplug::api::{Central, CentralEvent, Manager as _, ScanFilter};
    use btleplug::platform::{Adapter, Manager};

    use std::error::Error;
    use std::str::FromStr;
    
    
    use futures_util::{StreamExt};
    use log::{error, info};
    
    
    use crate::error::BleError;
    use crate::parse_advertisement::parse_advertisement;

    async fn get_central(manager: &Manager) -> Adapter {
        let adapters = manager.adapters().await.unwrap();
        adapters.into_iter().next().unwrap()
    }

    #[tokio::test]
    async fn it_works() -> Result<(), Box<dyn Error>> {
        env::set_var("RUST_LOG", "debug");
        pretty_env_logger::init();
        info!("start");

        let uuid_test = uuid::Uuid::from_str("0000fe95-0000-1000-8000-00805f9b34fb")?;

        let manager = Manager::new().await?;
        let central = get_central(&manager).await;
        let mut events = central.events().await?;

        // start scanning for devices
        central.start_scan(ScanFilter::default()).await?;
        // tokio::time::sleep(std::time::Duration::from_secs(10)).await;

        // 温湿度传感器
        //LYWSD02
        /*        let mut a = time::interval(Duration::from_secs(1));
                for i in 0..100 {
                    a.tick().await;
                    let a = central.peripherals().await?;
                    info!("peripherals:{:?}", a.len());
                    for x in a {
                        let addr = x.address();
                        let opts = x.properties()
                            .await
                            .unwrap();

                        info!("addr:{:?},name:{:?}",addr,opts);
                    }
                }*/


        tokio::spawn(async {});

        // info!("{:?}", a);

        // central.stop_scan()
        // Print based on whatever the event receiver outputs. Note that the event
        // receiver blocks, so in a real program, this should be run in its own
        // thread (not task, as this library does not yet use async channels).
        while let Some(event) = events.next().await {
            match event {
                CentralEvent::DeviceDiscovered(id) => {
                    let id = format!("{}", id);
                    if uuid_test.to_string() == id {
                        //info!("DeviceDiscovered: {:?}", id);
                    }
                }
                CentralEvent::DeviceConnected(_id) => {
                    //info!("DeviceConnected: {:?}", id);
                }
                CentralEvent::DeviceDisconnected(_id) => {
                    // info!("DeviceDisconnected: {:?}", id);
                }
                CentralEvent::ManufacturerDataAdvertisement {
                    id: _,
                    manufacturer_data,
                } => {
                    for _x in manufacturer_data {
                        // info!("ManufacturerDataAdvertisement: 0x{:x}", x.0);
                    }
                    //info!("ManufacturerDataAdvertisement: {:?}, {:?}",id, manufacturer_data);
                }
                //服务数据广播
                CentralEvent::ServiceDataAdvertisement { id: _, service_data } => {
                    for (uuid, bytes) in &service_data {
                        let bytes_hex = hex::encode(bytes);
                        if bytes_hex.contains("22b961bd7517") {
                            break;
                        }
                        //mesh忽略掉 b054990700e2f9e14aa374080e00
                        if bytes_hex.contains("e2f9e14aa374") {
                            break;
                        }
                        //不支持设备 552b0515000511011a0510a305
                        if bytes_hex.contains("0511011a0510") {
                            break;
                        }
                        //0x3139  0541393152368106ac8226e6e03e
                        if bytes_hex.contains("368106ac8226") {
                            break;
                        }
                        if bytes_hex.contains("3a1171b340b3") {
                            break;
                        }
                        // 人体传感器 48598d0a167ce65871029546c00f00b9fa47b7
                        if bytes_hex.contains("7ce658710295") {
                            break;
                        }



                        // let id = PeripheralId(uuid.clone());
                        let a = parse_advertisement(uuid, bytes.as_slice());
                        match a {
                            Ok(_e) => {
                                info!("success:{}", uuid);
                            }
                            Err(e) => {
                                if let BleError::NotSupportedDeviceType(e) = e {
                                    info!("ServiceDataAdvertisement: 0x{:x}", e);
                                }
                                {
                                    error!("ServiceDataAdvertisement :err:{e}: {:?},bytes: {:?}", uuid, bytes_hex);
                                };


                                // info!("ServiceDataAdvertisement: {}", e);
                            }
                        }
                    }
                    // if let Some(data) = service_data.get(&atc::UUID) {
                    //     info!("test");
                    // }
                    /*    let a = Reading::decode(&service_data);
                        if let Some(a) = a {
                            error!("ServiceDataAdvertisement:  {:?}", a);
                        };*/
                    // println!("ServiceDataAdvertisement: {:?}, {:?}", id, service_data);
                }
                CentralEvent::ServicesAdvertisement { id: _, services: _ } => {
                    /*let services: Vec<String> = services.into_iter().map(|s| s.to_short_string()).collect();
                    //0xfe3c alibaba
                    // 小米 "0xfe3c""0xfe3c"
                    // 小米（0xFDAB，0xFDAA，0xFE95）
                    info!("ServicesAdvertisement: {:?}, {:?}", id, services);
                    let id = format!("{}", id);
                    if uuid_test.to_string() == id {
                        info!("ServicesAdvertisement: {:?}", id);
                    }*/
                }
                _ => {}
            }
        }
        Ok(())
    }
}
