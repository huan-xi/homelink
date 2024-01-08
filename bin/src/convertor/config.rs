use std::iter::Map;
use hap::accessory::AccessoryCategory;
use hap::HapType;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::convertor::hap_type::MappingHapType;
use crate::db::entity::hap_characteristic::Property;

/*#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Property {
    pub siid: i32,
    pub piid: i32,
    pub min_value: Option<Value>,
    pub max_value: Option<Value>,
    pub step: Option<Value>,
    //单位
}*/

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MappingConfig {
    /// 对应的hap 类型
    pub hap_type: MappingHapType,
    /// 电源的映射
    pub power_state: Option<Property>,
    /// 亮度的映射
    pub brightness: Option<Property>,
    /// 色温映射
    pub color_temperature: Option<Property>,
    /// 温度读取属性
    pub status_tampered: Option<Property>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Miot2HapMapper {
    pub mappers: Vec<MappingConfig>,
}