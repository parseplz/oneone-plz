use body_plz::{body_struct::Body, reader::chunked_reader::ChunkReaderError};
use bytes::BytesMut;
use header_plz::{
    body_headers::parse::ParseBodyHeaders, error::HeaderReadError, info_line::InfoLine,
    message_head::MessageHead,
};
use protocol_traits_plz::Frame;
use thiserror::Error;

use crate::{convert::update_content_length, oneone::OneOne};

#[derive(Debug, Error)]
pub enum HttpReadError<T>
where
    T: InfoLine,
{
    #[error("header read| {0}")]
    InfoLine(#[from] HeaderReadError),
    #[error("chunkreader| {0}")]
    ChunkReaderFailed(#[from] ChunkReaderError),
    // Partial
    #[error("partial| header")]
    Unparsed(BytesMut),
    #[error("partial| content length")]
    ContentLengthPartial(OneOne<T>, BytesMut),
    //
    #[error("header not enough data")]
    NChunkReaderNotEnoughData(OneOne<T>, BytesMut), // partial body
    //
    #[error("chunk reader not enough data")]
    ChunkReaderNotEnoughData,
}

impl<T> From<HttpReadError<T>> for BytesMut
where
    T: InfoLine,
    MessageHead<T>: ParseBodyHeaders,
{
    fn from(value: HttpReadError<T>) -> Self {
        match value {
            HttpReadError::ContentLengthPartial(mut oneone, buf) => {
                let mut data = oneone.into_data();
                data.unsplit(buf);
                data
            }
            _ => BytesMut::new(),
        }
    }
}

impl<T> TryFrom<HttpReadError<T>> for OneOne<T>
where
    T: InfoLine,
    MessageHead<T>: ParseBodyHeaders,
{
    type Error = Self;

    fn try_from(value: HttpReadError<T>) -> Result<Self, Self::Error> {
        match value {
            HttpReadError::ContentLengthPartial(mut oneone, buf) => {
                let len = buf.len();
                oneone.set_body(Body::Raw(buf));
                update_content_length(&mut oneone, len);
                Ok(oneone)
            }
            _ => todo!(),
        }
    }
}
