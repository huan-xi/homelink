use std::str::FromStr;
use anyhow::anyhow;
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveValue, JsonValue, NotSet};
use crate::hap::hap_type::MappingHapType;
use serde_aux::prelude::deserialize_number_from_string;
use crate::db::entity::hap_characteristic::{BleToSensorParam, DbBleValueType, MappingMethod, MappingParam, PropMappingParam};
use serde_aux::prelude::deserialize_option_number_from_string;
use hap::characteristic::Format;
use hap::HapType;
use target_hap::hap_type_wrapper::HapTypeWrapper;
use crate::db::entity::common::{Property, PropertyVec};
use crate::db::entity::hap_bridge::BridgeCategory;
use crate::db::entity::iot_device::IotDeviceType;
use crate::db::entity::prelude::{HapAccessoryActiveModel, HapCharacteristicActiveModel};
use crate::init::manager::template_manager::BridgeMode;
// use crate::init::manager::template_manager::BridgeMode;

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
pub struct AccountParam {
    /// 账号
    pub account: String,
    pub password: Option<String>,
}

#[derive(serde::Deserialize, Debug)]
pub struct DidParam {
    /// did
    pub did: String,
}


#[derive(serde::Deserialize, Debug)]
pub struct MiConvertByTemplateParam {
    /// 使用的模板id
    pub id: String,
    pub did: String,
    /// 设备列表
    // pub devices: Vec<String>,
    pub bridge_mode: BridgeMode,
    #[serde(default, deserialize_with = "deserialize_option_number_from_string")]
    pub bridge_id: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_option_number_from_string")]
    pub gateway_id: Option<i64>,
}

#[derive(serde::Deserialize, Debug)]
pub struct MiConvertToIotParam {
    /// did
    pub id: String,
    pub did: String,
    pub device_type: IotDeviceType,
    pub name: String,
    #[serde(default, deserialize_with = "deserialize_option_number_from_string")]
    pub gateway_id: Option<i64>,
}


#[derive(serde::Deserialize, Debug)]
pub struct QueryIotDeviceParam {
    pub device_type: Option<IotDeviceType>,
}

#[derive(serde::Deserialize, Debug)]
pub struct TestPropParam {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub siid: i32,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub piid: i32,
    pub value: Option<JsonValue>,
}

#[derive(serde::Deserialize, Debug)]
pub struct AddServiceParam {
    pub memo: Option<String>,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub accessory_id: i64,
    pub configured_name: Option<String>,
    /// 服务类型
    pub service_type: HapTypeWrapper,
    pub characteristics: Vec<CharacteristicParam>,
}

#[derive(serde::Deserialize, Debug)]
pub struct AddHapAccessoryParam {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    device_id: i64,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    bridge_id: i64,
    name: String,
    memo: Option<String>,
    disabled: Option<bool>,
    category: BridgeCategory,
    listening_props: PropertyVec,
}

impl AddHapAccessoryParam {
    pub fn into_model(self) -> anyhow::Result<HapAccessoryActiveModel> {
        Ok(HapAccessoryActiveModel {
            device_id: Set(self.device_id),
            bridge_id: Set(self.bridge_id),
            name: Set(self.name),
            memo: Set(self.memo),
            disabled: Set(self.disabled.unwrap_or(false)),
            category: Set(self.category),
            update_at: Set(chrono::Local::now().naive_local()),
            ..Default::default()
        })
    }
}


#[derive(serde::Deserialize, Debug)]
pub struct CharacteristicParam {
    #[serde(default, deserialize_with = "deserialize_option_number_from_string")]
    pub cid: Option<i64>,
    pub characteristic_type: String,
    pub mapping_method: MappingMethod,
    pub mapping_property: Option<PropMappingParam>,
    pub name: Option<String>,
    pub ble_value_type: Option<DbBleValueType>,
    pub format: String,
    pub unit: Option<String>,
    pub min_value: Option<JsonValue>,
    pub max_value: Option<JsonValue>,
    pub max_len: Option<JsonValue>,
    // pub unit_convertor: Option<UnitConvertor>,
    // pub convertor_param: Option<ConvertorParamType>,
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
                        MappingParam::PropMapping(s)
                    }
                })
            }
            _ => None,
        };
        let format: Format = serde_json::from_str(format!("\"{}\"", self.format.as_str()).as_str())
            .map_err(|e| anyhow!("格式转换错误:{:?}", e))?;
        let model = HapCharacteristicActiveModel {
            cid: Default::default(),
            characteristic_type: Set(self.characteristic_type.clone()),
            name: Set(self.name),
            service_id: Set(service_id),
            disabled: Set(false),
            ..Default::default()
        };


        Ok(model)
    }
}


#[derive(serde::Deserialize, Debug)]
pub struct HapBridgeListParam {
    #[serde(default)]
    pub single_accessory: Option<bool>,
}

#[derive(serde::Deserialize, Debug)]
pub struct DisableParam {
    pub disabled: bool,
}

#[derive(serde::Deserialize, Debug)]
pub struct UpdateHapAccessoryParam {
    pub name: String,
    pub memo: Option<String>,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub bridge_id: i64,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub device_id: i64,
    pub category: BridgeCategory,
    // pub script: Option<String>,
    // pub register_props: PropertyVec,
}

impl UpdateHapAccessoryParam {
    pub fn into_model(self, id: i64) -> anyhow::Result<HapAccessoryActiveModel> {
        Ok(HapAccessoryActiveModel {
            aid: Set(id),
            name: Set(self.name),
            memo: Set(self.memo),
            category: Set(self.category),
            bridge_id: Set(self.bridge_id),
            device_id: Set(self.device_id),
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