use hap::accessory::{AccessoryInformation, HapAccessory};
use hap::accessory::lightbulb::LightbulbAccessory;
use hap::accessory::switch::SwitchAccessory;
use hap::characteristic::AsyncCharacteristicCallbacks;
use miot_spec::device::miot_spec_device::MiotSpecDevice;
use crate::convertor::config::MappingConfig;
use crate::convertor::miot2hap::Utils;

pub struct SwitchMapper {}

impl SwitchMapper{
    pub fn map_to_accessory(device: &dyn MiotSpecDevice, id: u64, config: MappingConfig) -> anyhow::Result<SwitchAccessory> {
        todo!()
    /*    let info = device.get_info().clone();
        let mut accessory = SwitchAccessory::new(
            id,
            AccessoryInformation {
                name: info.name.clone(),
                ..Default::default()
            },
        ).unwrap();
        let proto = device.get_proto();

        // 电源控制属性
        match config.power_state {
            None => {
                todo!("不支持无电源灯")
            }
            Some(ps) => {
                // let ptc = proto.clone();
                let id = info.did.clone();
                let func = Utils::get_set_func(id.clone(), proto.clone(), ps.clone());
                accessory.switch.power_state.on_update_async(Some(func));
                let func = Utils::get_read_func(id.clone(), proto.clone(), ps.clone(), |v| v.as_bool());
                accessory.switch.power_state.on_read_async(Some(func));
            }
        }

        Ok(accessory)*/
    }
}