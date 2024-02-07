use std::sync::Arc;
pub mod error;
pub mod event;
pub mod hl_device;
pub mod platform;

pub type HlDeviceResult<T> = Result<T, error::HlDeviceError>;

pub use hap::characteristic::delegate::CharReadParam;
pub use hap::characteristic::delegate::CharReadResult;
pub use hap::characteristic::delegate::CharUpdateParam;
pub use hap::characteristic::delegate::CharUpdateResult;
pub use hap::characteristic::delegate::ReadCharResults;
pub use hap::characteristic::delegate::UpdateCharResults;
pub use hl_device::HlDevice;



