use std::collections::HashMap;

use serde::{Serialize, Serializer};
use serde::ser::SerializeStruct;

use hap::characteristic::HapCharacteristic;
use hap::HapType;
use hap::service::HapService;

pub struct IotHapService {
    /// Instance ID of the Switch service.
    id: u64,
    /// [`HapType`](HapType) of the Switch service.
    hap_type: HapType,
    /// When set to true, this service is not visible to user.
    hidden: bool,
    /// When set to true, this is the primary service on the accessory.
    primary: bool,
    /// An array of numbers containing the instance IDs of the services that this service links to.
    linked_services: Vec<u64>,
    /// Power State characteristic (required).
    // pub power_state: PowerStateCharacteristic,
    // pub name: Option<NameCharacteristic>,
    characteristic_map: HashMap<u64, Box<dyn HapCharacteristic>>,
    tag: Option<String>,
}

impl IotHapService {
    pub fn new(id: u64, accessory_id: u64, hap_type: HapType, tag: Option<String>,) -> Self {
        let mut characteristic_map = HashMap::new();
        Self {
            id,
            hap_type,
            hidden: false,
            primary: false,
            linked_services: vec![],
            characteristic_map,
            tag,
        }
    }
    pub fn push_characteristic(&mut self, characteristic: Box<dyn HapCharacteristic>) {
        let id = characteristic.get_id();
        self.characteristic_map.insert(id, characteristic);
    }
}

impl HapService for IotHapService {
    fn get_id(&self) -> u64 {
        self.id
    }
    fn get_tag(&self) -> Option<String> {
        self.tag.clone()
    }

    fn set_id(&mut self, id: u64) {
        self.id = id;
    }

    fn get_type(&self) -> HapType {
        self.hap_type
    }

    fn set_type(&mut self, hap_type: HapType) {
        self.hap_type = hap_type;
    }

    fn get_hidden(&self) -> bool {
        self.hidden
    }

    fn set_hidden(&mut self, hidden: bool) {
        self.hidden = hidden;
    }

    fn get_primary(&self) -> bool {
        self.primary
    }

    fn set_primary(&mut self, primary: bool) {
        self.primary = primary;
    }

    fn get_linked_services(&self) -> Vec<u64> {
        self.linked_services.clone()
    }

    fn set_linked_services(&mut self, linked_services: Vec<u64>) {
        self.linked_services = linked_services;
    }

    fn get_characteristic(&self, hap_type: HapType) -> Option<&dyn HapCharacteristic> {
        for characteristic in self.get_characteristics() {
            if characteristic.get_type() == hap_type {
                return Some(characteristic);
            }
        }
        None
    }

    fn get_mut_characteristic(&mut self, hap_type: HapType) -> Option<&mut dyn HapCharacteristic> {
        for characteristic in self.get_mut_characteristics() {
            if characteristic.get_type() == hap_type {
                return Some(characteristic);
            }
        }
        None
    }

    fn get_characteristic_by_id(&self, id: u64) -> Option<&dyn HapCharacteristic> {
        self.characteristic_map.get(&id).map(|i| i.as_ref() as &dyn HapCharacteristic)
    }

    fn get_mut_characteristic_by_id(&mut self, id: u64) -> Option<&mut dyn HapCharacteristic> {
        self.characteristic_map.get_mut(&id).map(|i| i.as_mut() as &mut dyn HapCharacteristic)
    }
    fn get_characteristics(&self) -> Vec<&dyn HapCharacteristic> {
        self.characteristic_map.iter().map(|i| i.1.as_ref()).collect()
    }

    fn get_mut_characteristics(&mut self) -> Vec<&mut dyn HapCharacteristic> {
        self.characteristic_map.iter_mut().map(|mut i| i.1.as_mut() as &mut dyn HapCharacteristic).collect()
    }

}

impl Serialize for IotHapService {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("HapService", 5)?;
        state.serialize_field("iid", &self.get_id())?;
        state.serialize_field("type", &self.get_type())?;
        state.serialize_field("hidden", &self.get_hidden())?;
        state.serialize_field("primary", &self.get_primary())?;
        state.serialize_field("characteristics", &self.get_characteristics())?;
        // state.serialize_field("linked", &self.get_linked_services())?;
        // linked services left out for now
        state.end()
    }
}
