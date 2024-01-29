use std::time::Duration;
use anyhow::anyhow;
use bimap::BiMap;
use futures_util::FutureExt;
use lazy_static::lazy_static;
use log::{error, info, warn};
use sea_orm::JsonValue;
use serde_json::Value;
use tap::TapFallible;
use tokio::sync::{MutexGuard, TryLockError};
use hap::accessory::HapAccessory;

use hap::characteristic::{AsyncCharacteristicCallbacks, Characteristic, CharacteristicCallbacks, Format, HapCharacteristic, OnReadFuture, OnUpdateFuture, Perm, Unit};
use hap::HapType;
use miot_spec::device::common::emitter::{DataListener, EventType};
use miot_spec::device::common::emitter::EventType::UpdateProperty;

use crate::config::context::get_app_context;
use crate::db::entity::common::Property;
use crate::db::entity::hap_characteristic::{MappingMethod, MappingParam};
use crate::db::entity::prelude::HapCharacteristicModel;
use crate::hap::hap_type::MappingHapType;
use crate::hap::hap_type::MappingHapType::{SecuritySystemCurrentState, SecuritySystemTargetState};
use crate::hap::iot_characteristic::{CharacteristicValue, IotCharacteristic};
use crate::hap::unit_convertor::UnitConv;
use crate::init::{DevicePointer, HapAccessoryPointer};
use crate::init::hap_init::InitServiceContext;
use crate::init::manager::hap_manager::HapTask;
use crate::js_engine::channel::main_channel::{FromModuleResp, ToModuleEvent};
use crate::js_engine::channel::params::{BindDeviceModuleParam, OnCharReadParam, OnCharUpdateParam};

lazy_static! {
    static ref TARGET_CHAR_MAP: BiMap<MappingHapType,MappingHapType> ={
        let mut map = BiMap::new();
        map.insert(MappingHapType::SecuritySystemTargetState,MappingHapType::SecuritySystemTargetState);
        map
    };
}


