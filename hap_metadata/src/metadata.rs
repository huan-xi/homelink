use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::utils;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetadata {
    #[serde(rename = "Version")]
    pub version: usize,
    #[serde(rename = "SchemaVersion")]
    pub schema_version: usize,
    #[serde(rename = "PlistDictionary")]
    pub plist_dictionary: SystemPlistDictionary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemPlistDictionary {
    #[serde(rename = "Version")]
    pub version: usize,
    #[serde(rename = "SchemaVersion")]
    pub schema_version: usize,
    #[serde(rename = "HomeKit")]
    pub homekit: HomeKit,
    #[serde(rename = "HAP")]
    pub hap: Hap,
    #[serde(rename = "Assistant")]
    pub assistant: Assistant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HomeKit {
    #[serde(rename = "Categories")]
    pub categories: HashMap<String, HomeKitCategory>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HomeKitCategory {
    #[serde(rename = "DefaultDescription")]
    pub name: String,
    #[serde(rename = "Identifier")]
    pub number: u8,
    #[serde(rename = "UUID")]
    pub uuid: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hap {
    #[serde(rename = "Base UUID")]
    pub base_uuid: String,
    #[serde(rename = "Characteristics")]
    pub characteristics: HashMap<String, HapCharacteristic>,
    #[serde(rename = "Services")]
    pub services: HashMap<String, HapService>,
    #[serde(rename = "Properties")]
    pub properties: HashMap<String, HapProperty>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HapCharacteristic {
    #[serde(rename = "ShortUUID")]
    pub short_uuid: String,
    #[serde(rename = "DefaultDescription")]
    pub name: String,
    #[serde(rename = "Format")]
    pub format: String,
    #[serde(rename = "MinValue")]
    pub min_value: Option<Value>,
    #[serde(rename = "MaxValue")]
    pub max_value: Option<Value>,
    #[serde(rename = "StepValue")]
    pub step_value: Option<Value>,
    #[serde(rename = "MaxLength")]
    pub max_length: Option<Value>,
    #[serde(rename = "Units")]
    pub units: Option<String>,
    #[serde(rename = "Properties")]
    pub properties: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HapService {
    #[serde(rename = "ShortUUID")]
    pub short_uuid: String,
    #[serde(rename = "DefaultDescription")]
    pub name: String,
    #[serde(rename = "Characteristics")]
    pub characteristics: HapServiceCharacteristicRelation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HapServiceCharacteristicRelation {
    #[serde(rename = "Required")]
    pub required_characteristics: Vec<String>,
    #[serde(rename = "Optional")]
    pub optional_characteristics: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HapProperty {
    #[serde(rename = "DefaultDescription")]
    pub name: String,
    #[serde(rename = "Position")]
    pub number: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Assistant {
    #[serde(rename = "Characteristics")]
    pub characteristics: HashMap<String, AssistantCharacteristic>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssistantCharacteristic {
    #[serde(rename = "Format")]
    pub format: String,
    #[serde(rename = "Read")]
    pub read: Option<String>,
    #[serde(rename = "Write")]
    pub write: Option<String>,
    #[serde(rename = "ReadWrite")]
    pub read_write: Option<String>,
    #[serde(rename = "Values")]
    pub values: Option<HashMap<String, Value>>,
    #[serde(rename = "OutValues")]
    pub out_values: Option<HashMap<String, Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HapMetadata {
    pub categories: HashMap<String, HomeKitCategory>,
    pub sorted_categories: Vec<HomeKitCategory>,
    pub characteristics: HashMap<String, HapCharacteristic>,
    pub sorted_characteristics: Vec<HapCharacteristic>,
    pub services: HashMap<String, HapService>,
    pub sorted_services: Vec<HapService>,
    pub properties: HashMap<String, HapProperty>,
    pub assistant_characteristics: HashMap<String, AssistantCharacteristic>,
    pub characteristic_in_values: HashMap<String, HashMap<String, Value>>,
    pub characteristic_out_values: HashMap<String, HashMap<String, Value>>,
}

impl From<SystemMetadata> for HapMetadata {
    fn from(v: SystemMetadata) -> Self {
        let mut m = v.plist_dictionary;

        // rename mislabeled services
        let  accessory_information_service = m.hap.services.get_mut("accessory-information").unwrap();
        accessory_information_service.name = "Accessory Information".to_string();
        let  fan_v2_service = m.hap.services.get_mut("fanv2").unwrap();
        fan_v2_service.name = "Fan v2".to_string();
        let  smart_speaker_service = m.hap.services.get_mut("smart-speaker").unwrap();
        smart_speaker_service.name = "Smart Speaker".to_string();

        let mut sorted_categories = m.homekit.categories.iter().map(|(_, v)| v.clone()).collect::<Vec<_>>();
        sorted_categories.sort_by(|a, b| a.number.partial_cmp(&b.number).unwrap());

        let mut sorted_characteristics = m.hap.characteristics.iter().map(|(_, v)| v.clone()).collect::<Vec<_>>();
        sorted_characteristics.sort_by(|a, b| a.name.cmp(&b.name));

        let mut sorted_services = m.hap.services.iter().map(|(_, v)| v.clone()).collect::<Vec<_>>();
        sorted_services.sort_by(|a, b| a.name.cmp(&b.name));

        let mut characteristic_in_values = HashMap::new();
        let mut characteristic_out_values = HashMap::new();

        for (_, characteristic) in m.assistant.characteristics.clone() {
            if let (Some(ref read_name), Some(ref values), &None) =
                (&characteristic.read, &characteristic.values, &characteristic.out_values)
            {
                characteristic_in_values.insert(read_name.clone(), values.clone());
            }

            if let (Some(ref read_write_name), Some(ref values), &None) = (
                &characteristic.read_write,
                &characteristic.values,
                &characteristic.out_values,
            ) {
                characteristic_in_values.insert(read_write_name.clone(), values.clone());
            }

            if let (Some(read_name), Some(out_values)) = (characteristic.read, characteristic.out_values) {
                characteristic_out_values.insert(read_name, out_values);
            }

            if let (Some(write_name), Some(values)) = (characteristic.write, characteristic.values) {
                characteristic_in_values.insert(write_name, values);
            }
        }
     /*   let mut characteristics = HashMap::new();
        m.hap.characteristics.values().for_each(|c| {
            //名称
            let key = utils::pascal_case(c.name.as_str());
            characteristics.insert(key, c.clone());
        });*/
        let mut services = HashMap::new();
        m.hap.services.values().for_each(|c| {
            //名称
            let key = utils::pascal_case(c.name.as_str());
            services.insert(key, c.clone());
        });

        Self {
            categories: m.homekit.categories,
            sorted_categories,
            characteristics:m.hap.characteristics,
            sorted_characteristics,
            services,
            sorted_services,
            properties: m.hap.properties,
            assistant_characteristics: m.assistant.characteristics,
            characteristic_in_values,
            characteristic_out_values,
        }
    }
}