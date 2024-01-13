use anyhow::anyhow;
use futures_util::FutureExt;
use hap::characteristic::{AsyncCharacteristicCallbacks, Characteristic, Format, HapCharacteristic, OnReadFuture, OnUpdateFuture, Perm, Unit};
use hap::HapType;
use log::{debug, error, info};
use sea_orm::JsonValue;
use serde_json::Value;
use tap::{Tap, TapFallible, TapOptional};
use miot_spec::device::emitter::{DataListener, EventType};
use miot_spec::device::emitter::EventType::{SetProperty};
use crate::hap::iot_characteristic::{CharacteristicValue, IotCharacteristic};
use crate::hap::unit_convertor::{ConvertorParamType, UnitConv, UnitConvertor};
use crate::db::entity::hap_characteristic::{MappingMethod, MappingParam, Model, Property};
use crate::db::entity::prelude::HapCharacteristicModel;
use crate::init::{DevicePointer, HapAccessoryPointer};


pub async fn to_characteristic(sid: u64, aid: u64, index: usize, ch: HapCharacteristicModel,
                               device: DevicePointer, accessory: HapAccessoryPointer) -> anyhow::Result<IotCharacteristic> {
    let format: Format = serde_json::from_str(format!("\"{}\"", ch.format).as_str())
        .map_err(|e| anyhow!("格式转换错误:{:?}", e))?;
    let unit: Option<Unit> = match ch.unit.as_ref() {
        None => { None }
        Some(i) => {
            Some(serde_json::from_str(format!("\"{}\"", i).as_str())
                .map_err(|e| anyhow!("格式转换错误:{:?}", e))?)
        }
    };


    let mut cts = IotCharacteristic(Characteristic::<CharacteristicValue> {
        id: sid + index as u64 + 1,
        accessory_id: aid,
        hap_type: ch.characteristic_type.into(),
        // hap_type: HapType::PowerState,
        format,
        unit,
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
    //当前特征值设置
    cts.0.value = match cts.0.min_value.as_ref() {
        None => {
            CharacteristicValue::default()
        }
        Some(min) => {
            min.clone()
        }
    };

    Ok(cts)
}

/// 设置特征的读写方法
async fn set_read_update_method(sid: u64, cts: &mut IotCharacteristic, ch: HapCharacteristicModel, device: DevicePointer, accessory: HapAccessoryPointer) -> anyhow::Result<()> {
    // let cid = cts.get_id();
    let hap_type = cts.get_type();
    let unit_conv = UnitConv(ch.unit_convertor.clone(), ch.convertor_param.clone());
    match ch.mapping_method {
        MappingMethod::None => {
            //不映射属性
        }
        MappingMethod::PropMapping => {
            // 设置读写映射
            if let Some(MappingParam::MIotSpec(param)) = ch.mapping_param {
                let ps = param.property;
                let read = ToChUtils::get_read_func(device.clone(), ps.clone(), unit_conv.clone());
                cts.on_read_async(Some(read));
                let set = ToChUtils::get_set_func(device.clone(), ps.clone(), unit_conv.clone());
                cts.on_update_async(Some(set));
                //设置监听器
                let did = device.get_info().did.clone();
                //注册属性事件
                device.register_property(ps.siid, ps.piid).await;
                let listener = ToChUtils::get_miot_listener(sid, hap_type, did, accessory.clone(), ps.clone(), unit_conv.clone());
                device.add_listener(Box::new(listener)).await;
            } else {
                return Err(anyhow!("映射参数不能为空"));
            }
        }
        MappingMethod::JsScript => {
            let read = ToChUtils::get_js_read_func(device.clone(), "".to_string(), unit_conv.clone());
        }
        _ => {
            //脚本映射,需要设置读,写,校验脚本
            // read:
            // actions:
            //   action: eval(val/10)
            //   action: set_ps(val/10)
            // 读到一个value -> 操作函数[] -> value
        }
    }

    Ok(())
}

fn set_default_for_cts(cts: &mut IotCharacteristic, ch: HapCharacteristicModel) -> anyhow::Result<()> {
    let default = IotCharacteristic::new_default(ch.characteristic_type.clone());
    if let Some(default) = default {
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
    pub fn get_miot_listener(sid: u64, hap_type: HapType, did: String, accessory: HapAccessoryPointer,
                             property: Property,
                             unit_conv: UnitConv) -> DataListener<EventType> {
        Box::new(move |data: EventType| {
            let unit_conv = unit_conv.clone();
            let accessory = accessory.clone();
            let did = did.clone();
            let property = property.clone();
            async move {
                if let SetProperty(res) = data {
                    if let Some(value) = res.value {
                        if res.did.as_str() == did.as_str() && res.piid == property.piid && res.siid == property.siid {
                            info!("listen property:{},{},{:?}", res.siid, res.piid, value);

                            // 蓝牙数据设置到特征上
                            match accessory.lock()
                                .await
                                .get_id_mut_service(sid)
                                .and_then(|s| s.get_mut_characteristic(hap_type)) {
                                None => {
                                    return Err(anyhow::anyhow!("特征不存在"));
                                }
                                Some(cts) => {
                                    ///类型转换器,设置值
                                    let value = Self::conv_to_value(unit_conv, value);
                                    cts.set_value(value).await?;
                                }
                            }
                        };
                    };
                };
                Err(anyhow!("数据类型错误"))
            }.boxed()
        })
    }

    pub fn conv_from_value(conv: UnitConv, value: JsonValue) -> JsonValue {
        match conv.0 {
            None => {
                value
            }
            Some(cv) => {
                cv.get_convertor()
                    .from(conv.1, value)
                    .tap_err(|e| {
                        error!("单位转换错误:{:?}",e);
                    })
                    .unwrap_or(Value::Null)
            }
        }
    }

    //转换成目标值
    pub fn conv_to_value(conv: UnitConv, value: JsonValue) -> JsonValue {
        match conv.0 {
            None => {
                value
            }
            Some(uc) => {
                let old = format!("{:?}", value);
                let value = uc.get_convertor().to(conv.1, value)
                    .tap_err(|e| {
                        error!("To单位转换错误:{:?}",e);
                    })
                    .unwrap_or(Value::Null);
                debug!("convert to hap value:{:?}=>{:?}", old, value);
                value
            }
        }
    }

    pub async fn read_property(property: Property, dev: DevicePointer) -> anyhow::Result<Option<Value>> {
        dev.read_property(property.siid, property.piid).await
    }

    pub fn get_set_func(device: DevicePointer, property_id: Property, conv: UnitConv) -> impl OnUpdateFuture<CharacteristicValue>
    {
        move |old: CharacteristicValue, new: CharacteristicValue| {
            let dev = device.clone();
            let conv = conv.clone();
            async move {
                if old == new {
                    return Ok(());
                };
                let value = Self::conv_from_value(conv, new.value);
                //读取状态
                info!("set to iot value:{},{},{:?}",property_id.siid,property_id.piid,value);
                let res = dev.set_property(property_id.siid, property_id.piid, value.clone())
                    .await
                    .tap_err(|e| {
                        error!("设置属性失败:{}", e);
                    });

                Ok(())
            }.boxed()
        }
    }

    // 读取脚本
    pub fn get_js_read_func(device: DevicePointer,
                            read_script: String,
                            conv: UnitConv,
    ) -> impl OnReadFuture<CharacteristicValue> {
        let conv = conv.clone();
        move || {
            let dev = device.clone();
            let conv = conv.clone();
            let read_script = read_script.clone();
            async move {
                let dev = dev.clone();
                let value = dev.eval_js(read_script).await?;
                Ok(Some(CharacteristicValue::new(Self::conv_from_value(conv, value))))
            }.boxed()
        }
    }


    /// 获取wifi 设备读取函数
    pub fn get_read_func(device: DevicePointer,
                         property: Property,
                         conv: UnitConv,
    ) -> impl OnReadFuture<CharacteristicValue> {
        let conv = conv.clone();
        move || {
            let dev = device.clone();
            let conv = conv.clone();
            async move {
                //读取状态
                let value = Self::read_property(property.clone(), dev.clone()).await
                    .tap_err(|e| {
                        error!("属性读取失败:{}", e);
                    })?;

                Ok(value
                    .map(|f| Self::conv_to_value(conv, f))
                    .map(|f| CharacteristicValue::new(f)))
            }.boxed()
        }
    }
}


#[test]
pub fn test() {
    let f = "\"string\"";
    let ff = format!("\"{}\"", "string");
    println!("{}", f);

    // let fmt = serde_json::Value::from(f);
    let format: Format = serde_json::from_str(f)
        .map_err(|e| anyhow!("格式转换错误:{:?}", e)).unwrap();
    println!("{:?}", format);
}