use anyhow::anyhow;
use futures_util::FutureExt;
use hap::characteristic::{AsyncCharacteristicCallbacks, Characteristic, Format, HapCharacteristic, OnReadFuture, OnUpdateFuture, Perm};
use log::{debug, error, info};
use serde_json::Value;
use tap::{Tap, TapFallible, TapOptional};
use miot_spec::device::miot_spec_device::MiotDeviceType;
use miot_spec::device::value::ListenDateType;
use miot_spec::device::value::ListenDateType::BleData;
use miot_spec::proto::miio_proto::MiotSpecDTO;
use crate::convertor::iot_characteristic::{CharacteristicValue, IotCharacteristic};
use crate::convertor::miot2hap::Utils;
use crate::convertor::unit_convertor::UnitConvertor;
use crate::db::entity::hap_characteristic::{MappingMethod, MappingParam, Model, Property};
use crate::db::entity::prelude::HapCharacteristicModel;
use crate::init::{DevicePointer, HapAccessoryPointer};

pub async fn to_characteristic(sid: u64, aid: u64, index: usize, ch: HapCharacteristicModel,
                               device: DevicePointer, accessory: HapAccessoryPointer) -> anyhow::Result<IotCharacteristic> {
    let mut cts = IotCharacteristic(Characteristic::<CharacteristicValue> {
        id: sid + index as u64 + 1,
        accessory_id: aid,
        hap_type: ch.characteristic_type.into(),
        // hap_type: HapType::PowerState,
        format: Format::default(),
        max_value: ch.max_value.clone().map(|i| CharacteristicValue::new(i)),
        min_value: ch.min_value.clone().map(|i| CharacteristicValue::new(i)),
        perms: vec![
            Perm::Events,
            Perm::PairedRead,
            Perm::PairedWrite,
        ],
        ..Default::default()
    });
    // 设置默认值
    set_default_for_cts(&mut cts, ch.clone())?;
    // 这是属性方法
    set_read_update_method(sid, &mut cts, ch, device, accessory).await?;


    Ok(cts)
}

async fn set_read_update_method(sid: u64, cts: &mut IotCharacteristic, ch: HapCharacteristicModel, device: DevicePointer, accessory: HapAccessoryPointer) -> anyhow::Result<()> {
    let cid = cts.get_id();

    match ch.mapping_method {
        MappingMethod::Unknown => {
            return Err(anyhow!("不支持的映射方法:{:?}",ch.mapping_method));
        }
        MappingMethod::MIotSpec => {
            // 设置读写映射
            if let Some(MappingParam::MIotSpec(param)) = ch.mapping_param {
                let ps = param.property;
                let conv = ch.unit_convertor;
                let read = ToChUtils::get_read_func(device.clone(), ps.clone(), conv.clone());
                cts.on_read_async(Some(read));
                let set = ToChUtils::get_set_func(device.clone(), ps.clone(), conv.clone());
                cts.on_update_async(Some(set));
                //当前特征值设置
                if let Ok(Some(v)) = ToChUtils::read_property(ps.clone(), device.clone()).await {
                    cts.0.value = CharacteristicValue::new(v)
                } else {
                    cts.0.value = match cts.0.min_value.as_ref() {
                        None => {
                            CharacteristicValue::default()
                        }
                        Some(min) => {
                            min.clone()
                        }
                    }
                }
            } else {
                return Err(anyhow!("映射参数不能为空"));
            }
        }
        MappingMethod::BleToSensor => {
            //蓝牙设备转传感器
            // 只有读取,无更新函数
            if let Some(MappingParam::BleToSensor(param)) = ch.mapping_param {
                if let MiotDeviceType::Ble(_) = device.get_device_type() {
                    let dev_c = device.clone();
                    let conv_c = ch.unit_convertor.clone();
                    let conv_param = ch.convertor_param.clone();
                    let param_c = param.clone();
                    cts.on_read_async(Some(move || {
                        let dev_c = dev_c.clone();
                        let conv = conv_c.clone();
                        let conv_param = conv_param.clone();
                        let param_cc = param_c.clone();
                        async move {
                            if let MiotDeviceType::Ble(dev) = dev_c.get_device_type() {
                                return Ok(dev.values.read()
                                    .await.get_value(param_cc.ble_value_type.into())
                                    .map(|v| {
                                        //单位转换器去转换
                                        let v = match conv {
                                            None => v,
                                            Some(conv) => {
                                                conv.get_convertor().to(conv_param, v)
                                                    .tap_err(|v| {
                                                        error!("单位转换错误:{:?}",v);
                                                    })
                                                    .unwrap_or(Value::Null)
                                            }
                                        };
                                        CharacteristicValue::new(v)
                                    }).tap(|v| {
                                    debug!("读取蓝牙设备值:{:?}",v);
                                }));
                            }
                            Ok(None)
                        }.boxed()
                    }));

                    let accessory_c = accessory.clone();
                    let conv_param = ch.convertor_param.clone();
                    let param_c = param.clone();
                    // 捕获设备设置值
                    let func = move |data: ListenDateType| {
                        let accessory_c = accessory_c.clone();
                        let param_cc = param_c.clone();
                        let conv_param = conv_param.clone();
                        async move {
                            let v = match data {
                                BleData(v) => v,
                                _ => {
                                    return Err(anyhow::anyhow!("数据类型错误"));
                                }
                            };
                            let value = v.get_value(param_cc.ble_value_type.into())
                                .map(|v| {
                                    //单位转换器去转换
                                    let v = match conv_c {
                                        None => {
                                            v
                                        }
                                        Some(conv) => {
                                            conv.get_convertor().to(conv_param.clone(), v).unwrap_or(Value::Null)
                                        }
                                    };
                                    v
                                }).tap_some(|v| {
                                debug!("读取设备特征值:{:?}",v);
                            });


                            // let value = v.temperature.map(|i| serde_json::Value::from(i.value as f32 / 10.0));
                            if let Some(value) = value {
                                // 蓝牙数据设置到特征上
                                match accessory_c.lock()
                                    .await
                                    .get_id_mut_service(sid)
                                    .and_then(|s| s.get_id_mut_characteristic(cid)) {
                                    None => {
                                        return Err(anyhow::anyhow!("特征不存在"));
                                    }
                                    Some(cts) => {
                                        cts.set_value(value).await?;
                                    }
                                }
                            };
                            Ok(())
                        }.boxed()
                    };
                    device.add_listener(Box::new(func)).await;
                    // let a = device.accessories.remove(0);
                    //hs.get_id(); HapType::TemperatureSensor,
                    //add_listener,

                    // info!("当前温度:{:?}", a);


                    //设置默认值
                    cts.0.value = match cts.0.min_value.as_ref() {
                        None => {
                            CharacteristicValue::default()
                        }
                        Some(min) => {
                            min.clone()
                        }
                    }
                } else {
                    return Err(anyhow!("设备类型不是蓝牙设备"));
                }
            } else {
                return Err(anyhow!("映射参数不能为空"));
            }
        }
    }

    Ok(())
}

