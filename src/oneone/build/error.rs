use header_plz::error::HeaderReadError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BuildMessageError {
    #[error("Failed to FindCRLF")]
    UnableToFindCRLF,
    #[error("Failed to DecodeHTTP| {0}")]
    HttpDecodeError(#[from] HeaderReadError),
}
