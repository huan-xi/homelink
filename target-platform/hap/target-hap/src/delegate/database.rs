use once_cell::sync::OnceCell;
use std::sync::Arc;
use dashmap::DashMap;
use dashmap::mapref::entry::Entry;
use crate::delegate::model::{ContextPointer, HapModelExtPointer};


pub static MODEL_EXT_DATABASE: OnceCell<AccessoryModelExtDatabase> = OnceCell::new();

pub type AccessoryModelExtConstructorFunc = fn(ContextPointer, Option<serde_json::Value>) -> anyhow::Result<HapModelExtPointer>;

#[derive(Default)]
pub struct AccessoryModelExtDatabase {
    pub model_map: DashMap<String, AccessoryModelExtConstructorFunc>,
}

impl AccessoryModelExtDatabase {
    pub fn get(&self, name: &str) -> Option<AccessoryModelExtConstructorFunc> {
        self.model_map.get(name).map(|v| v.value().clone())
    }
    pub fn insert(&self, name: String, func: AccessoryModelExtConstructorFunc) -> anyhow::Result<()> {
        match self.model_map.entry(name.clone()) {
            Entry::Occupied(_) => {
                return Err(anyhow::anyhow!("AccessoryModelExt {} already exists", name));
            }
            Entry::Vacant(e) => {
                e.insert(func);
            }
        };
        Ok(())
    }
}


pub fn get_hap_model_ext_database() -> &'static AccessoryModelExtDatabase {
    MODEL_EXT_DATABASE.get_or_init(|| AccessoryModelExtDatabase::default())
}