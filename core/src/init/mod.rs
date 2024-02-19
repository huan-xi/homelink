use std::sync::Arc;
use async_trait::async_trait;
use dashmap::DashMap;
use serde_json::Value;
use tokio::sync::RwLock;
use hap::accessory::HapAccessory;
use hap_metadata::metadata::HapMetadata;
use hl_integration::HlSourceDevice;
use miot_proto::device::miot_spec_device::{ MiotSpecDevice};
use miot_proto::device::MiotDevicePointer;
use miot_proto::proto::miio_proto::MiotSpecId;
use target_hap::hap_manager::HapManage;
// use crate::device::platform::PlatformDevice;


pub mod hap_init;
mod characteristic_init;
pub mod manager;
mod accessory_init;
pub(crate) mod helper;
pub mod logger_init;

pub type FuturesMutex<T> = futures_util::lock::Mutex<T>;
pub type TokioMutex<T> = tokio::sync::Mutex<T>;
pub type HapAccessoryPointer = Arc<RwLock<Box<dyn HapAccessory>>>;
pub type AFuturesMutex<T> = Arc<futures_util::lock::Mutex<T>>;



pub type DevicePointer = Arc<dyn HlSourceDevice + Send + Sync + 'static>;
// pub type DevicePointer = MiotDevicePointer;
pub type DeviceMap = DashMap<i64, DevicePointer>;


pub struct Managers {
    pub hap_metadata: Arc<HapMetadata>,
    pub hap_manager: HapManage,
    pub device_manager: manager::device_manager::IotDeviceManager,
    pub mi_account_manager: manager::mi_account_manager::MiAccountManager,
    pub template_manager: manager::template_manager::TemplateManager,
    pub ble_manager: manager::ble_manager::BleManager,
}