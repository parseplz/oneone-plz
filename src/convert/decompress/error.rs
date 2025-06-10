use std::io::Error;

use bytes::BytesMut;
use header_plz::body_headers::content_encoding::ContentEncoding;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DecompressError {
    #[error("brotli| {0}")]
    Brotli(Error),
    #[error("deflate| {0}")]
    Deflate(Error),
    #[error("gzip| {0}")]
    Gzip(Error),
    #[error("zstd| {0}")]
    Zstd(Error),
    #[error("unknown| {0}")]
    Unknown(String),
}

impl DecompressError {
    pub fn is_unknown_encoding(&self) -> bool {
        matches!(self, DecompressError::Unknown(_))
    }
}

#[derive(Debug)]
pub struct DEStruct {
    pub body: BytesMut,
    pub error: DecompressError,
}

impl DEStruct {
    pub fn is_unknown_encoding(&self) -> bool {
        matches!(self.error, DecompressError::Unknown(_))
    }
}

impl From<(BytesMut, DecompressError)> for DEStruct {
    fn from((body, error): (BytesMut, DecompressError)) -> Self {
        DEStruct { body, error }
    }
}

impl From<&DEStruct> for ContentEncoding {
    fn from(value: &DEStruct) -> Self {
        match &value.error {
            DecompressError::Brotli(_) => ContentEncoding::Brotli,
            DecompressError::Deflate(_) => ContentEncoding::Deflate,
            DecompressError::Gzip(_) => ContentEncoding::Gzip,
            DecompressError::Zstd(_) => ContentEncoding::Zstd,
            DecompressError::Unknown(e) => ContentEncoding::Unknown(e.to_string()),
        }
    }
}
