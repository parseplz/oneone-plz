use bytes::BytesMut;
use header_plz::const_headers::CONTENT_LENGTH;

use super::*;
pub mod error;
use error::*;
mod request;
mod response;

pub trait BuildFrame {
    fn build(buf: BytesMut) -> Result<Self, UpdateFrameError>
    where
        Self: Sized;
}
