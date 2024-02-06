use thiserror::Error;

#[derive(Debug, Error)]
pub enum HlDeviceError {
    /// 事件执行错误
    #[error("EventError")]
    EventError,
}