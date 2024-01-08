use hap::accessory::AccessoryInformation;
use hap::accessory::lightbulb::LightbulbAccessory;
use hap::characteristic::{AsyncCharacteristicCallbacks, HapCharacteristic};
use log::info;
use serde_json::Value;
use miot_spec::device::miot_spec_device::MiotSpecDevice;
use crate::convertor::config::MappingConfig;

#[derive(Default, Copy, Clone)]
pub struct LightbulbMapper {}

impl LightbulbMapper {
    pub async fn map_to_accessory(device: &dyn MiotSpecDevice, id: u64, config: MappingConfig) -> anyhow::Result<LightbulbAccessory> {
        let proto = device.get_proto().await.unwrap();
        let info = device.get_info().clone();

        let mut accessory = LightbulbAccessory::new(
            id,
            AccessoryInformation {
                name: info.name.clone(),
                ..Default::default()
            },
        ).unwrap();

        // 电源控制属性
        match config.power_state {
            None => {
                return Err(anyhow::anyhow!("不支持无电源灯"));
            }
            Some(ps) => {
                // let ptc = proto.clone();
                let id = info.did.clone();
                let func = crate::convertor::miot2hap::Utils::get_set_func_1(id.clone(), proto.clone(), ps.clone());
                accessory.lightbulb.power_state.on_update_async(Some(func));
                let func = crate::convertor::miot2hap::Utils::get_read_func_1(id.clone(), proto.clone(), ps.clone(), |v| v.as_bool());
                accessory.lightbulb.power_state.on_read_async(Some(func));
            }
        }

        // 亮度设置
        match config.brightness {
            None => {
                accessory.lightbulb.brightness = None;
            }
            Some(ps) => {
                let id = info.did.clone();
                let func = crate::convertor::miot2hap::Utils::get_set_func_1(id.clone(), proto.clone(), ps.clone());
                accessory.lightbulb.brightness.as_mut().unwrap().on_update_async(Some(func));
                let func = crate::convertor::miot2hap::Utils::get_read_func_1(id.clone(), proto.clone(), ps.clone(), |value| value.as_i64().map(|v| v as i32));
                accessory.lightbulb.brightness.as_mut().unwrap().on_read_async(Some(func));
            }
        }

        //色温设置
        match config.color_temperature {
            None => {
                accessory.lightbulb.color_temperature = None;
            }
            Some(ps) => {
          /*      fn kelvin_to_mired(kelvin: i32) -> i32 {
                    (1_000_000.0 / kelvin as f64) as i32
                }
                fn kelvin_to_mired_json(kelvin: Option<Value>) -> Option<Value> {
                    kelvin.map_or(None, |k| k.as_i64()
                        .map(|k1| Value::from(kelvin_to_mired(k1 as i32))))
                }

                fn mired_to_kelvin(mired: i32) -> i32 {
                    (1_000_000.0 / mired as f64) as i32
                }

                let conv = |value: Value| {
                    // kelvin 转 Mirek
                    value.as_i64().map(|v| kelvin_to_mired(v as i32))
                };
                let id = info.did.clone();
                let func = crate::convertor::miot2hap::Utils::get_set_func_conv(id.clone(), proto.clone(), ps.clone(),
                                                                                |v| { Value::from(mired_to_kelvin(v)) });

                let color_temperature = accessory.lightbulb.color_temperature.as_mut().unwrap();
                color_temperature.on_update_async(Some(func));

                let func = crate::convertor::miot2hap::Utils::get_read_func(id.clone(), proto.clone(), ps.clone(), conv);
                color_temperature.on_read_async(Some(func));

                let min = kelvin_to_mired_json(ps.max_value.clone());
                let max = kelvin_to_mired_json(ps.min_value.clone());
                let step = kelvin_to_mired_json(ps.step.clone());

                info!("range {:?}~{:?} step:{:?}",min,max,step);
                //todo 最大值不能小于500?
                //todo 最大值-最小值不能<360?
                //range Some(Number(153))~Some(Number(370)) step:Some(Number(1000000))
                color_temperature.set_max_value(Some(Value::from(500))).unwrap();
                color_temperature.set_min_value(Some(Value::from(140))).unwrap();
                // color_temperature.set_step_value(step).unwrap();
                accessory.lightbulb.hue = None;
                accessory.lightbulb.saturation = None;*/
                todo!();
            }
        }

        Ok(accessory)
    }
}