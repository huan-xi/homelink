use dashmap::DashMap;
use log::info;
use ble_monitor::BleValue;
use hl_device::{CharReadParam, CharUpdateParam, ReadCharResults, UpdateCharResults};
use hl_device::error::DeviceExitError;
use hl_device::event::{EventListener, HlDeviceListenable};
use hl_device::hl_device::RetryInfo;
use hl_device::HlDevice;
use hl_device::platform::hap::hap_device_ext;
use hl_device::platform::hap::hap_device_ext::HapDeviceExt;
use miot_spec::device::miot_spec_device::{AsMiotGatewayDevice, MiotSpecDevice};
use crate::init::manager::ble_manager::BleManager;

/// 特征映射map
pub struct NativeBleDevice {
    // fields omitted
    mac: [u8; 6],
    dev_id: String,
    // 存储最近一次的值
    value_register: DashMap<u16, BleValue>,
    ble_manager: BleManager,
}

#[async_trait::async_trait]
impl HlDeviceListenable for NativeBleDevice {
    async fn add_listener(&self, listener: EventListener) {
        todo!()
    }
}

#[async_trait::async_trait]
impl HlDevice for NativeBleDevice {
    fn dev_id(&self) -> &str {
        self.dev_id.as_str()
    }

    async fn run(&self) -> Result<(), Box<dyn DeviceExitError>> {
        let mut recv = self.ble_manager.recv();
        loop {
            let value = recv.recv().await.unwrap();
            info!("recv value: {:?}", value);
            // self.value_register.insert(value.handle, value);
        }
    }

    fn retry_info(&self) -> &RetryInfo {
        todo!()
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

    async fn read_chars_value(&self, params: Vec<CharReadParam>) -> ReadCharResults {
        /// 读取特征值,1->value_type ->convert to hap value


        todo!()
    }

    async fn update_chars_value(&self, params: Vec<CharUpdateParam>) -> UpdateCharResults {
        todo!()
    }
}


#[cfg(test)]
mod test {
    use std::sync::Arc;
    use std::time::Duration;
    use hl_device::HlDevice;
    use miot_spec::device::miot_spec_device::MiotSpecDevice;
    use crate::init::logger_init::init_logger;

    #[tokio::test]
    pub async fn test() {
        init_logger();
        let ble_manager = super::BleManager::new();
        ble_manager.init().await.unwrap();
        let dev = Arc::new(super::NativeBleDevice {
            mac: [23, 117, 189, 97, 185, 34],
            dev_id: "".to_string(),
            value_register: Default::default(),
            ble_manager,
        });
        let dev_c = dev.clone();
        tokio::spawn(async move {
            dev_c.run().await.unwrap();
        });

        tokio::time::sleep(Duration::from_secs(10)).await;
    }
}