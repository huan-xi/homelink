use packed_struct::derive::PackedStruct;
use serde_json::Value;
use tap::Tap;

#[derive(Debug)]
pub enum BleValueType {
    Temperature,
    Humidity,
    Battery,
    ContactValue,
}

#[derive(Default, Debug, Clone)]
pub struct BleValue {
    pub temperature: Option<TemperatureValue>,
    pub humidity: Option<HumidityValue>,
    pub contact: Option<ContactValue>,
    pub battery: Option<u8>,
}

impl BleValue {
    pub fn get_value(&self, value_type: BleValueType) -> Option<Value> {
        match value_type {
            BleValueType::Temperature => {
                self.temperature.as_ref().map(|v| Value::from(v.value))
            }
            BleValueType::Humidity => {
                self.humidity.as_ref().map(|v| Value::from(v.value))
            }
            BleValueType::Battery => {
                self.battery.as_ref().map(|v| Value::from(*v))
            }
            BleValueType::ContactValue => {
                self.contact.as_ref().map(|v| Value::from(v.value))
            }
        }.tap(|v| {
            log::debug!("get_value: {:?} {:?}", value_type, v);
        })
    }
}


///温度值
#[derive(PackedStruct, Debug, Clone)]
#[packed_struct(endian = "lsb")]
pub struct TemperatureValue {
    pub value: i16,
}

#[derive(PackedStruct, Debug, Clone)]
#[packed_struct(endian = "lsb")]
pub struct HumidityValue {
    pub value: i16,
}

#[derive(PackedStruct, Debug, Clone)]
#[packed_struct(endian = "lsb")]
pub struct ContactValue {
    pub value: i16,
}
