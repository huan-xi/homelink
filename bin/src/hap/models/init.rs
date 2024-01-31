use std::sync::Arc;
use dashmap::DashMap;
use crate::hap::models::{AccessoryModelExtPointer, deerma, lumi};

impl Default for super::AccessoryModelExtDatabase {
    fn default() -> Self {
        let model_map: DashMap<String, AccessoryModelExtPointer> = DashMap::new();
        model_map.insert("lumi.acpartner.vmcn02".to_string(), Arc::new(lumi::lumi_acpartner_mcn02::ModelExt::default()));
        model_map.insert("lumi.gateway.mgl03".to_string(), Arc::new(lumi::lumi_gateway_mgl03::ModelExt::default()));
        model_map.insert("deerma.humidifier.jsq3".to_string(), Arc::new(deerma::deerma_humidifier_jsq3::ModelExt::default()));
        Self {
            model_map,
        }
    }
}
