use std::fmt::Debug;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum HlDeviceError {
    /// 事件执行错误
    #[error("EventError")]
    EventError,
}

pub trait DeviceExitError: Debug + Send + Sync {
    /// 是否可重试
    fn retryable(&self) -> bool;
}
