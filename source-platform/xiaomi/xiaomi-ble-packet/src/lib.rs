pub mod ble_value_type;
pub mod error;

pub type MiPacketResult<T> = Result<T, error::MiPacketError>;