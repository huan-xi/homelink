use serde::{Deserialize, Serialize};
use crate::db::entity::common::Property;
use crate::hap::hap_type::MappingHapType;

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