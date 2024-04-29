use std::str::FromStr;
use anyhow::anyhow;
use sea_orm::{FromJsonQueryResult, JsonValue};
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use strum_macros::EnumString;
use hap::characteristic::{Format, Perm, Unit};
use hap::HapType;
use miot_proto::proto::miio_proto::MiotSpecId;
use target_hap::hap_type_wrapper::HapTypeWrapper;
use target_hap::types::{CharIdentifier, HapCharInfo, ModelDelegateParam};
use crate::db::entity::hap_bridge::BridgeCategory;
use crate::db::entity::iot_device::DeviceType;
use crate::db::entity::prelude::HapCharacteristicModel;
use crate::template::hap::accessory::AccessoryTemplate;
use crate::unit_convertor::{UnitConvertor, UnitConvertorType};

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub enum TemplateFormat {
    #[serde(rename = "yaml")]
    Yaml,
    #[serde(rename = "toml")]
    Toml,
}

impl TemplateFormat {
    pub fn parse<'a, T: DeserializeOwned>(&'a self, text: &'a str) -> anyhow::Result<T> {
        Ok(match self {
            TemplateFormat::Yaml => {
                serde_yaml::from_str(text)?
            }
            TemplateFormat::Toml => {
                toml::from_str(text)?
            }
        })
    }
    pub fn format_to_str<T: Serialize>(&self, value: &T) -> anyhow::Result<String> {
        Ok(match self {
            TemplateFormat::Yaml => serde_yaml::to_string(value)?,
            TemplateFormat::Toml => toml::to_string(value)?,
        })
    }
}

pub(crate) fn default_text(str: &'static str) -> String {
    str.to_string()
}


pub(crate) fn default_str() -> String {
    "default".to_string()
}

fn default_false() -> bool {
    false
}

/// iot device 转模板
/// 一个米家设备 对应多个iot device模板
/// 一个iot device 对应多个配件，配件必须基于桥接器存在
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct HlDeviceTemplate {
    ///模板的标识
    pub id: String,
    /// 模型名称
    pub model_name: String,
    /// 模板的版本
    pub version: String,
    /// 模型
    pub model: String,
    /// 模型版本
    pub fw_version: Option<String>,
    /// 模型icon
    pub model_icon: Option<String>,
    /// 设备
    #[serde(default)]
    pub devices: Vec<DeviceTemplate>,
}

impl FromStr for HlDeviceTemplate {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let result: HlDeviceTemplate = toml::from_str(s)?;
        Ok(result)
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DeviceTemplate {
    pub integration: String,
    #[serde(default = "default_str")]
    pub tag: String,
    pub interval: Option<u64>,
    /// 展示名称
    pub display_name: Option<String>,
    pub device_type: Option<DeviceType>,
    pub disabled: Option<bool>,
    pub timeout: Option<u64>,
    #[serde(default)]
    pub poll_properties: Vec<MiotSpecId>,
    pub memo: Option<String>,
    #[serde(default)]
    pub params: JsonValue,
    /// 配件
    #[serde(default)]
    pub accessories: Vec<AccessoryTemplate>,
}


pub mod test {
    use tokio::fs::File;
    use tokio::io::AsyncReadExt;
    use hap::accessory::security_system::SecuritySystemAccessory;
    use hap::characteristic::security_system_current_state::SecuritySystemCurrentStateCharacteristic;
    use hap::characteristic::security_system_target_state::SecuritySystemTargetStateCharacteristic;
    use hap::service::lightbulb::LightbulbService;
    use crate::db::entity::hap_accessory::Column::Category;
    use crate::hap::hap_type::MappingHapType;
    use crate::hap::hap_type::MappingHapType::SecuritySystemTargetState;
    use crate::template::hl_template::HlDeviceTemplate;

    #[tokio::test]
    pub async fn test() -> anyhow::Result<()> {
        let mut file = File::open("/Users/huanxi/project/homelink/templates/mijia/chuangmi.plug.212a01.toml").await.unwrap();
        let mut content = String::new();
        file.read_to_string(&mut content).await?;
        // SecuritySystemAccessory::new();
        // MappingHapType::SecuritySystem;
        // SecuritySystemCurrentStateCharacteristic::new();
        // SecuritySystemTargetStateCharacteristic::new();
        // LightbulbService::new();
        let result: HlDeviceTemplate = toml::from_str(content.as_str()).unwrap();
        println!("{:?}", result);
        // let result: HashMap<String, Miot2HapMapper> = toml::from_str(content.as_str())?;
        // PowerStateCharacteristic::new();
        Ok(())
    }
}