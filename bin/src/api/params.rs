use std::str::FromStr;
use sea_orm::JsonValue;
use serde_json::Value;
use crate::convertor::hap_type::MappingHapType;
use serde_aux::prelude::deserialize_number_from_string;
use miot_spec::device::ble::value_types::BleValueType;
use crate::db::entity::hap_characteristic::{DbBleValueType, MappingMethod, Property};

fn deserialize_number_or_string<'de, D>(deserializer: D) -> Result<i64, D::Error>
    where
        D: serde::Deserializer<'de>,
{
    let value: Value = serde::Deserialize::deserialize(deserializer)?;
    match value {
        Value::Number(number) => {
            if let Some(number) = number.as_i64() {
                Ok(number)
            } else {
                Err(serde::de::Error::custom("Invalid number format"))
            }
        }
        Value::String(string) => i64::from_str(&string).map_err(serde::de::Error::custom),
        _ => Err(serde::de::Error::custom("Invalid value type")),
    }
}


#[derive(serde::Deserialize, Debug)]
pub struct AddServiceParam {
    /// 服务名称可空
    pub name: Option<String>,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub accessory_id: i64,
    /// 服务类型
    pub service_type: MappingHapType,
}

#[derive(serde::Deserialize, Debug)]
pub struct AddCharacteristicParam {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub service_id: i64,
    pub characteristic_type: MappingHapType,
    pub mapping_method: MappingMethod,
    pub mapping_property: Option<Property>,

    pub ble_value_type: Option<DbBleValueType>,
    pub format: Option<String>,
    pub min_value: Option<JsonValue>,
    pub max_value: Option<JsonValue>,
}

#[derive(serde::Deserialize, Debug)]
pub struct DisableParam {
    pub disabled: bool,
}