/// 转成特征
pub async fn to_characteristic(ctx: InitServiceContext,
                               index: usize, ch: HapCharacteristicModel, ) -> anyhow::Result<IotCharacteristic> {
    let sid = ctx.sid;
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
        accessory_id: ctx.aid,
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
    set_read_update_method(ctx.clone(), &mut cts, ch).await?;
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

/// TargetHeatingCoolingState 0关闭,1制热,2,制冷 3 自动
/// 设置特征的读写方法
async fn set_read_update_method(ctx: InitServiceContext, cts: &mut IotCharacteristic, ch: HapCharacteristicModel) -> anyhow::Result<()> {
    // let cid = cts.get_id();
    let cid = cts.get_id();
    let unit_conv = UnitConv(ch.unit_convertor.clone(), ch.convertor_param.clone());
    match ch.mapping_method {
        MappingMethod::None => {
            let cts_c = cts.get_type();
            //不映射属性
            cts.on_read(Some(move ||
                {
                    info!("read value:{:?}", cts_c);
                    Ok(None)
                }
            ));
            cts.on_update(Some(move |old: &CharacteristicValue, new: &CharacteristicValue| {
                info!("set type:{:?}:to iot value:{:?}",cts_c, new.value);
                Ok(())
            }));
        }
        MappingMethod::FixValue => {
            //todo 固定值
            /*  cts.0.value = ch.fix_value.clone()
                  .map(|i| CharacteristicValue::new(i))
                  .unwrap_or(CharacteristicValue::default());*/
        }
        MappingMethod::PropMapping => {
            // 设置读写映射
            if let Some(MappingParam::MIotSpec(param)) = ch.mapping_param.clone() {
                let ps = param.property;
                let read = ToChUtils::get_read_func(ctx.device.clone(), ps.clone(), unit_conv.clone());
                cts.on_read_async(Some(read));
                // ch.characteristic_type
                let set = ToChUtils::get_set_func(ctx.clone(), ps.clone(), unit_conv.clone(), ch.clone());
                cts.on_update_async(Some(set));

                //注册属性事件
                ctx.device.register_property(ps.siid, ps.piid).await;
                let listener = ToChUtils::get_miot_listener(ctx.clone(), cid, ps.clone(), unit_conv.clone());
                ctx.device.add_listener(listener).await;
            } else {
                return Err(anyhow!("映射参数不能为空"));
            }
        }
        MappingMethod::JsScript => {

            // 获取 channel
            let stag = ctx.stag.clone().ok_or(anyhow!("service 特征标识不能为空"))?;
            let char_tag = ch.tag.clone().ok_or(anyhow!("char 特征标识不能为空"))?;
            let sender = get_app_context().js_engine.sender.clone();
            let sender_c = sender.clone();
            let char_tag_c = char_tag.clone();
            let stag_c = stag.clone();
            let read = move || {
                let char_tag = char_tag_c.clone();
                let stag = stag_c.clone();
                let sender = sender_c.clone();
                async move {
                    let ch_id = get_app_context()
                        .hap_manager
                        .get_accessory_ch_id(ctx.aid).await?;

                    let resp = sender.send(ToModuleEvent::OnCharRead(OnCharReadParam {
                        ch_id,
                        service_tag: stag,
                        char_tag,
                    })).await?;
                    if let FromModuleResp::CharReadResp(value) = resp {
                        return Ok(value.value.map(|f| CharacteristicValue::new(f)));
                    }
                    Err(anyhow!("未知错误"))?
                }.boxed()
            };
            cts.on_read_async(Some(read));
            let sender_c = sender.clone();
            let char_tag_c = char_tag.clone();
            let update = move |old_val: CharacteristicValue, new_val: CharacteristicValue| {
                let sender = sender_c.clone();
                let char_tag = char_tag_c.clone();
                let stag = stag.clone();
                async move {
                    if old_val == new_val {
                        return Ok(());
                    };
                    let ch_id = get_app_context()
                        .hap_manager
                        .get_accessory_ch_id(ctx.aid)
                        .await?;
                    // 像js 发送更新事件
                    let _ = sender.send(ToModuleEvent::OnCharUpdate(OnCharUpdateParam {
                        ch_id,
                        service_tag: stag,
                        old_value: old_val.value,
                        char_tag,
                        new_value: new_val.value,
                    })).await?;

                    Ok(())
                }.boxed()
            };
            cts.on_update_async(Some(update));
            //注册事件,dev 将事件 发送到js,js调用ch 的 事件处理方法,可以复用消息
            let did = ctx.device.get_info().did.clone();
            get_app_context().device_manager.enable_to_js(did.as_str()).await?;

            // crate::js_engine::channel::main_channel::ToModuleEvent
            let ch_id = get_app_context()
                .hap_manager
                .get_accessory_ch_id(ctx.aid).await?;

            let _ = get_app_context()
                .js_engine
                .send(
                    ToModuleEvent::BindDeviceModule(BindDeviceModuleParam {
                        ch_id,
                        dev_id: did,
                    })).await
                .map_err(|f| anyhow!("绑定事件通道失败"))?;
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
            cts.0.max_value = default.get_max_value().map(|i| CharacteristicValue::new(i));
        }
        if cts.0.min_value.is_none() {
            cts.0.min_value = default.get_min_value().map(|i| CharacteristicValue::new(i));
        }
        if cts.0.step_value.is_none() {
            cts.0.step_value = default.get_step_value().map(|i| CharacteristicValue::new(i));
        }
        if cts.get_type() == HapType::CurrentHeatingCoolingState {
            let a = default.get_valid_values();
            info!("valid_values:{:?}", a);
        };
        if cts.0.valid_values.is_none() {
            cts.0.valid_values = default.get_valid_values()
                .map(|f| f.into_iter().map(|i| CharacteristicValue::new(i)).collect());
        };
        if cts.0.valid_values_range.is_none() {
            /*cts.0.valid_values_range = default.get_valid_values_range()
                .map(|f| f.into_iter()
                    .map(|i| CharacteristicValue::new(serde_json::Value::from(i)))
                    .collect());*/
        }

        cts.0.perms = default.get_perms();
    } else {
        warn!("特征:{:?}没有默认值", cts.get_type());
    }

    Ok(())
}


pub struct ToChUtils {}


impl ToChUtils {
    pub fn get_miot_listener(ctx: InitServiceContext,
                             cid: u64,
                             property: Property,
                             unit_conv: UnitConv) -> DataListener<EventType> {
        Box::new(move |data: EventType| {
            let unit_conv = unit_conv.clone();
            let accessory = ctx.accessory.clone();
            let did = ctx.device.get_info().did.clone();
            let property = property.clone();
            async move {
                if let UpdateProperty(res) = data {
                    if let Some(value) = res.value {
                        if res.did.as_str() == did.as_str() && res.piid == property.piid && res.siid == property.siid {
                            // todo 只有值变化时才更新
                            // info!("listen property:{},{},{:?}", res.siid, res.piid, value);
                            // 蓝牙数据设置到特征上
                            tokio::spawn(async move {
                                match accessory.lock()
                                    .await
                                    .get_id_mut_service(ctx.sid)
                                    .and_then(|s| s.get_id_mut_characteristic(cid)) {
                                    None => {
                                        warn!("特征:{}不存在",cid)
                                    }
                                    Some(cts) => {
                                        ///类型转换器,设置值
                                        let value = Self::conv_to_value(unit_conv, value);
                                        if let Err(e) = cts.set_value(value).await {
                                            warn!("设置特征值失败:{:?}",e);
                                        }
                                    }
                                };
                            });
                            return Ok(());
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
                // debug!("convert to hap value:{:?}=>{:?}", old, value);
                value
            }
        }
    }

    pub async fn read_property(dev: DevicePointer, property: Property) -> anyhow::Result<Option<Value>> {
        dev.read_property(property.siid, property.piid).await
    }

    /// hap 配件更新到设备
    /// 存在target 属性
    pub fn get_set_func(ctx: InitServiceContext,
                        property_id: Property,
                        conv: UnitConv,
                        ch_model: HapCharacteristicModel) -> impl OnUpdateFuture<CharacteristicValue>
    {
        move |old: CharacteristicValue, new: CharacteristicValue| {
            let dev = ctx.device.clone();
            let accessory = ctx.accessory.clone();
            let conv = conv.clone();
            async move {
                if old == new {
                    return Ok(());
                };

                let value = Self::conv_from_value(conv, new.value.clone());
                //读取状态
                // debug!("set to iot value:{},{},{:?}",property_id.siid,property_id.piid,value);
                if TARGET_CHAR_MAP.contains_right(&ch_model.characteristic_type) {
                    return Ok(());
                };

                let res = dev.set_property(property_id.siid, property_id.piid, value.clone())
                    .await
                    .tap_err(|e| {
                        error!("设置属性失败:{}", e);
                    });
                if let Ok(_) = res {
                    //设置成功,判断是否存在curr特征
                    if let Some(curr_type) = TARGET_CHAR_MAP.get_by_left(&ch_model.characteristic_type) {
                        let curr: HapType = curr_type.clone().into();
                        //开一个task更新,否则会重入死锁
                        let sid = ctx.sid;
                        let value = new.value.clone();
                        tokio::spawn(async move {
                            if let Some(svc) = accessory.lock().await.get_id_mut_service(sid) {
                                if let Some(curr) = svc.get_mut_characteristic(curr) {
                                    if let Err(e) = curr.set_value(value).await {
                                        warn!("设置curr特征失败:{:?}", e);
                                    };
                                }
                            }
                        });
                    }
                };


                Ok(())
            }.boxed()
        }
    }


    /// 获取 属性映射读取函数
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
                let value = Self::read_property(dev.clone(), property.clone()).await
                    .tap_err(|e| {
                        error!("属性读取失败:{}", e);
                    })?;
                info!("read value:{:?}",value);
                Ok(value
                    .map(|f| Self::conv_to_value(conv, f))
                    .map(|f| CharacteristicValue::new(f)))
            }.boxed()
        }
    }
}

#[test]
pub fn test2() {
    let a = MappingHapType::SecuritySystemTargetState;
    let json = serde_json::to_string(&a).unwrap();
    println!("{:?}", a);
    println!("{}", json);
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