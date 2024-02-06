use crate::hap_platform::hap_device_ext::HapDeviceExt;

pub mod hap_platform;
pub mod error;
pub mod event;

pub type HlDeviceResult<T> = Result<T, error::HlDeviceError>;

pub use hap::characteristic::CharReadParam;
pub use hap::characteristic::CharReadResult;
pub use hap::characteristic::CharUpdateParam;
pub use hap::characteristic::CharUpdateResult;
pub use hap::characteristic::ReadCharResults;
pub use hap::characteristic::UpdateCharResults;



/// 平台的设备抽象
/// 属性(特征),服务,事件
/// 读写特征值,调用服务,订阅事件
pub trait HlDevice {
    /// 转成hap 设备扩展
    fn as_hap_device_ext(&self) -> Option<&dyn HapDeviceExt> {
        None
    }
}
