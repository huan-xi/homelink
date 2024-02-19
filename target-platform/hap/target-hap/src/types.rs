use impl_new::New;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use hap::characteristic::{Format, Perm, Unit};
use hap::HapType;
use hap_metadata::metadata::Hap;
use hl_integration::JsonValue;
use crate::hap_type_wrapper::HapTypeWrapper;

// #[derive(Debug, Clone)]
// pub struct CharReadParam {
//     pub sid: u64,
//     pub stag: String,
//     pub cid: u64,
//     pub ctag: HapType,
// }

/// 定位char
#[derive(Eq, PartialEq, Hash, Debug, Clone, New, Serialize, Deserialize)]
pub struct CharIdentifier {
    pub stag: String,
    pub ctag: HapTypeWrapper,
}

impl From<&hap::characteristic::delegate::CharReadParam> for CharIdentifier {
    fn from(value: &hap::characteristic::delegate::CharReadParam) -> Self {
        CharIdentifier {
            stag: value.stag.clone(),
            ctag: value.ctag.into(),
        }
    }
}


#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelDelegateParam {
    pub chars: Vec<CharIdentifier>,
    ///配件模型 接管读写事件
    pub model: String,
    /// 模型 运行时参数
    pub params: Option<JsonValue>,
}


/// hap_platform 的一些属性
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct HapCharInfo {
    pub format: Format,
    pub unit: Option<Unit>,
    pub min_value: Option<JsonValue>,
    pub max_value: Option<JsonValue>,
    pub step_value: Option<JsonValue>,
    pub max_len: Option<u16>,
    pub max_data_len: Option<u32>,
    pub valid_values: Option<Vec<JsonValue>>,
    pub valid_values_range: Option<Vec<JsonValue>>,
    pub ttl: Option<u64>,
    #[serde(default)]
    pub perms: Vec<Perm>,
    pub pid: Option<u64>,
}



