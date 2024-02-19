use std::sync::Arc;
use dashmap::DashMap;
use once_cell::sync::OnceCell;
use sea_orm::JsonValue;
use crate::hap::models::{AccessoryModelExtConstructor, AccessoryModelExtPointer, common, ContextPointer, deerma, lumi};



impl Default for AccessoryModelExtDatabase {
    fn default() -> Self {
        let model_map: DashMap<String, AccessoryModelExtConstructorFunc> = DashMap::new();
        model_map.insert("common.mode_switch".to_string(), common::mode_switch::ModelExt::new);
        model_map.insert("common.miot_spec_prop_mapping".to_string(), common::miot_spec_prop_mapping::ModelExt::new);
        model_map.insert("common.native_ble".to_string(), common::native_ble::ModelExt::new);
        model_map.insert("lumi.acpartner.vmcn02".to_string(), lumi::lumi_acpartner_mcn02::ModelExt::new);
        model_map.insert("lumi.gateway.mgl03".to_string(), lumi::lumi_gateway_mgl03::ModelExt::new);
        model_map.insert("deerma.humidifier.jsq3".to_string(),deerma::deerma_humidifier_jsq3::ModelExt::new);
        Self {
            model_map,
        }
    }
}
