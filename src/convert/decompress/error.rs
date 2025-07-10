use std::io::Error;

use bytes::BytesMut;
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

#[derive(Debug)]
pub struct DecompressErrorStruct {
    pub body: BytesMut,
    pub extra_body: Option<BytesMut>,
    pub error: DecompressError,
}

impl DecompressErrorStruct {
    pub fn is_unknown_encoding(&self) -> bool {
        matches!(self.error, DecompressError::Unknown(_))
    }

    pub fn new(body: BytesMut, extra_body: Option<BytesMut>, error: DecompressError) -> Self {
        DecompressErrorStruct {
            body,
            extra_body,
            error,
        }
    }

    pub fn into_body_and_error(mut self) -> (BytesMut, DecompressError) {
        if let Some(ebody) = self.extra_body {
            self.body.unsplit(ebody);
        }
        (self.body, self.error)
    }
}
