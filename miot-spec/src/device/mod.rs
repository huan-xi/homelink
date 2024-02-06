use std::sync::Arc;
use crate::device::miot_spec_device::MiotSpecDevice;

pub mod miot_spec_device;
pub mod wifi_device;
pub mod mesh_device;
pub mod gateway;
pub mod gw_zigbee_device;
pub mod ble;
pub mod common;
pub mod cloud_device;


pub type MiotDevicePointer = Arc<dyn MiotSpecDevice  + 'static>;