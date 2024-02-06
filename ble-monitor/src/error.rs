use std::error::Error;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BleError {
    #[error("Failed to unpack advertisement data {0}")]
    UnpackError(#[from] Box<dyn Error>),
    #[error("BleValueTypeError {0}")]
    BleValueTypeError(u16),
    #[error("UnpackDataError {0}")]
    UnpackDataError(&'static str),
    #[error("PackingError {0}")]
    PackingError(#[from] packed_struct::PackingError),
    #[error("NotSupported {0}")]
    NotSupported(&'static str),
    #[error("NotSupportedPlatform {0}")]
    NotSupportedPlatform(u16),
}