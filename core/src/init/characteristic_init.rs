use std::str::FromStr;
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
use hap::characteristic::power_state::PowerStateCharacteristic;
use hap::HapType;
use miot_proto::device::common::emitter::{DataListener, EventType};
use miot_proto::device::common::emitter::EventType::UpdateProperty;
use miot_proto::proto::miio_proto::MiotSpecId;
use target_hap::hap_type_wrapper::HapTypeWrapper;
use target_hap::iot::characteristic_value::CharacteristicValue;
use target_hap::iot::iot_characteristic::IotCharacteristic;
use target_hap::types::HapCharInfo;
use crate::db::entity::prelude::HapCharacteristicModel;
use crate::hap::hap_type::MappingHapType;
use crate::init::hap_init::InitServiceContext;
//
// #[cfg(feature = "deno")]
// use crate::js_engine::channel::{
//     main_channel::{FromModuleResp, ToModuleEvent},
//     params::{BindDeviceModuleParam, OnCharReadParam, OnCharUpdateParam},
// };
//

lazy_static! {
    static ref TARGET_CHAR_MAP: BiMap<MappingHapType,MappingHapType> ={
        let mut map = BiMap::new();
        map.insert(MappingHapType::SecuritySystemTargetState,MappingHapType::SecuritySystemCurrentState);
        map
    };
}


/// 转成特征
pub async fn to_characteristic(ctx: InitServiceContext, index: usize, ch: HapCharacteristicModel) -> anyhow::Result<IotCharacteristic> {
    let sid = ctx.sid;
    let info = ch.info.0.clone();
    let format: Format = info.format;
    let unit: Option<Unit> = info.unit;

    // let unit_conv = UnitConv(ch.unit_convertor.clone(), ch.convertor_param.clone());

    let mut cts = IotCharacteristic(Characteristic::<CharacteristicValue> {
        id: sid + index as u64 + 1,
        accessory_id: ctx.aid,
        hap_type: HapTypeWrapper::from_str(ch.characteristic_type.as_str())?.into(),
        format,
        unit,
        max_value: info.max_value
            .clone()
            .map(|i| CharacteristicValue::format(format, i)),
        min_value: info.min_value.clone()
            .map(|i| CharacteristicValue::format(format, i)),
        perms: info.perms.clone(),
        ..Default::default()
    });

    // 设置默认值
    let df = ctx.hap_manage.get_hap_default_info(cts.0.hap_type);
    set_default_for_cts(&mut cts,  ch, df)?;

    // 这是属性方法
    // set_read_update_method(ctx.clone(), &mut cts, ch).await?;
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

fn set_default_for_cts(cts: &mut IotCharacteristic, ch:  HapCharacteristicModel, default: Option<HapCharInfo>) -> anyhow::Result<()> {
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
    //default value 设置
    if let Some(ref min_value) = &cts.0.min_value {
        cts.0.value = min_value.clone();
    } else if let Some(ref valid_values) = &cts.0.valid_values {
        if !valid_values.is_empty() {
            cts.0.value = valid_values[0].clone();
        }
    }
    // PowerStateCharacteristic::new();


    Ok(())
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