use hap::characteristic::{CharReadParam, CharUpdateParam, ReadCharResults, UpdateCharResults};
use crate::error::DeviceExitError;
use crate::event::HlDeviceListenable;
use crate::hl_device::RetryInfo;
use crate::HlDevice;

#[derive(Debug)]
pub struct DeviceInfo {
    pub manufacturer: String,
    /// Contains the manufacturer-specific model of the accessory, e.g. "A1234".
    pub model: String,
    /// Contains the manufacturer-specific serial number of the accessory, e.g. "1A2B3C4D5E6F".
    /// The length must be greater than 1.
    pub serial_number: String,
    pub software_revision: Option<String>,
    pub firmware_revision: Option<String>,
}


/// 实现该trait可以转成homekit 配件
/// 特征值读写,提交事件 能力
/// 初始化绑定,
#[async_trait::async_trait]
pub trait HapDeviceExt: HlDeviceListenable {
    /// 获取设备信息
    fn get_hap_info(&self) -> DeviceInfo;

    /// hap设备特征值批量写入
    /// 配件读写值
    async fn read_chars_value(&self, params: Vec<CharReadParam>) -> ReadCharResults;
    async fn update_chars_value(&self, params: Vec<CharUpdateParam>) -> UpdateCharResults;
}




impl <T:HapDeviceExt> AsHapDeviceExt for T {
    fn as_hap_device_ext(&self) -> Option<&dyn HapDeviceExt> {
        Some(self)
    }
}

pub trait AsHapDeviceExt {
    fn as_hap_device_ext(&self) -> Option<&dyn HapDeviceExt>;
}
