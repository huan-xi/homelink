use std::sync::Arc;
use dashmap::DashMap;
use once_cell::sync::OnceCell;
use sea_orm::JsonValue;
use crate::hap::models::{AccessoryModelExtConstructor, AccessoryModelExtPointer, common, ContextPointer, deerma, lumi};

pub static MODEL_EXT_DATABASE: OnceCell<AccessoryModelExtDatabase> = OnceCell::new();

pub type AccessoryModelExtConstructorFunc = fn(ContextPointer, Option<JsonValue>) -> anyhow::Result<AccessoryModelExtPointer>;

pub struct AccessoryModelExtDatabase {
    pub(crate) model_map: DashMap<String, AccessoryModelExtConstructorFunc>,
}

impl AccessoryModelExtDatabase {
    pub fn get(&self, name: &str) -> Option<AccessoryModelExtConstructorFunc> {
        self.model_map.get(name).map(|v| v.value().clone())
    }
/*    pub fn insert(&self, name: &str, ext: AccessoryModelExtPointer) {
        self.model_map.insert(name.to_string(), ext);
    }*/
}


impl Default for AccessoryModelExtDatabase {
    fn default() -> Self {
        let model_map: DashMap<String, AccessoryModelExtConstructorFunc> = DashMap::new();
        model_map.insert("common.mode_switch".to_string(), common::mode_switch::ModelExt::new);
        model_map.insert("lumi.acpartner.vmcn02".to_string(), lumi::lumi_acpartner_mcn02::ModelExt::new);
        model_map.insert("lumi.gateway.mgl03".to_string(), lumi::lumi_gateway_mgl03::ModelExt::new);
        model_map.insert("deerma.humidifier.jsq3".to_string(),deerma::deerma_humidifier_jsq3::ModelExt::new);
        Self {
            model_map,
        }
    }
}
