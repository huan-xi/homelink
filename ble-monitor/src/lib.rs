use crate::error::BleError;

pub mod parse_advertisement;
pub mod parser;
pub mod error;


pub type BltResult<T> = Result<T, BleError>;

#[derive(Debug,Clone)]
pub enum BleValue {
    U8(u8),
    I16(i16),
}

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use std::env;
    use btleplug::api::{Central, CentralEvent, Manager as _, Peripheral, ScanFilter};
    use btleplug::platform::{Adapter, Manager, PeripheralId};

    use std::error::Error;
    use std::str::FromStr;
    use btsensor::{atc, Reading};
    use futures_util::{StreamExt, TryFutureExt};
    use log::{error, info};
    use crate::parse_advertisement::parse_advertisement;

    async fn get_central(manager: &Manager) -> Adapter {
        let adapters = manager.adapters().await.unwrap();
        adapters.into_iter().next().unwrap()
    }

    #[tokio::test]
    async fn it_works() -> Result<(), Box<dyn Error>> {
        env::set_var("RUST_LOG", "debug");
        pretty_env_logger::init();
        let uuid_test = uuid::Uuid::from_str("0000fe95-0000-1000-8000-00805f9b34fb")?;

        let manager = Manager::new().await?;
        let central = get_central(&manager).await;
        let mut events = central.events().await?;

        // start scanning for devices
        central.start_scan(ScanFilter::default()).await?;
        // tokio::time::sleep(std::time::Duration::from_secs(10)).await;

        // 温湿度传感器
        //LYWSD02

        let a = central.peripherals().await?;

        for x in a {
            let p = x.properties().await?.unwrap();
            if p.manufacturer_data.get(&76).is_some() {
                continue;
            };

            let name = p;
            info!("addr:{:?}",name);
        }
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
                        info!("DeviceDiscovered: {:?}", id);
                    }
                }
                CentralEvent::DeviceConnected(id) => {
                    info!("DeviceConnected: {:?}", id);
                }
                CentralEvent::DeviceDisconnected(id) => {
                    // info!("DeviceDisconnected: {:?}", id);
                }
                CentralEvent::ManufacturerDataAdvertisement {
                    id,
                    manufacturer_data,
                } => {
                    for x in manufacturer_data {
                        // info!("ManufacturerDataAdvertisement: 0x{:x}", x.0);
                    }
                    //info!("ManufacturerDataAdvertisement: {:?}, {:?}",id, manufacturer_data);
                }
                //服务数据广播
                CentralEvent::ServiceDataAdvertisement { id, service_data } => {
                    for (uuid, bytes) in &service_data {
                        let a = hex::encode(bytes);
                        info!("ServiceDataAdvertisement: {:?},bytes: {:?}", uuid, a);
                        // let id = PeripheralId(uuid.clone());
                        let p = central.peripheral(&id).await;
                        if let Ok(o) = p {
                            // o.properties().await
                            info!("name:{:?}",o);
                        }
                        parse_advertisement(uuid, bytes.as_slice());
                    }
                    if let Some(data) = service_data.get(&atc::UUID) {
                        info!("test");
                    }
                    /*    let a = Reading::decode(&service_data);
                        if let Some(a) = a {
                            error!("ServiceDataAdvertisement:  {:?}", a);
                        };*/
                    // println!("ServiceDataAdvertisement: {:?}, {:?}", id, service_data);
                }
                CentralEvent::ServicesAdvertisement { id, services } => {
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
