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
    pub encoding_index: usize,
    pub extra_body: Option<BytesMut>,
    pub error: DecompressError,
}

impl DEStruct {
    pub fn is_unknown_encoding(&self) -> bool {
        matches!(self.error, DecompressError::Unknown(_))
    }

    pub fn new(
        encoding_index: usize,
        body: BytesMut,
        extra_body: Option<BytesMut>,
        error: DecompressError,
    ) -> Self {
        DEStruct {
            encoding_index,
            body,
            extra_body,
            error,
        }
    }
}

/*
impl From<(usize, BytesMut, Option<BytesMut>, DecompressError)> for DEStruct {
    fn from(
        (encoding_index, body, extra_body, error): (
            usize,
            BytesMut,
            Option<BytesMut>,
            DecompressError,
        ),
    ) -> Self {
        DEStruct {
            encoding_index,
            body,
            extra_body,
            error,
        }
    }
}
*/
