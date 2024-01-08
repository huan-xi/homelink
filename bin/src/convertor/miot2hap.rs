use std::cmp::{max, min};
use std::collections::HashMap;
use std::fmt::{Debug, format};
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::time::Duration;
use hap::accessory::{AccessoryCategory, AccessoryInformation, HapAccessory};
use hap::accessory::contact_sensor::ContactSensorAccessory;
use hap::accessory::lightbulb::LightbulbAccessory;
use hap::accessory::outlet::OutletAccessory;
use hap::accessory::switch::SwitchAccessory;
use hap::accessory::temperature_sensor::TemperatureSensorAccessory;
use hap::characteristic::{AsyncCharacteristicCallbacks, CharacteristicCallbacks, HapCharacteristic, HapCharacteristicSetup, OnReadFuture, OnUpdateFuture, Unit};
use hap::characteristic::name::NameCharacteristic;
use hap::futures::future::BoxFuture;
use hap::futures::FutureExt;
use hap::service::switch::SwitchService;
use log::{debug, error, info};
use serde_json::Value;
use tap::Tap;
use miot_spec::proto::miio_proto::{MiIOProtocol, MiotSpecDTO};
use tap::tap::TapFallible;
use miot_spec::device::miot_spec_device::DeviceInfo;
use crate::convertor::config::{MappingConfig, Miot2HapMapper};
use crate::convertor::iot_characteristic::CharacteristicValue;
use crate::convertor::unit_convertor::UnitConvertor;
use crate::db::entity::hap_characteristic::Property;
use crate::db::entity::prelude::HapServiceModel;
use crate::init::DevicePointer;


/*impl IntoHapAccessory for dyn MiotSpecDevice {
    /// model=>{}
    fn mapping_to_hap_accessories<'a>(&'a self, param: &'a IntoHapParam<'a>) -> BoxFuture<Vec<anyhow::Result<Box<dyn HapAccessory>>>> {
        async move {
            let info = self.get_info();
            let model_str = info.model.as_str();

            let mut list = vec![];
            let id = param.id.fetch_add(1, Ordering::SeqCst);
            let configs = param.model_mapper.get(model_str).expect(format!("不支持该模型:{}", model_str).as_str());
            for mapper in configs.mappers.iter() {
                let config = mapper.clone();
                let device = match config.hap_type {
                    MappingHapType::Lightbulb => {
                        LightbulbMapper::map_to_accessory(self, id, config)
                            .map(|e| Box::new(e) as Box<dyn HapAccessory>)
                            .await
                    }

                    MappingHapType::Switch => {
                        todo!();
                        /*SwitchMapper::map_to_accessory(self, id, config)
                            .map(|e| Box::new(e) as Box<dyn HapAccessory>)*/
                    }
                    MappingHapType::Outlet => {
                        let info = self.get_info().clone();
                        let mut accessory = OutletAccessory::new(
                            id,
                            AccessoryInformation {
                                name: info.name.clone(),
                                ..Default::default()
                            },
                        ).unwrap();
                        let proto = self.get_proto().await?;

                        // 电源控制属性
                        match config.power_state {
                            None => {
                                todo!("不支持无电源灯")
                            }
                            Some(ps) => {
                                // let ptc = proto.clone();
                                let id = info.did.clone();
                                let func = Utils::get_set_func(id.clone(), proto.clone(), ps.clone());
                                accessory.outlet.power_state.on_update_async(Some(func));
                                let func = Utils::get_read_func(id.clone(), proto.clone(), ps.clone(), |v| v.as_bool());
                                accessory.outlet.power_state.on_read_async(Some(func));
                            }
                        }
                        //Ok(Box::new(accessory) as Box<dyn HapAccessory>)
                    }

                    MappingHapType::ContactSensor => {
                        todo!()
                        /*  let id = param.id.fetch_add(1, Ordering::SeqCst);
                          let mut accessory = ContactSensorAccessory::new(id, AccessoryInformation {
                              name: info.name.clone(),
                              ..Default::default()
                          }).unwrap();
                          // accessory.contact_sensor.status_active;
                          list.push(Box::new(accessory) as Box<dyn HapAccessory>);*/
                    }
                    MappingHapType::TemperatureSensor => {
                        todo!();
                        /*  let id = param.id.fetch_add(1, Ordering::SeqCst);
                          let mut accessory = TemperatureSensorAccessory::new(id, AccessoryInformation {
                              name: info.name.clone(),
                              model: "test".to_string(),
                              ..Default::default()
                          }).unwrap();
                          accessory.temperature_sensor.status_active.as_mut().unwrap().on_read(
                              Some(|| {
                                  Ok(Some(true))
                              })
                          );
                          let proto = self.get_proto();
                          let id = info.did.clone();
                          let b = &mut accessory.temperature_sensor.current_temperature;
                          // accessory.temperature_sensor.current_temperature.set_event_emitter()
                          tokio::spawn(
                              async move {
                                  let mut a = 40;

                                  loop {
                                      a = a + 1;
                                      b.set_value(Value::from(a + 1)).await;
                                      tokio::time::sleep(Duration::from_secs(2)).await;
                                  }
                              }
                          );

                          accessory.temperature_sensor.current_temperature
                              .on_read_async(Some(|| {
                                  async move {
                                      info!("on read");
                                      Ok(Some(32f32))
                                  }.boxed()
                              }));
                          /*    let ps = config.status_tampered.unwrap();
                              let func = Utils::get_read_func(id.clone(), proto.clone(), ps.clone(), |f| f.as_u64().map(|v| v as u8));

                              accessory.temperature_sensor.status_tampered
                                  .as_mut()
                                  .unwrap()
                                  .on_read_async(Some(func));*/

                          // accessory.contact_sensor.status_active;
                          list.push(Box::new(accessory) as Box<dyn HapAccessory>);*/
                    }
                    _ => {
                        todo!("未实现的类型")
                    }
                };
                list.push(device)
            }


            list
        }.boxed()
    }
}
*/


