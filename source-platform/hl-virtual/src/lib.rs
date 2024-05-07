use hl_integration::error::DeviceExitError;
use hl_integration::event::{EventListener, HlDeviceListenable};
use hl_integration::hl_device::{HlDevice, RetryInfo};
use hl_integration::HlSourceDevice;
use hl_integration::platform::hap::hap_device::{AsHapDevice, DeviceInfo, HapDevice};

pub struct VirtualDevice {
    retry_info: RetryInfo,
}
impl VirtualDevice {
    pub fn new() -> Self {
        Self {
            retry_info: RetryInfo::default(),
        }
    }
}

impl HlSourceDevice for VirtualDevice {}

#[async_trait::async_trait]
impl HlDeviceListenable for VirtualDevice {
    async fn add_listener(&self, listener: EventListener) -> i64 {
        0
    }

    fn remove_listener(&self, id: i64) -> i64 {
        0
    }
}

#[async_trait::async_trait]
impl HlDevice for VirtualDevice {
    fn dev_id(&self) -> String {
        "virtual_device".to_string()
    }

    fn device_type(&self) -> &str {
        "virtual_device"
    }

    async fn run(&self) -> Result<(), Box<dyn DeviceExitError>> {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(10)).await;
        }
    }

    async fn enabled(&self) -> bool {
        true
    }

    fn retry_info(&self) -> &RetryInfo {
        &self.retry_info
    }
}


impl HapDevice for VirtualDevice {
    fn get_hap_info(&self) -> DeviceInfo {
        DeviceInfo {
            manufacturer: "zs".to_string(),
            model: "zs".to_string(),
            serial_number: "zs".to_string(),
            software_revision: None,
            firmware_revision: None,
        }
    }
}