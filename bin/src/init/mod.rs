use std::sync::Arc;
use dashmap::DashMap;
use tokio::sync::RwLock;
use hap::accessory::HapAccessory;
use miot_spec::device::miot_spec_device::MiotSpecDevice;
use crate::init::manager::device_manager::DeviceWithJsEngine;

pub mod hap_init;
pub mod device_init;
mod characteristic_init;
pub mod manager;
mod accessory_init;
mod manager_init;
pub(crate) mod helper;

pub type FuturesMutex<T> = futures_util::lock::Mutex<T>;
pub type TokioMutex<T> = tokio::sync::Mutex<T>;
pub type HapAccessoryPointer = Arc<RwLock<Box<dyn HapAccessory>>>;
pub type AFuturesMutex<T> = Arc<futures_util::lock::Mutex<T>>;
pub type DevicePointer = DeviceWithJsEngine;
// pub type DevicePointer = Arc<RwLock<IotDeviceAccessory>>;
pub type DeviceMap = DashMap<i64, DevicePointer>;


pub struct Managers {
    pub hap_manager: manager::hap_manager::HapManage,
    pub iot_device_manager: manager::device_manager::IotDeviceManager,
    pub mi_account_manager: manager::mi_account_manager::MiAccountManager,
}