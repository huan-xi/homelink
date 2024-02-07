use std::str::FromStr;
use sea_orm::{FromJsonQueryResult, JsonValue};
use serde::{Deserialize, Serialize};
use hap::characteristic::{Format, Perm, Unit};
use miot_spec::proto::miio_proto::MiotSpecId;
use crate::db::entity::hap_accessory::ModelDelegateParam;
use crate::db::entity::hap_bridge::BridgeCategory;
use crate::db::entity::hap_characteristic::{HapCharInfo, MappingMethod, MappingParam};
use crate::db::entity::iot_device::{DeviceParam, IotDeviceType};
use crate::hap::hap_type::MappingHapType;
use crate::hap::unit_convertor::{ConvertorParamType, UnitConvertor};

fn default_text() -> String {
    "default".to_string()
}

fn default_false() -> bool {
    false
}

/// iot device 转模板
/// 一个米家设备 对应多个iot device模板
/// 一个iot device 对应多个配件，配件必须基于桥接器存在
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MiotTemplate {
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

impl FromStr for MiotTemplate {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let result: MiotTemplate = toml::from_str(s)?;
        Ok(result)
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DeviceTemplate {
    pub device_type: IotDeviceType,
    #[serde(default = "default_text")]
    pub tag: String,
    pub interval: Option<u64>,
    /// 展示名称
    #[serde(default)]
    pub display_name: Option<String>,
    pub timeout: Option<u64>,
    #[serde(default)]
    pub poll_properties: Vec<MiotSpecId>,
    pub desc: Option<String>,
    #[serde(default)]
    pub params: Option<DeviceParam>,
    /// 配件
    #[serde(default)]
    pub accessories: Vec<AccessoryTemplate>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AccessoryTemplate {
    /// 配件的类型
    pub category: BridgeCategory,
    #[serde(default = "default_text")]
    pub tag: String,

/*    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub model_params: Option<JsonValue>,*/
    #[serde(default)]
    pub desc: Option<String>,
    /// 配件的名称,默认取上一级
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub hap_delegates: Vec<ModelDelegateParam>,
    pub services: Vec<ServiceTemplate>,

}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ServiceTemplate {
    /// 配件的类型
    pub service_type: MappingHapType,

    pub characteristics: Vec<CharacteristicTemplate>,
    #[serde(default = "default_text")]
    pub tag: String,
    #[serde(default)]
    pub configured_name: Option<String>,
    #[serde(default)]
    pub desc: Option<String>,
    #[serde(default = "default_false")]
    pub primary: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CharacteristicTemplate {
    pub char_type: MappingHapType,
    #[serde(default)]
    pub info: HapCharInfoTemp,
    pub name: Option<String>,
    pub description: Option<String>,
}


#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize, Default)]
pub struct HapCharInfoTemp {
    pub format: Option<Format>,
    pub unit: Option<Unit>,
    pub min_value: Option<JsonValue>,
    pub max_value: Option<JsonValue>,
    pub step_value: Option<JsonValue>,
    pub max_len: Option<u16>,
    pub max_data_len: Option<u32>,
    pub valid_values: Option<Vec<JsonValue>>,
    pub valid_values_range: Option<Vec<JsonValue>>,
    pub ttl: Option<u64>,
    pub perms: Option<Vec<Perm>>,
    pub pid: Option<u64>,
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
    use crate::template::miot_template::MiotTemplate;

    #[tokio::test]
    pub async fn test() -> anyhow::Result<()> {
        let mut file = File::open("/Users/huanxi/project/homelink/templates/mihome/chuangmi.plug.212a01.toml").await.unwrap();
        let mut content = String::new();
        file.read_to_string(&mut content).await?;
        // SecuritySystemAccessory::new();
        // MappingHapType::SecuritySystem;
        // SecuritySystemCurrentStateCharacteristic::new();
        // SecuritySystemTargetStateCharacteristic::new();
        // LightbulbService::new();
        let result: MiotTemplate = toml::from_str(content.as_str()).unwrap();
        println!("{:?}", result);
        // let result: HashMap<String, Miot2HapMapper> = toml::from_str(content.as_str())?;
        // PowerStateCharacteristic::new();
        Ok(())
    }
}