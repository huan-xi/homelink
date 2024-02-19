use crate::event::HlDeviceListenable;

#[derive(Debug,Clone)]
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
pub trait HapDevice: HlDeviceListenable {
    /// 获取设备信息
    fn get_hap_info(&self) -> DeviceInfo;
}



impl <T: HapDevice> AsHapDevice for T {
    fn as_hap_device(&self) -> Option<&dyn HapDevice> {
        Some(self)
    }
}

// impl AsHapDevice for dyn HlSourceDevice {
//     fn as_hap_device(&self) -> Option<&dyn HapDevice> {
//         self.downcast_ref::<>()
//     }
// }


pub trait AsHapDevice {
    fn as_hap_device(&self) -> Option<&dyn HapDevice>;
}
