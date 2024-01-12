use serde_json::Value;
use hap_metadata::metadata::HapCharacteristic;
use crate::db::entity::prelude::{HapAccessoryModel, HapBridge, HapBridgeModel, IotDeviceModel};

#[derive(Debug, serde::Serialize)]
pub struct HapAccessoryResult {
    #[serde(flatten)]
    pub model: HapAccessoryModel,
    pub bridge: Option<HapBridgeModel>,
    pub device: Option<IotDeviceModel>,
}

#[derive(Debug, serde::Serialize)]
pub struct ServiceMetaResult {
    //必须的
    pub required: Vec<CharacteristicMetaResult>,
    //可选的
    pub optional: Vec<CharacteristicMetaResult>,
}

#[derive(Debug, serde::Serialize)]
pub struct CharacteristicMetaResult {
    // pub name: String,
    pub format: String,
    pub name: String,
    pub characteristic_type: String,
    pub min_value: Option<Value>,
    pub max_value: Option<Value>,
    pub step_value: Option<Value>,
    pub max_length: Option<Value>,
    pub units: Option<String>,
    // pub properties: usize,
}

impl CharacteristicMetaResult {
    pub(crate) fn from_ch(c: &HapCharacteristic, name: &str) -> Self {
        let characteristic_type = hap_metadata::utils::pascal_case(c.name.as_str());
        Self {
            format: c.format.clone(),
            name: name.to_string(),
            characteristic_type,
            min_value: c.min_value.clone(),
            max_value: c.max_value.clone(),
            step_value: c.step_value.clone(),
            max_length: c.max_length.clone(),
            units: c.units.clone(),
        }
    }
}
