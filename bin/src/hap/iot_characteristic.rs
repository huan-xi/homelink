use axum::async_trait;
use hap::characteristic::{AsyncCharacteristicCallbacks, Characteristic, CharacteristicCallbacks, Format, HapCharacteristic, HapCharacteristicSetup, OnReadFn, OnReadFuture, OnUpdateFn, OnUpdateFuture, Perm, Unit};
use hap::{HapType, pointer};
use hap::characteristic::brightness::BrightnessCharacteristic;
use hap::characteristic::color_temperature::ColorTemperatureCharacteristic;
use hap::characteristic::contact_sensor_state::ContactSensorStateCharacteristic;
use hap::characteristic::current_temperature::CurrentTemperatureCharacteristic;
use hap::characteristic::hue::HueCharacteristic;
use hap::characteristic::power_state::PowerStateCharacteristic;
use hap::characteristic::saturation::SaturationCharacteristic;
use hap::service::lightbulb::LightbulbService;
use serde::{Deserialize, Serialize, Serializer};
use serde_json::json;
use hap::characteristic::active::ActiveCharacteristic;
use hap::characteristic::cooling_threshold_temperature::CoolingThresholdTemperatureCharacteristic;
use hap::characteristic::current_fan_state::CurrentFanStateCharacteristic;
use hap::characteristic::current_heater_cooler_state::CurrentHeaterCoolerStateCharacteristic;
use hap::characteristic::current_heating_cooling_state::CurrentHeatingCoolingStateCharacteristic;
use hap::characteristic::heating_threshold_temperature::HeatingThresholdTemperatureCharacteristic;
use hap::characteristic::lock_physical_controls::LockPhysicalControlsCharacteristic;
use hap::characteristic::outlet_in_use::OutletInUseCharacteristic;
use hap::characteristic::rotation_speed::RotationSpeedCharacteristic;
use hap::characteristic::swing_mode::SwingModeCharacteristic;
use hap::characteristic::target_fan_state::TargetFanStateCharacteristic;
use hap::characteristic::target_heater_cooler_state::TargetHeaterCoolerStateCharacteristic;
use hap::characteristic::target_heating_cooling_state::TargetHeatingCoolingStateCharacteristic;
use hap::characteristic::target_temperature::TargetTemperatureCharacteristic;
use hap::characteristic::temperature_display_units::TemperatureDisplayUnitsCharacteristic;
use crate::hap::hap_type::MappingHapType;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct CharacteristicValue {
    pub(crate) value: serde_json::Value,
}

impl CharacteristicValue {
    pub fn new(value: serde_json::Value) -> Self {
        Self { value }
    }
}


impl Into<serde_json::Value> for CharacteristicValue {
    fn into(self) -> serde_json::Value {
        self.value
    }
}

impl<'a> Deserialize<'a> for CharacteristicValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'a> {
        let value = serde_json::Value::deserialize(deserializer)?;
        Ok(CharacteristicValue { value })
    }
}

impl Serialize for CharacteristicValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        self.value.serialize(serializer)
    }
}


// #[derive(Debug, Default, Serialize)]
// pub struct NameCharacteristic(Characteristic<String>);
#[derive(Debug, Default, Serialize)]
pub struct IotCharacteristic(pub Characteristic<CharacteristicValue>);

pub type NewCharacteristicFunc = fn(u64, u64) -> IotCharacteristic;


