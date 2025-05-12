use std::io;

use thiserror::Error;

use crate::{convert::error::DecompressError, error::HttpReadError};

#[derive(Debug, Error)]
pub enum OneOneRWError {
    #[error("read| {0}")]
    Read(io::Error),
    #[error("write| {0}")]
    Write(io::Error),
    #[error("parse| {0}")]
    HttpError(#[from] HttpReadError),
    #[error("decompress| {0}")]
    Decompress(#[from] DecompressError),
}
