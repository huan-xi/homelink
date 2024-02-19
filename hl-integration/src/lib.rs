use std::any::{Any, TypeId};
use std::sync::Arc;
use crate::platform::hap::hap_device::AsHapDevice;

pub mod error;
pub mod event;
pub mod hl_device;
pub mod platform;
pub mod integration;

pub type HlDeviceResult<T> = Result<T, error::HlDeviceError>;


pub trait HlSourceDevice: Any + hl_device::HlDevice + AsHapDevice {}

impl dyn HlSourceDevice {
    pub fn downcast_ref<T: HlSourceDevice>(&self) -> Option<&T> {
        if self.type_id() == TypeId::of::<T>() {
            unsafe { Some(&*(self as *const dyn HlSourceDevice as *const T)) }
        } else {
            None
        }
    }
}

pub type SourceDevicePointer = Arc<dyn HlSourceDevice>;

pub type JsonValue = serde_json::Value;