impl IotCharacteristic {
    pub fn new_default(hap_type: MappingHapType) -> Option<Box<dyn HapCharacteristic>> {
        // LightSensorService::new();
        LightbulbService::new(0, 0);
        match hap_type {
            MappingHapType::PowerState => {
                Some(Box::new(PowerStateCharacteristic::new(0, 0)))
            }
            MappingHapType::Brightness => {
                Some(Box::new(BrightnessCharacteristic::new(0, 0)))
            }
            MappingHapType::ColorTemperature => {
                Some(Box::new(ColorTemperatureCharacteristic::new(0, 0)))
            }
            MappingHapType::Hue => {
                Some(Box::new(HueCharacteristic::new(0, 0)))
            }
            MappingHapType::Saturation => {
                Some(Box::new(SaturationCharacteristic::new(0, 0)))
            }
            MappingHapType::CurrentTemperature => {
                Some(Box::new(CurrentTemperatureCharacteristic::new(0, 0)))
            }
            MappingHapType::ContactSensor => {
                // ContactSensorService::new(0, 0);
                Some(Box::new(ContactSensorStateCharacteristic::new(0, 0)))
            }
            MappingHapType::Outlet => {
                Some(Box::new(OutletInUseCharacteristic::new(0, 0)))
            }
            MappingHapType::TemperatureDisplayUnits => {
                Some(Box::new(TemperatureDisplayUnitsCharacteristic::new(0, 0)))
            }
            MappingHapType::TargetHeaterCoolerState=>{
                Some(Box::new(TargetHeaterCoolerStateCharacteristic::new(0, 0)))
            }
            MappingHapType::CurrentHeaterCoolerState=>{
                Some(Box::new(CurrentHeaterCoolerStateCharacteristic::new(0, 0)))
            }
            MappingHapType::TargetTemperature=>{
                Some(Box::new(TargetTemperatureCharacteristic::new(0, 0)))
            }
            MappingHapType::TargetHeatingCoolingState=>{
                Some(Box::new(TargetHeatingCoolingStateCharacteristic::new(0, 0)))
            }
            MappingHapType::CurrentHeatingCoolingState=>{
                Some(Box::new(CurrentHeatingCoolingStateCharacteristic::new(0, 0)))
            }
            MappingHapType::CoolingThresholdTemperature=>{
                Some(Box::new(CoolingThresholdTemperatureCharacteristic::new(0, 0)))
            }
            MappingHapType::HeatingThresholdTemperature=>{
                Some(Box::new(HeatingThresholdTemperatureCharacteristic::new(0, 0)))
            }
            MappingHapType::RotationSpeed=>{
                Some(Box::new(RotationSpeedCharacteristic::new(0, 0)))
            }
            MappingHapType::SwingMode=>{
                Some(Box::new(SwingModeCharacteristic::new(0, 0)))
            }
            MappingHapType::LockPhysicalControls=>{
                Some(Box::new(LockPhysicalControlsCharacteristic::new(0, 0)))
            }
            MappingHapType::TargetFanState=>{
                Some(Box::new(TargetFanStateCharacteristic::new(0, 0)))
            }
            MappingHapType::CurrentFanState=>{
                Some(Box::new(CurrentFanStateCharacteristic::new(0, 0)))
            }
            MappingHapType::Active=>{
                Some(Box::new(ActiveCharacteristic::new(0, 0)))
            }
            _ => None
        }
    }

    pub fn new(id: u64, accessory_id: u64, hap_type: HapType) -> Self {
        Self(Characteristic::<CharacteristicValue> {
            id,
            accessory_id,
            hap_type,
            format: Format::String,
            perms: vec![
                Perm::PairedRead,
            ],
            // max_len: Some(64),
            ..Default::default()
        })
    }
}

#[async_trait]
impl HapCharacteristic for IotCharacteristic {
    fn get_id(&self) -> u64 { HapCharacteristic::get_id(&self.0) }

    fn set_id(&mut self, id: u64) { HapCharacteristic::set_id(&mut self.0, id) }

    fn get_type(&self) -> HapType { HapCharacteristic::get_type(&self.0) }

    fn set_type(&mut self, hap_type: HapType) { HapCharacteristic::set_type(&mut self.0, hap_type) }

    fn get_format(&self) -> Format { HapCharacteristic::get_format(&self.0) }

    fn set_format(&mut self, format: Format) { HapCharacteristic::set_format(&mut self.0, format) }

    fn get_perms(&self) -> Vec<Perm> { HapCharacteristic::get_perms(&self.0) }

    fn set_perms(&mut self, perms: Vec<Perm>) { HapCharacteristic::set_perms(&mut self.0, perms) }

    fn get_description(&self) -> Option<String> { HapCharacteristic::get_description(&self.0) }

    fn set_description(&mut self, description: Option<String>) {
        HapCharacteristic::set_description(&mut self.0, description)
    }

    fn get_event_notifications(&self) -> Option<bool> { HapCharacteristic::get_event_notifications(&self.0) }

    fn set_event_notifications(&mut self, event_notifications: Option<bool>) {
        HapCharacteristic::set_event_notifications(&mut self.0, event_notifications)
    }

    async fn get_value(&mut self) -> hap::Result<serde_json::Value> { HapCharacteristic::get_value(&mut self.0).await }

