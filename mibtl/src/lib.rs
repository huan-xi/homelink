pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use std::env;
    use btleplug::api::{bleuuid::BleUuid, Central, CentralEvent, Manager as _, Peripheral, ScanFilter};
    use btleplug::platform::{Adapter, Manager};
    use futures_util::stream::StreamExt;
    use std::error::Error;
    use log::{error, info};

    async fn get_central(manager: &Manager) -> Adapter {
        let adapters = manager.adapters().await.unwrap();
        adapters.into_iter().nth(0).unwrap()
    }

    #[tokio::test]
    async fn it_works() -> Result<(), Box<dyn Error>> {
        env::set_var("RUST_LOG", "debug");
        pretty_env_logger::init();

        let manager = Manager::new().await?;
        let central = get_central(&manager).await;

        // Each adapter has an event stream, we fetch via events(),
        // simplifying the type, this will return what is essentially a
        // Future<Result<Stream<Item=CentralEvent>>>.
        let mut events = central.events().await?;

        // start scanning for devices
        central.start_scan(ScanFilter::default()).await?;
        tokio::time::sleep(std::time::Duration::from_secs(10)).await;

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
        /*        while let Some(event) = events.next().await {
                    match event {
                        CentralEvent::DeviceDiscovered(id) => {
                            // info!("DeviceDiscovered: {:?}", id);
                        }
                        CentralEvent::DeviceConnected(id) => {
                            // info!("DeviceConnected: {:?}", id);
                        }
                        CentralEvent::DeviceDisconnected(id) => {
                            // info!("DeviceDisconnected: {:?}", id);
                        }
                        CentralEvent::ManufacturerDataAdvertisement {
                            id,
                            manufacturer_data,
                        } => {
                            info!(
                                "ManufacturerDataAdvertisement: {:?}, {:?}",
                                id, manufacturer_data
                            );
                        }
                        CentralEvent::ServiceDataAdvertisement { id, service_data } => {
                            let a = Reading::decode(&service_data);
                            if let Some(a) = a {
                                error!("ServiceDataAdvertisement:  {:?}", a);
                            };


                            // println!("ServiceDataAdvertisement: {:?}, {:?}", id, service_data);
                        }
                        CentralEvent::ServicesAdvertisement { id, services } => {
                            let services: Vec<String> =
                                services.into_iter().map(|s| s.to_short_string()).collect();
                            println!("ServicesAdvertisement: {:?}, {:?}", id, services);
                        }
                        _ => {}
                    }
                }*/
        Ok(())
    }
}