pub struct ServiceSetter {}

impl ServiceSetter {
    /// 设置服务名称
    pub(crate) async fn set_service_name(service: &HapServiceModel, name_ch: &mut Option<NameCharacteristic>) -> anyhow::Result<()> {
        if let Some(name) = service.name.as_ref() {
            let name = serde_json::Value::String(name.clone());
            name_ch.as_mut().unwrap().set_value(name).await?;
        } else {
            *name_ch = None;
        }
        Ok(())
    }

    pub fn set_power_state<T>(hs: &mut T, info: &DeviceInfo, proto: Arc<MiIOProtocol>, ps: &Property)
        where
            T: AsyncCharacteristicCallbacks<bool> + 'static,
    {
        let func = Utils::get_set_func_1(info.did.clone(), proto.to_owned(), ps.to_owned());
        hs.on_update_async(Some(func));

        let func = Utils::get_read_func_1(info.did.clone(), proto.to_owned(), ps.to_owned(), |v| v.as_bool());
        hs.on_read_async(Some(func));
    }
}

pub struct Utils {}

impl Utils {


    pub fn get_read_func_1<T>(did: String,
                              prop: Arc<MiIOProtocol>,
                              property: Property,
                              conv: fn(Value) -> Option<T>) -> impl OnReadFuture<T>
        where T: std::default::Default + std::clone::Clone + serde::Serialize + std::marker::Send + std::marker::Sync + 'static {
        move || {
            let ptc = prop.clone();
            let id = did.clone();
            async move {
                //读取状态
                let value = ptc.get_property(MiotSpecDTO {
                    did: id,
                    siid: property.siid,
                    piid: property.piid,
                    value: None,
                }).await
                    .tap_err(
                        |e| {
                            error!("属性读取失败:{}", e);
                        }
                    )?;
                info!("read value:{},{},{:?}",property.siid,property.piid,value);

                Ok(value.and_then(conv))
            }.boxed()
        }
    }
    // new.into()

    /// did 设备id
    pub fn get_set_func_1<T>(did: String, prop: Arc<MiIOProtocol>, property_id: Property) -> impl OnUpdateFuture<T>
        where
            T: Into<Value> + Clone + Send + Debug + Sync + 'static + std::default::Default + serde::ser::Serialize, {
        Self::get_set_func_conv(did, prop, property_id, |i: T| i.into())
    }

    /// PropertyId 转set func
    /// 获取设置属性的函数
    pub fn get_set_func_conv<T>(did: String, prop: Arc<MiIOProtocol>, property_id: Property, conv: fn(T) -> Value) -> impl OnUpdateFuture<T>
        where
            T: Into<Value> + Clone + Send + Debug + Sync + 'static + std::default::Default + serde::ser::Serialize, {
        move |old: T, new: T| {
            let ptc = prop.clone();
            let id = did.clone();
            async move {
                let value = Some(conv(new));
                //读取状态
                info!("set value:{},{},{:?}",property_id.siid,property_id.piid,value);
                let res = ptc.set_property(MiotSpecDTO {
                    did: id,
                    siid: property_id.siid,
                    piid: property_id.piid,
                    value,
                }).await.tap_err(|e| {
                    error!("设置属性失败:{}", e);
                });
                Ok(())
            }.boxed()
        }
    }
}