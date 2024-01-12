use std::sync::{Arc};
use dashmap::DashMap;
use hap::accessory::HapAccessory;
use tokio::sync::RwLock;
use miot_spec::device::miot_spec_device::MiotSpecDevice;
use crate::hap::iot_hap_accessory::IotDeviceAccessory;

pub mod hap_init;
pub mod device_init;
mod mapping_characteristic;
pub mod device_manage;
pub mod hap_manage;


//
pub type FuturesMutex<T> = futures_util::lock::Mutex<T>;
pub type HapAccessoryPointer = Arc<FuturesMutex<Box<dyn HapAccessory>>>;

pub type AFuturesMutex<T> = Arc<futures_util::lock::Mutex<T>>;
pub type DevicePointer = Arc<dyn MiotSpecDevice + Send + Sync>;

// pub type DevicePointer = Arc<RwLock<IotDeviceAccessory>>;

pub type DeviceMap = DashMap<i64, DevicePointer>;
