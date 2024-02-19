use std::net::SocketAddr;
use serde::Serialize;
use serde_json::Value;
use hap_metadata::metadata::HapCharacteristic;
use crate::db::entity::prelude::{HapAccessoryModel, HapBridgeEntity, HapBridgeModel, IotDeviceModel, MiotDeviceModel};
use crate::init::manager::ble_manager::Status;

#[derive(Debug, serde::Serialize)]
pub struct UserInfoResult {
    pub(crate) username: String,
    pub(crate) roles: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct HapBridgeResult {
    #[serde(flatten)]
    pub(crate) model: HapBridgeModel,
    pub setup_uri: String,
    pub peers: Vec<SocketAddr>,
    pub running: bool,

    pub accessory_count: u64,
    pub is_paired: bool,
}

#[derive(Debug, serde::Serialize)]
pub struct IotDeviceResult {
    #[serde(flatten)]
    pub model: IotDeviceModel,

    pub running: bool,
    pub source: Option<MiotDeviceResult>,
}

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


#[derive(serde::Deserialize, Debug, Serialize)]
pub struct MiotDeviceModelResult {
    #[serde(flatten)]
    pub model: MiotDeviceModel,
    /// 是否有模板
    pub has_template: bool,
}

#[derive(serde::Deserialize, Debug, Serialize)]
pub struct MiotDeviceResult {
    pub did: String,
    pub token: String,
    pub name: String,
    pub model: String,
    pub localip: Option<String>,
    pub mac: Option<String>,
    pub is_online: Option<bool>,
    pub full: Option<String>,
}


#[derive(Debug, serde::Serialize)]
pub struct NativeBleDeviceResult {
    pub(crate) status: Status,
    pub(crate) peripherals: Vec<NativeBleDevice>,
}

#[derive(Debug, serde::Serialize)]
pub struct NativeBleDevice {
    pub mac: String,
    pub name: Option<String>,
    pub rssi: Option<i16>,
}