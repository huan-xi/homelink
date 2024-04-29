use thiserror::Error;

#[derive(Debug, Error)]
pub enum MiPacketError {
    #[error("PackingError {0}")]
    PackingError(#[from] packed_struct::PackingError),
    #[error("UnpackDataError {0}")]
    UnpackDataError(String),

}