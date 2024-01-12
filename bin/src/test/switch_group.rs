// this file is auto-generated by hap-hap_metadata

use hap::accessory::{AccessoryInformation, HapAccessory};
use hap::service::accessory_information::AccessoryInformationService;
use hap::service::HapService;
use hap::service::switch::SwitchService;
use serde::ser::{Serialize, SerializeStruct, Serializer};
use anyhow::Result;
use hap::characteristic::brightness::BrightnessCharacteristic;
use hap::characteristic::characteristic_value_active_transition_count::CharacteristicValueActiveTransitionCountCharacteristic;
use hap::characteristic::characteristic_value_transition_control::CharacteristicValueTransitionControlCharacteristic;
use hap::characteristic::name::NameCharacteristic;
use hap::HapType;

/// Switch accessory.
#[derive(Debug, Default)]
pub struct SwitchGroupAccessory {
    /// ID of the Switch accessory.
    id: u64,

    /// Accessory Information service.
    pub accessory_information: AccessoryInformationService,
    /// Switch service.
    pub switch: Vec<SwitchService>,
}

impl SwitchGroupAccessory {
    /// Creates a new Switch accessory.
    pub fn new(id: u64, information: AccessoryInformation) -> Result<Self> {
        let accessory_information = information.to_service(1, id)?;
        let switch_id = accessory_information.get_characteristics().len() as u64;
        let mut switch1 = SwitchService::new(1 + switch_id + 1, id);
        switch1.set_primary(true);

        let mut switch2 = SwitchService::new(1 + switch_id + 2 + 2, id);
        switch2.set_primary(true);

        // brightness: Some(BrightnessCharacteristic::new(id + 1 + 0 + 1, accessory_id)),
        // Some(CharacteristicValueActiveTransitionCountCharacteristic::new(id + 1 + 1 + 1, accessory_id)),
        // characteristic_value_transition_control: Some(CharacteristicValueTransitionControlCharacteristic::new(id + 1 + 2 + 1, accessory_id)),


        // switch2.name = Some(NameCharacteristic::new());
        // accessory_information.get_linked_services().push(switch2.get_id());

        Ok(Self {
            id,
            accessory_information,
            switch: vec![switch1, switch2],
        })
    }
}

impl HapAccessory for SwitchGroupAccessory {
    fn get_id(&self) -> u64 {
        self.id
    }

    fn set_id(&mut self, id: u64) {
        self.id = id;
    }

    fn get_service(&self, hap_type: HapType) -> Option<&dyn HapService> {
        for service in self.get_services() {
            if service.get_type() == hap_type {
                return Some(service);
            }
        }
        None
    }

    fn get_mut_service(&mut self, hap_type: HapType) -> Option<&mut dyn HapService> {
        for service in self.get_mut_services() {
            if service.get_type() == hap_type {
                return Some(service);
            }
        }
        None
    }

    fn get_services(&self) -> Vec<&dyn HapService> {
        vec![
            &self.accessory_information,
            &self.switch[0],
            &self.switch[1],
        ]
    }

    fn get_mut_services(&mut self) -> Vec<&mut dyn HapService> {
        vec![
            &mut self.accessory_information,
            &mut self.switch[0],
        ]
    }
}

impl Serialize for SwitchGroupAccessory {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("HapAccessory", 2)?;
        state.serialize_field("aid", &self.get_id())?;
        state.serialize_field("services", &self.get_services())?;
        state.end()
    }
}
