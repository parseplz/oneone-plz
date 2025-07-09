use body_plz::{reader::chunked_reader::ChunkReaderError, variants::Body};
use bytes::BytesMut;
use header_plz::{
    InfoLine, body_headers::parse::ParseBodyHeaders, error::HeaderReadError,
    message_head::MessageHead,
};
use protocol_traits_plz::Frame;
use thiserror::Error;

use crate::{convert::content_length::update_content_length, oneone::OneOne};

#[derive(Debug, Error)]
pub enum HttpStateError<T>
where
    T: InfoLine,
{
    #[error("header read| {0}")]
    InfoLine(#[from] HeaderReadError),
    #[error("chunkreader| {0}")]
    ChunkState(#[from] ChunkReaderError),
    // Partial
    #[error("partial| header")]
    Unparsed(BytesMut),
    #[error("partial| content length")]
    ContentLengthPartial(OneOne<T>, BytesMut),
    #[error("header not enough data")]
    ChunkReaderPartial(OneOne<T>, BytesMut),
}

impl<T> From<HttpStateError<T>> for BytesMut
where
    T: InfoLine,
    MessageHead<T>: ParseBodyHeaders,
{
    fn from(value: HttpStateError<T>) -> Self {
        match value {
            HttpStateError::Unparsed(buf) => buf,
            HttpStateError::ContentLengthPartial(oneone, buf)
            | HttpStateError::ChunkReaderPartial(oneone, buf) => {
                let mut data = oneone.into_bytes();
                data.unsplit(buf);
                data
            }
            _ => BytesMut::new(),
        }
    }
}

impl<T> TryFrom<HttpStateError<T>> for OneOne<T>
where
    T: InfoLine,
    MessageHead<T>: ParseBodyHeaders,
{
    type Error = HttpStateError<T>;

    fn try_from(value: HttpStateError<T>) -> Result<Self, Self::Error> {
        match value {
            HttpStateError::ContentLengthPartial(mut oneone, buf) => {
                let len = buf.len();
                oneone.set_body(Body::Raw(buf));
                update_content_length(&mut oneone, len);
                Ok(oneone)
            }
            _ => Err(value),
        }
    }
}
