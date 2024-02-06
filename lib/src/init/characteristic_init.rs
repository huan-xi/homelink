use anyhow::anyhow;
use bimap::BiMap;
use futures_util::{FutureExt, SinkExt};
use lazy_static::lazy_static;
use log::{error, info, warn};
use sea_orm::JsonValue;
use serde_json::Value;
use tap::TapFallible;

use hap::accessory::HapAccessory;
use hap::characteristic::{AsyncCharacteristicCallbacks, Characteristic, CharacteristicCallbacks, Format, HapCharacteristic, OnReadFuture, OnUpdateFuture, Perm, Unit};
use hap::HapType;
use miot_spec::device::common::emitter::{DataListener, EventType};
use miot_spec::device::common::emitter::EventType::UpdateProperty;
use miot_spec::proto::miio_proto::MiotSpecId;
use crate::db::entity::hap_characteristic::{HapCharInfo, MappingMethod, MappingParam};
use crate::db::entity::prelude::HapCharacteristicModel;
use crate::hap::hap_type::MappingHapType;
use crate::hap::iot::iot_characteristic::{CharacteristicValue, IotCharacteristic};
use crate::hap::unit_convertor::UnitConv;
use crate::init::DevicePointer;
use crate::init::hap_init::InitServiceContext;

#[cfg(feature = "deno")]
use crate::js_engine::channel::{
    main_channel::{FromModuleResp, ToModuleEvent},
    params::{BindDeviceModuleParam, OnCharReadParam, OnCharUpdateParam},
};


lazy_static! {
    static ref TARGET_CHAR_MAP: BiMap<MappingHapType,MappingHapType> ={
        let mut map = BiMap::new();
        map.insert(MappingHapType::SecuritySystemTargetState,MappingHapType::SecuritySystemCurrentState);
        map
    };
}


/// 转成特征
pub async fn to_characteristic(ctx: InitServiceContext,
                               index: usize, ch: HapCharacteristicModel, ) -> anyhow::Result<IotCharacteristic> {
    let sid = ctx.sid;
    let format: Format = ch.info.format;
    let unit: Option<Unit> = ch.info.unit;

    let unit_conv = UnitConv(ch.unit_convertor.clone(), ch.convertor_param.clone());

    let mut cts = IotCharacteristic(Characteristic::<CharacteristicValue> {
        id: sid + index as u64 + 1,
        accessory_id: ctx.aid,
        hap_type: ch.characteristic_type.into(),
        format,
        unit,
        max_value: ch.info.max_value
            .clone()
            .map(|i| CharacteristicValue::format(format, ToChUtils::conv_from_value(unit_conv.clone(), i))),
        min_value: ch.info.min_value.clone()
            .map(|i| CharacteristicValue::format(format, ToChUtils::conv_from_value(unit_conv.clone(), i))),
        perms: ch.info.perms.clone(),
        ..Default::default()
    });

    if let Some(cnv) = ch.unit_convertor.as_ref() {
        if cnv.get_convertor().is_inverse(ch.convertor_param.clone()) {
            //大小值反转
            let max = cts.0.max_value.clone();
            cts.0.max_value = cts.0.min_value.clone();
            cts.0.min_value = max;
        }
    };

    // 设置默认值
    let df = ctx.hap_manage.get_hap_default_info(cts.0.hap_type);
    set_default_for_cts(&mut cts, ch.clone(), df)?;

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
        MappingMethod::AccessoryModel | MappingMethod::JsScript => {
            //模型接管什么都不需要做
        }
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
        // 米家属性映射
        MappingMethod::PropMapping => {
            /*// 设置读写映射
            if let Some(MappingParam::PropMapping(param)) = ch.mapping_param.clone() {
                let ps: MiotSpecId = param.into();
                let read = ToChUtils::get_read_func(ctx.device.clone(), ps, unit_conv.clone());
                cts.on_read_async(Some(read));
                // ch.characteristic_type
                let set = ToChUtils::get_set_func(ctx.clone(), ps, unit_conv.clone(), ch.clone());
                cts.on_update_async(Some(set));

                //注册属性事件
                ctx.device.register_property(ps.siid, ps.piid).await;
                let listener = ToChUtils::get_miot_listener(ctx.clone(), cid, ps, unit_conv.clone());
                ctx.device.add_listener(listener).await;
            } else {
                return Err(anyhow!("映射参数不能为空"));
            }*/
        }
    }

    Ok(())
}

