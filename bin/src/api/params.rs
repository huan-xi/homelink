use std::str::FromStr;
use anyhow::anyhow;
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveValue, JsonValue};
use crate::hap::hap_type::MappingHapType;
use serde_aux::prelude::deserialize_number_from_string;
use crate::db::entity::hap_characteristic::{BleToSensorParam, DbBleValueType, MappingMethod, MappingParam, MiotSpecParam};
use serde_aux::prelude::deserialize_option_number_from_string;
use hap::HapType;
use crate::db::entity::common::{Property, PropertyVec};
use crate::db::entity::hap_bridge::BridgeCategory;
use crate::db::entity::prelude::{HapAccessoryActiveModel, HapCharacteristicActiveModel};
use crate::hap::unit_convertor::{ConvertorParamType, UnitConvertor};

#[derive(serde::Deserialize, Debug)]
pub struct LoginParam {
    /// 账号
    pub password: String,
    pub username: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct AddHapBridgeParam {
    /// pin码
    pub pin_code: Option<String>,

    pub category: BridgeCategory,
    pub name: String,

}

#[derive(serde::Deserialize, Debug)]
pub struct SyncDeviceParam {
    /// 账号
    pub account: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct AddServiceParam {
    pub memo: Option<String>,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub accessory_id: i64,
    pub name: Option<String>,
    /// 服务类型
    pub service_type: MappingHapType,
    pub characteristics: Vec<CharacteristicParam>,
}

#[derive(serde::Deserialize, Debug)]
pub struct AddHapAccessoryParam {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    device_id: i64,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    bridge_id: i64,
    name: Option<String>,
    memo: Option<String>,
    script: Option<String>,
    disabled: Option<bool>,
    hap_type: Option<MappingHapType>,
    register_props: PropertyVec,
}

impl AddHapAccessoryParam {
    pub fn into_model(self) -> anyhow::Result<HapAccessoryActiveModel> {
        Ok(HapAccessoryActiveModel {
            aid: Default::default(),
            device_id: Set(self.device_id),
            bridge_id: Set(self.bridge_id),
            name: Set(self.name),
            memo: Set(self.memo),
            disabled: Set(self.disabled.unwrap_or(false)),
            hap_type: Set(self.hap_type),
            info: Default::default(),
            script: Set(self.script),
            register_props: Set(self.register_props),
        })
    }
}


#[derive(serde::Deserialize, Debug)]
pub struct CharacteristicParam {
    #[serde(default, deserialize_with = "deserialize_option_number_from_string")]
    pub cid: Option<i64>,
    pub characteristic_type: MappingHapType,
    pub mapping_method: MappingMethod,
    pub mapping_property: Option<Property>,
    pub name: Option<String>,
    pub ble_value_type: Option<DbBleValueType>,
    pub format: String,
    pub unit: Option<String>,
    pub min_value: Option<JsonValue>,
    pub max_value: Option<JsonValue>,
    pub max_len: Option<JsonValue>,
    pub unit_convertor: Option<UnitConvertor>,
    pub convertor_param: Option<ConvertorParamType>,
    pub fixed_value: Option<String>,
}

impl CharacteristicParam {
    pub fn into_model(self, service_id: i64) -> anyhow::Result<HapCharacteristicActiveModel> {
        let mapping_param = match &self.mapping_method {
            MappingMethod::PropMapping => {
                Some(match self.mapping_property.clone() {
                    None => {
                        return Err(anyhow!("mapping_property 不能为空"));
                    }
                    Some(s) => {
                        MappingParam::MIotSpec(MiotSpecParam {
                            property: s,
                        })
                    }
                })
            }
            _ => None,
        };
        Ok(HapCharacteristicActiveModel {
            cid: Default::default(),
            characteristic_type: Set(self.characteristic_type),
            mapping_method: Set(self.mapping_method),
            mapping_param: Set(mapping_param),
            name: Set(self.name),
            format: Set(self.format),
            unit: Set(self.unit),
            min_value: Set(self.min_value),
            max_value: Set(self.max_value),
            tag: Default::default(),
            max_len: Set(self.max_len),
            unit_convertor: Set(self.unit_convertor),
            service_id: Set(service_id),
            disabled: Set(false),
            // fixed_value: Default::default(),
            convertor_param: Set(self.convertor_param),
            fixed_value: Set(self.fixed_value),
        })
    }
}


#[derive(serde::Deserialize, Debug)]
pub struct DisableParam {
    pub disabled: bool,
}

#[derive(serde::Deserialize, Debug)]
pub struct UpdateHapAccessoryParam {
    pub name: Option<String>,
    pub memo: Option<String>,
    pub hap_type: Option<MappingHapType>,
    // pub script: Option<String>,
    // pub register_props: PropertyVec,
}

impl UpdateHapAccessoryParam {
    pub fn into_model(self, id: i64) -> anyhow::Result<HapAccessoryActiveModel> {
        Ok(HapAccessoryActiveModel {
            aid: Set(id),
            name: Set(self.name),
            memo: Set(self.memo),
            hap_type: Set(self.hap_type),
            // script: Set(self.script),
            // register_props: Set(self.register_props),
            ..Default::default()
        })
    }
}


#[derive(serde::Deserialize, Debug)]
pub struct Test {
    #[serde(default, deserialize_with = "deserialize_option_number_from_string")]
    pub cid: Option<i32>,
}


#[test]
pub fn test() {
    let str = r#"{"characteristic_type":"PowerState","mapping_method":"MIotSpec","service_id":"1194242687084003328","mapping_property":{"siid":"2","piid":"1"},"format":"bool","name":"on"}"#;
    let param: Test = serde_json::from_str(str).unwrap();
    println!("{:?}", param);
}

#[test]
pub fn test2() {
    let a = MappingHapType::Name;
    let str = serde_json::to_string(&a).unwrap();
    println!("{}", str);
    println!("{:?}", a);
}