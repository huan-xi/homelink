use std::sync::Arc;
use hl_virtual::VirtualDevice;
use crate::db::entity::prelude::IotDeviceModel;
use crate::init::DevicePointer;
use crate::init::manager::device_manager::IotDeviceManagerInner;


impl IotDeviceManagerInner {
    ///本地蓝牙设备
    pub async fn init_hl_virtual(&self, dev: IotDeviceModel) -> anyhow::Result<DevicePointer> {
        let dev = VirtualDevice::new();
        // let dev = NativeBleDevice::new([23, 117, 189, 97, 185, 34], dev.device_id , self.ble_manager.clone());
        Ok(Arc::new(dev))
    }
}