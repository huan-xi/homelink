use std::net::SocketAddr;
use anyhow::anyhow;
use serde::Serialize;
use serde_json::Value;
use hap::characteristic::{Format, Unit};
use hap_metadata::metadata::HapCharacteristic;
use target_hap::types::HapCharInfo;
use crate::db::entity::prelude::{HapAccessoryModel, HapBridgeEntity, HapBridgeModel, IotDeviceModel, MiotDeviceModel};
use crate::init::manager::ble_manager::Status;
use crate::template::hap::accessory::AccessoryTemplate;
use crate::template::hl_template::{DeviceTemplate, HlDeviceTemplate, TemplateFormat};

#[derive(Debug, serde::Serialize)]
pub struct UserInfoResult {
    pub(crate) username: String,
    pub(crate) name: String,
    pub(crate) avatar: String,
    pub(crate) userid: i64,
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
    pub running: bool,
    pub bridge: Option<HapBridgeModel>,
    pub device: Option<IotDeviceModel>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct CheckTemplateResult {
    pub new_devices: Vec<DeviceTemplate>,
    pub new_accessories: Vec<AccessoryTemplate>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct TemplateResult {
    pub(crate) text: String,
    pub(crate) format: TemplateFormat,
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
    pub characteristic_type: String,
    pub info: HapCharInfo,
    pub name: String,
    pub memo: Option<String>,

}

impl CharacteristicMetaResult {
    pub(crate) fn from_ch(c: &HapCharacteristic, name: &str) -> anyhow::Result<Self> {
        let characteristic_type = hap_metadata::utils::pascal_case(c.name.as_str());

        let format: Format = serde_json::from_str(format!("\"{}\"", c.format.as_str()).as_str())
            .map_err(|e| anyhow!("格式转换错误:{:?}", e))?;

        let unit: Option<Unit> = c.units.clone().map(|u| {
            let unit: anyhow::Result<Unit> = serde_json::from_str(format!("\"{}\"", u.as_str()).as_str())
                .map_err(|e| anyhow!("单位转换错误:{:?}", e));
            unit
        }).transpose()?;

        Ok(Self {
            characteristic_type,
            info: HapCharInfo {
                format,
                min_value: c.min_value.clone(),
                max_value: c.max_value.clone(),
                step_value: c.step_value.clone(),
                max_data_len: None,
                valid_values: None,
                valid_values_range: None,
                ttl: None,
                perms: vec![],
                max_len: c.max_length.clone()
                    .and_then(|i| i.as_u64().and_then(|i| Some(i as u16))),
                unit,
                pid: None,
            },
            name: "".to_string(),
            memo: None,
        })
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
pub struct NativeBleStatus {
    pub(crate) status: Status,
}

#[derive(Debug, serde::Serialize)]
pub struct NativeBleDevice {
    pub mac: String,
    pub id: String,
    pub name: Option<String>,
    pub rssi: Option<i16>,
}