fn set_default_for_cts(cts: &mut IotCharacteristic, ch: HapCharacteristicModel) -> anyhow::Result<()> {
    let default = IotCharacteristic::new_default(ch.characteristic_type.clone());
    // todo 格式设置
    /* match ch.format {
          None => {
          }
          Some(f) => {
              // cts.0.format = f.into();
          }
      }*/


    if ch.format.is_none() && default.is_none() {
        return Err(anyhow!("服务:{},特征:{},格式不能为空", ch.service_id, ch.cid));
    };

    if let Some(default) = default {
        if ch.format.is_none() {
            cts.0.format = default.get_format();
        };
        if cts.0.unit.is_none() {
            cts.0.unit = default.get_unit();
        };
        if cts.0.max_len.is_none() {
            cts.0.max_len = default.get_max_len();
        };
        if cts.0.max_value.is_none() {
            cts.0.max_value = default.get_max_value().map(|i| CharacteristicValue::new(serde_json::Value::from(i)));
        }
        if cts.0.min_value.is_none() {
            cts.0.min_value = default.get_min_value().map(|i| CharacteristicValue::new(serde_json::Value::from(i)));
        }
        if cts.0.step_value.is_none() {
            cts.0.step_value = default.get_step_value().map(|i| CharacteristicValue::new(serde_json::Value::from(i)));
        }
        cts.0.perms = default.get_perms();
    }

    Ok(())
}


pub struct ToChUtils {}

impl ToChUtils {
    pub async fn read_property(property: Property, dev: DevicePointer) -> anyhow::Result<Option<Value>> {
        dev.get_proto().await.map_err(|e| anyhow::anyhow!("err"))?
            .get_property(MiotSpecDTO {
                did: dev.get_info().did.clone(),
                siid: property.siid,
                piid: property.piid,
                value: None,
            }).await
    }

    pub fn get_set_func(device: DevicePointer, property_id: Property, conv: Option<UnitConvertor>) -> impl OnUpdateFuture<CharacteristicValue>
    {
        move |old: CharacteristicValue, new: CharacteristicValue| {
            let dev = device.clone();
            let id = dev.get_info().did.clone();
            async move {
                if old == new {
                    return Ok(());
                };
                let value = match conv {
                    None => {
                        Some(new.value)
                    }
                    Some(conv) => {
                        let new_val = new.value.clone();
                        let value = conv.get_convertor().to(None, new.value)?;
                        debug!("set convert value:{:?}=>{:?}", new_val, value);
                        Some(value)
                    }
                };
                if value.is_none() {
                    return Ok(());
                };


                //读取状态
                info!("set value:{},{},{:?}",property_id.siid,property_id.piid,value);
                let proto = dev.get_proto().await.map_err(|e| { anyhow::anyhow!("获取连接协议失败") })?;
                let res = proto.set_property(MiotSpecDTO {
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

    /// 获取wifi 设备读取函数
    pub fn get_read_func(device: DevicePointer,
                         property: Property,
                         convertor: Option<UnitConvertor>) -> impl OnReadFuture<CharacteristicValue>
    {
        move || {
            let dev = device.clone();
            let id = dev.get_info().did.clone();
            let conv = convertor;

            async move {
                //读取状态
                let proto = dev.get_proto()
                    .await
                    .map_err(|e| { anyhow::anyhow!("获取连接协议失败") })?;
                let mut value = proto.get_property(MiotSpecDTO {
                    did: id,
                    siid: property.siid,
                    piid: property.piid,
                    value: None,
                }).await.tap_err(
                    |e| {
                        error!("属性读取失败:{}", e);
                    }
                )?;
                // info!("aa read value:{},{},{:?}",property.siid,property.piid,value);
                Ok(value.map(|f| {
                    CharacteristicValue::new(match conv {
                        None => {
                            f
                        }
                        Some(conv) => {
                            conv.get_convertor()
                                .from(None, f)
                                .unwrap_or(Value::Null)
                        }
                    })
                }).tap(|f| {
                    info!("read value:{},{},{:?}",property.siid,property.piid,f);
                }))
            }.boxed()
        }
    }
}