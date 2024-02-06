use std::sync::Arc;
pub mod error;
pub mod event;
pub mod hl_device;
pub mod platform;

pub type HlDeviceResult<T> = Result<T, error::HlDeviceError>;

pub use hap::characteristic::CharReadParam;
pub use hap::characteristic::CharReadResult;
pub use hap::characteristic::CharUpdateParam;
pub use hap::characteristic::CharUpdateResult;
pub use hap::characteristic::ReadCharResults;
pub use hap::characteristic::UpdateCharResults;
pub use hl_device::HlDevice;



