use std::sync::Arc;
use dashmap::DashMap;
use hap::accessory::HapAccessory;
use miot_spec::device::miot_spec_device::MiotSpecDevice;
use crate::init::manager::device_manager::DeviceWithJsEngine;

pub mod hap_init;
pub mod device_init;
mod mapping_characteristic;
/// 米家转换模板
pub mod template;
pub mod manager;


pub type FuturesMutex<T> = futures_util::lock::Mutex<T>;
pub type HapAccessoryPointer = Arc<FuturesMutex<Box<dyn HapAccessory>>>;
pub type AFuturesMutex<T> = Arc<futures_util::lock::Mutex<T>>;
pub type DevicePointer = DeviceWithJsEngine;
// pub type DevicePointer = Arc<RwLock<IotDeviceAccessory>>;
pub type DeviceMap = DashMap<i64, DevicePointer>;
