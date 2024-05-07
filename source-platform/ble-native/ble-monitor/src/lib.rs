use crate::error::BleError;

pub mod parse_advertisement;
pub mod parser;
pub mod error;


pub type BltResult<T> = Result<T, BleError>;