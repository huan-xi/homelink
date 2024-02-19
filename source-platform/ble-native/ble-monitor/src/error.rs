use std::error::Error;
use thiserror::Error;
use xiaomi_ble_packet::error::MiPacketError;

#[derive(Debug, Error)]
pub enum BleError {
    #[error("Failed to unpack advertisement data {0}")]
    PacketUnpackError(#[from] Box<dyn Error>),
    #[error("BleValueTypeError {0}")]
    BleValueTypeError(u16),
    #[error("UnpackDataError {0}")]
    UnpackDataError(&'static str),
    #[error("PackingError {0}")]
    PackingError(#[from] packed_struct::PackingError),
    #[error("NotSupported {0}")]
    NotSupported(&'static str),
    #[error("NotSupportedPlatform 0x{0:x}")]
    NotSupportedPlatform(u16),
    #[error("NotSupportedPlatform 0x{0:x}")]
    NotSupportedDeviceType(u16),
    #[error("MiPacketError {0}")]
    MiPacketError(#[from] MiPacketError),
}

// `From<MiPacketError>`