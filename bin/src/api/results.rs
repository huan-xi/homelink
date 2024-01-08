use crate::db::entity::prelude::{HapAccessoryModel, HapBridge, HapBridgeModel, IotDeviceModel};

#[derive(Debug, serde::Serialize)]
pub struct HapAccessoryResult {
    #[serde(flatten)]
    pub model: HapAccessoryModel,
    pub bridge: Option<HapBridgeModel>,
    pub device: Option<IotDeviceModel>,
}