    async fn set_value(&mut self, value: serde_json::Value) -> hap::Result<()> {
        HapCharacteristic::set_value(&mut self.0, value).await
    }

    fn get_unit(&self) -> Option<Unit> { HapCharacteristic::get_unit(&self.0) }

    fn set_unit(&mut self, unit: Option<Unit>) { HapCharacteristic::set_unit(&mut self.0, unit) }

    fn get_max_value(&self) -> Option<serde_json::Value> { HapCharacteristic::get_max_value(&self.0).map(|v| json!(v)) }

    fn set_max_value(&mut self, max_value: Option<serde_json::Value>) -> hap::Result<()> {
        HapCharacteristic::set_max_value(&mut self.0, max_value)
    }

    fn get_min_value(&self) -> Option<serde_json::Value> { HapCharacteristic::get_min_value(&self.0).map(|v| json!(v)) }

    fn set_min_value(&mut self, min_value: Option<serde_json::Value>) -> hap::Result<()> {
        HapCharacteristic::set_min_value(&mut self.0, min_value)
    }

    fn get_step_value(&self) -> Option<serde_json::Value> {
        HapCharacteristic::get_step_value(&self.0).map(|v| json!(v))
    }

    fn set_step_value(&mut self, step_value: Option<serde_json::Value>) -> hap::Result<()> {
        HapCharacteristic::set_step_value(&mut self.0, step_value)
    }

    fn get_max_len(&self) -> Option<u16> { HapCharacteristic::get_max_len(&self.0) }

    fn set_max_len(&mut self, max_len: Option<u16>) { HapCharacteristic::set_max_len(&mut self.0, max_len) }

    fn get_max_data_len(&self) -> Option<u32> { HapCharacteristic::get_max_data_len(&self.0) }

    fn set_max_data_len(&mut self, max_data_len: Option<u32>) {
        HapCharacteristic::set_max_data_len(&mut self.0, max_data_len)
    }

    fn get_valid_values(&self) -> Option<Vec<serde_json::Value>> { HapCharacteristic::get_valid_values(&self.0) }

    fn set_valid_values(&mut self, valid_values: Option<Vec<serde_json::Value>>) -> hap::Result<()> {
        HapCharacteristic::set_valid_values(&mut self.0, valid_values)
    }

    fn get_valid_values_range(&self) -> Option<[serde_json::Value; 2]> {
        HapCharacteristic::get_valid_values_range(&self.0)
    }

    fn set_valid_values_range(&mut self, valid_values_range: Option<[serde_json::Value; 2]>) -> hap::Result<()> {
        HapCharacteristic::set_valid_values_range(&mut self.0, valid_values_range)
    }

    fn get_ttl(&self) -> Option<u64> { HapCharacteristic::get_ttl(&self.0) }

    fn set_ttl(&mut self, ttl: Option<u64>) { HapCharacteristic::set_ttl(&mut self.0, ttl) }

    fn get_pid(&self) -> Option<u64> { HapCharacteristic::get_pid(&self.0) }

    fn set_pid(&mut self, pid: Option<u64>) { HapCharacteristic::set_pid(&mut self.0, pid) }
}


impl HapCharacteristicSetup for IotCharacteristic {
    fn set_event_emitter(&mut self, event_emitter: Option<pointer::EventEmitter>) {
        HapCharacteristicSetup::set_event_emitter(&mut self.0, event_emitter)
    }
}

impl CharacteristicCallbacks<CharacteristicValue> for IotCharacteristic {
    fn on_read(&mut self, f: Option<impl OnReadFn<CharacteristicValue>>) { CharacteristicCallbacks::on_read(&mut self.0, f) }

    fn on_update(&mut self, f: Option<impl OnUpdateFn<CharacteristicValue>>) { CharacteristicCallbacks::on_update(&mut self.0, f) }
}

impl AsyncCharacteristicCallbacks<CharacteristicValue> for IotCharacteristic {
    fn on_read_async(&mut self, f: Option<impl OnReadFuture<CharacteristicValue>>) {
        AsyncCharacteristicCallbacks::on_read_async(&mut self.0, f)
    }

    fn on_update_async(&mut self, f: Option<impl OnUpdateFuture<CharacteristicValue>>) {
        AsyncCharacteristicCallbacks::on_update_async(&mut self.0, f)
    }
}
