use std::any::{Any, TypeId};
use std::sync::Arc;
use crate::HlSourceDevice;

pub type DeviceEventPointer = Arc<dyn DeviceEvent + Send + Sync + 'static>;

pub trait DeviceEvent: Any {
}

impl dyn DeviceEvent {
    pub fn downcast_ref<T: DeviceEvent>(&self) -> Option<&T> {
        if self.type_id() == TypeId::of::<T>() {
            unsafe { Some(&*(self as *const dyn DeviceEvent as *const T)) }
        } else {
            None
        }
    }
}



//
// /// 设备产生的事件
// #[derive(Debug, Clone)]
// pub struct DeviceEvent {
//     /// 事件名称
//     pub name: String,
//     /// 事件值
//     pub value: serde_json::Value,
// }