use std::collections::HashMap;
use std::sync::Arc;
use dashmap::DashMap;
use hap::accessory::HapAccessory;
use hap::HapType;
use hap::service::accessory_information::AccessoryInformationService;
use hap::service::HapService;
use serde::{Serialize, Serializer};
use serde::ser::SerializeStruct;
use tokio::sync::Mutex;
use miot_spec::device::miot_spec_device::MiotSpecDevice;
use crate::convertor::iot_hap_service::IotHapService;

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

pub struct IotHapAccessory {
    /// ID of the Switch accessory.
    id: u64,
    // pub device: Arc<dyn MiotSpecDevice + Send + Sync>,
    /// Accessory Information service.
    // pub accessory_information: AccessoryInformationService,
    /// Switch service.
    pub services: HashMap<u64, Box<dyn HapService>>,

}

impl IotHapAccessory {
    pub fn new(id: u64, scvs: Vec<Box<dyn HapService>>) -> Self {
        let mut services = HashMap::new();
        scvs.into_iter().for_each(|i| {
            services.insert(i.get_id(), i);
        });

        Self {
            id,
            services,
        }
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
    fn get_id_mut_service(&mut self, id: u64) -> Option<&mut dyn HapService> {
        self.services.get_mut(&id).map(|i| i.as_mut() as &mut dyn HapService)
    }
     fn push_service(&mut self, service: Box<dyn HapService>) {
        self.services.insert(service.get_id(), service);
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
}

impl Serialize for IotHapAccessory {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("HapAccessory", 2)?;
        state.serialize_field("aid", &self.get_id())?;
        state.serialize_field("services", &self.get_services())?;
        state.end()
    }
}
