use dashmap::DashMap;
use log::info;
use serde_json::json;
use hl_integration::error::DeviceExitError;
use hl_integration::event::{EventListener, HlDeviceListenable};
use hl_integration::event::emitter::EventEmitter;
use hl_integration::event::events::DeviceEvent;
use hl_integration::hl_device::RetryInfo;
use hl_integration::hl_device::HlDevice;
use miot_proto::device::miot_spec_device::{AsMiotDevice,  MiotSpecDevice};
use crate::device::platform::{NotSupportNativeBleDeviceError, PlatformDevice};
use crate::init::manager::ble_manager::BleManager;

/// 特征映射map
pub struct NativeBleDevice {
    // fields omitted
    mac: [u8; 6],
    dev_id: i64,
    ble_manager: BleManager,
    // 存储最近一次的值
    pub value_register: DashMap<u16, Vec<u8>>,
    retry_info: RetryInfo,
    event_emitter: EventEmitter,
}

impl NativeBleDevice {
    pub fn new(mac: [u8; 6], dev_id: i64, ble_manager: BleManager) -> Self {
        NativeBleDevice {
            mac,
            dev_id,
            value_register: Default::default(),
            ble_manager,
            retry_info: Default::default(),
            event_emitter: Default::default(),
        }
    }
}

/// 属性数据
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct PropData {
    pub etype: u16,
    pub edata: Vec<u8>,
}


#[async_trait::async_trait]
impl HlDevice for NativeBleDevice {
    fn dev_id(&self) -> String {
        self.dev_id.to_string()
    }

    async fn run(&self) -> Result<(), Box<dyn DeviceExitError>> {
        let mut recv = self.ble_manager.recv();
        loop {
            let value = recv.recv().await.unwrap();
            //发布事件
            info!("recv value: {:?}", value);
            if value.mac == self.mac {
                if !self.value_register
                    .get(&value.etype)
                    .map(|v| v.value() == &value.edata)
                    .unwrap_or(false) {
                    self.value_register.insert(value.etype, value.edata.clone());
                    info!("emit 本地蓝牙 event: {:?}", value);
                    self.event_emitter.emit(DeviceEvent::PropertyChanged(json!(PropData {
                    edata: value.edata,
                    etype: value.etype,
                }))).await;
                };
            }
        }
    }

    fn retry_info(&self) -> &RetryInfo {
        &self.retry_info
    }
}

#[async_trait::async_trait]
impl HapDeviceExt for NativeBleDevice {
    fn get_hap_info(&self) -> hap_device_ext::DeviceInfo {
        hap_device_ext::DeviceInfo {
            manufacturer: "未知".to_string(),
            model: "".to_string(),
            serial_number: "xxx".to_string(),
            software_revision: None,
            firmware_revision: None,
        }
    }
}

impl AsMiotDevice for NativeBleDevice {}

impl PlatformDevice for NativeBleDevice {
    fn as_native_ble(&self) -> Result<&NativeBleDevice, NotSupportNativeBleDeviceError> {
        Ok(self)
    }
}


#[async_trait::async_trait]
impl HlDeviceListenable for NativeBleDevice {
    async fn add_listener(&self, listener: EventListener) {
        self.event_emitter.add_listener(listener).await;
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;
    use std::time::Duration;
    use hl_integration::HlDevice;
    use miot_proto::device::miot_spec_device::MiotSpecDevice;
    use crate::init::logger_init::init_logger;

    #[tokio::test]
    pub async fn test() {
        init_logger();
        let ble_manager = super::BleManager::new();
        ble_manager.init().await;

        // super::NativeBleDevice {
        //     mac: [23, 117, 189, 97, 185, 34],
        //     dev_id: 1,
        //     value_register: Default::default(),
        //     ble_manager,
        //     retry_info: Default::default(),
        //     event_emitter: Default::default(),
        // }

        let dev = Arc::new(super::NativeBleDevice::new([23, 117, 189, 97, 185, 34], 1, ble_manager));
        let dev_c = dev.clone();
        tokio::spawn(async move {
            dev_c.run().await.unwrap();
        });

        tokio::time::sleep(Duration::from_secs(10)).await;
    }
}