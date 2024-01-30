use std::collections::HashMap;
use std::sync::Arc;
use hap::accessory::HapAccessory;
use hap::HapType;
use hap::service::HapService;
use serde::{Serialize, Serializer};
use serde::ser::SerializeStruct;
use tokio::sync::Mutex;
use miot_spec::device::miot_spec_device::MiotSpecDevice;
use crate::hap::models::{AccessoryModel, AccessoryModelExt, AccessoryModelExtPointer};

/// 一个设备可能存在多个配件
/// 一个配件多个服务，一个服务多个特征值
pub struct IotDeviceAccessory {
    pub device: Arc<dyn MiotSpecDevice + Send + Sync>,
    pub accessories: Vec<Arc<Mutex<Box<dyn HapAccessory>>>>,
}

impl IotDeviceAccessory {
    pub fn new(device: Arc<dyn MiotSpecDevice + Send + Sync>) -> Self {
        Self {
            device,
            accessories: vec![],
        }
    }
}

pub struct TagsIdMap {
    map: HashMap<String, Vec<u64>>,
}

/*impl TagsIdMap {
    pub fn new() -> Self {
        Self {
            map: Default::default(),
        }
    }
    pub fn push(&mut self, tag: String, id: u64) {
        let ids = self.map.entry(tag).or_insert(vec![]);
        ids.push(id);
    }
}*/

pub struct IotHapAccessory {
    /// ID of the Switch accessory.
    id: u64,
    // pub device: Arc<dyn MiotSpecDevice + Send + Sync>,
    /// Accessory Information service.
    // pub accessory_information: AccessoryInformationService,
    /// Switch service.
    pub services: HashMap<u64, Box<dyn HapService>>,
    pub tag_ids_map: HashMap<String, Vec<u64>>,
    pub model_ext: Option<AccessoryModel>,
}

impl IotHapAccessory {
    pub fn new(id: u64, scv_list: Vec<Box<dyn HapService>>, model_ext: Option<AccessoryModel>) -> Self {
        let mut services = HashMap::new();
        scv_list.into_iter().for_each(|i| {
            services.insert(i.get_id(), i);
        });

        Self {
            id,
            services,
            tag_ids_map: Default::default(),
            model_ext,
        }
    }
    pub fn on_read() {

    }

}

impl HapAccessory for IotHapAccessory {
    fn get_id(&self) -> u64 {
        self.id
    }

    fn set_id(&mut self, id: u64) {
        self.id = id
    }

    fn get_service(&self, hap_type: HapType) -> Option<&dyn HapService> {
        todo!()
    }
    fn get_mut_service(&mut self, hap_type: HapType) -> Option<&mut dyn HapService> {
        todo!()
    }
    fn get_services(&self) -> Vec<&dyn HapService> {
        self.services.iter().map(|i| i.1.as_ref()).collect()
    }

    fn get_mut_services<'a>(&'a mut self) -> Vec<&'a mut dyn HapService> {
        self.services.values_mut().map(|i| i.as_mut() as (&mut dyn HapService )).collect()
    }

    fn push_service(&mut self, tag: Option<String>, service: Box<dyn HapService>) {
        if let Some(tag) = tag {
            self.tag_ids_map.entry(tag)
                .or_insert(vec![])
                .push(service.get_id())
        };
        self.services.insert(service.get_id(), service);
    }

    fn get_mut_services_by_tag(&mut self, tag: &str) -> Vec<&mut dyn HapService> {
        let ids = self.tag_ids_map.get(tag);
        if let Some(ids) = ids {
            return self.services
                .values_mut()
                .filter(|i| ids.contains(&i.get_id()))
                .map(|i| i.as_mut() as (&mut dyn HapService )).collect();
        }
        vec![]
    }
    fn get_mut_service_by_id(&mut self, id: u64) -> Option<&mut dyn HapService> {
        self.services.get_mut(&id).map(|i| i.as_mut() as &mut dyn HapService)
    }
}

impl Serialize for IotHapAccessory {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("HapAccessory", 2)?;
        state.serialize_field("aid", &self.get_id())?;
        state.serialize_field("services", &self.get_services())?;
        state.end()
    }
}