fn set_default_for_cts(cts: &mut IotCharacteristic, ch: HapCharacteristicModel, default: Option<HapCharInfo>) -> anyhow::Result<()> {
    // let default = IotCharacteristic::new_default(ch.characteristic_type.clone());
    if let Some(default) = default {
        if cts.0.unit.is_none() {
            cts.0.unit = default.unit;
        };
        if cts.0.max_len.is_none() {
            cts.0.max_len = default.max_len;
        };
        if cts.0.max_value.is_none() {
            cts.0.max_value = default.max_value.map(|i| CharacteristicValue::new(i));
        }
        if cts.0.min_value.is_none() {
            cts.0.min_value = default.min_value.map(|i| CharacteristicValue::new(i));
        }
        if cts.0.step_value.is_none() {
            cts.0.step_value = default.step_value.map(|i| CharacteristicValue::new(i));
        }
        if cts.0.valid_values.is_none() {
            cts.0.valid_values = default.valid_values
                .map(|f| f.into_iter().map(|i| CharacteristicValue::new(i)).collect());
        };
        cts.0.perms = default.perms;
    } else {
        warn!("特征:{:?}没有默认值", cts.get_type());
    }

    Ok(())
}


pub struct ToChUtils;


impl ToChUtils {
 /*   pub fn get_miot_listener(ctx: InitServiceContext,
                             cid: u64,
                             property: MiotSpecId,
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
                            // info!("listen property:{},{},{:?}", res.siid, res.piid, value);
                            // 蓝牙数据设置到特征上
                            tokio::spawn(async move {
                                match accessory.write()
                                    .await
                                    .get_mut_service_by_id(ctx.sid)
                                    .and_then(|s| s.get_mut_characteristic_by_id(cid)) {
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
    }*/

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

    //hap_platform 转换成目标值
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
                // debug!("convert to hap_platform value:{:?}=>{:?}", old, value);
                value
            }
        }
    }

    pub async fn read_property(dev: DevicePointer, property: MiotSpecId) -> anyhow::Result<Option<Value>> {
        dev.read_property(property.siid, property.piid).await
    }

    /// hap_platform 配件更新到设备
    /// 存在target 属性
    pub fn get_set_func(ctx: InitServiceContext,
                        property_id: MiotSpecId,
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

                dev.set_property(property_id, value.clone())
                    .await
                    .tap_err(|e| {
                        error!("设置属性失败:{}", e);
                    })?;

                //设置成功,判断是否存在curr特征
                if let Some(curr_type) = TARGET_CHAR_MAP.get_by_left(&ch_model.characteristic_type) {
                    let curr: HapType = curr_type.clone().into();
                    //开一个task更新,否则会重入死锁
                    let sid = ctx.sid;
                    let value = new.value.clone();
                    tokio::spawn(async move {
                        if let Some(svc) = accessory.write().await.get_mut_service_by_id(sid) {
                            if let Some(curr) = svc.get_mut_characteristic(curr) {
                                if let Err(e) = curr.set_value(value).await {
                                    warn!("设置curr特征失败:{:?}", e);
                                };
                            }
                        }
                    });
                }


                Ok(())
            }.boxed()
        }
    }


    /// 获取 属性映射读取函数
    pub fn get_read_func(device: DevicePointer,
                         property: MiotSpecId,
                         conv: UnitConv,
    ) -> impl OnReadFuture<CharacteristicValue> {
        let conv = conv.clone();
        move || {
            let dev = device.clone();
            let conv = conv.clone();
            async move {
                //读取状态

                let value = dev.read_property(property.siid, property.piid)
                    .await
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