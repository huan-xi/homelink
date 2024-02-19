use log::error;
use serde::{Deserialize, Serialize, Serializer};
use serde_json::json;
use hap::characteristic::Format;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct CharacteristicValue {
    pub value: serde_json::Value,
}

impl CharacteristicValue {
    pub fn new(value: serde_json::Value) -> Self {
        Self { value }
    }
    /// string
    pub fn format(format: Format, value: serde_json::Value) -> Self {
        Self::try_format(format, value).unwrap_or_else(|e| {
            error!("format error:{:?}", e);
            Self::new(json!(""))
        })
    }
    pub fn try_format(format: Format, value: serde_json::Value) -> anyhow::Result<Self> {
        let value = if value.is_string() && format != Format::String {
            match format {
                Format::Bool => {
                    let v = value.as_str().unwrap().parse::<bool>()?;
                    json!(v)
                }
                Format::Int32 => {
                    let v = value.as_str().unwrap().parse::<i32>()?;
                    json!(v)
                }
                Format::UInt8 => {
                    let v = value.as_str().unwrap().parse::<u8>()?;
                    json!(v)
                }
                Format::UInt16 => {
                    let v = value.as_str().unwrap().parse::<u16>()?;
                    json!(v)
                }
                Format::UInt32 => {
                    let v = value.as_str().unwrap().parse::<u32>()?;
                    json!(v)
                }

                Format::Float => {
                    let v = value.as_str().unwrap().parse::<f32>()?;
                    json!(v)
                }
                _ => { value }
            }
        } else if format == Format::Bool && value.is_number() {
            let val = value.as_u64() == Some(1);
            json!(val)
        } else { value };

        Ok(Self {
            value
        })
    }
}


impl Into<serde_json::Value> for CharacteristicValue {
    fn into(self) -> serde_json::Value {
        self.value
    }
}

impl<'a> Deserialize<'a> for CharacteristicValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'a> {
        let value = serde_json::Value::deserialize(deserializer)?;
        Ok(CharacteristicValue { value })
    }
}

impl Serialize for CharacteristicValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        self.value.serialize(serializer)
    }
}
