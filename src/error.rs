use body_plz::{reader::chunked_reader::ChunkReaderError, variants::Body};
use bytes::BytesMut;
use decompression_plz::{
    DecompressTrait, content_length::update_content_length,
};
use header_plz::{
    OneHeader, OneInfoLine, body_headers::parse::ParseBodyHeaders,
    error::HeaderReadError, message_head::MessageHead,
};
use http_plz::OneOne;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum HttpStateError<T>
where
    T: OneInfoLine,
{
    #[error("header read| {0}")]
    InfoLine(#[from] HeaderReadError),
    #[error("chunkreader| {0}")]
    ChunkState(#[from] ChunkReaderError),
    // Partial
    #[error("partial| header")]
    Unparsed(BytesMut),
    #[error("partial| content length")]
    ContentLengthPartial(Box<(OneOne<T>, BytesMut)>),
    #[error("header not enough data")]
    ChunkReaderPartial(Box<(OneOne<T>, BytesMut)>),
}

impl<T> HttpStateError<T>
where
    T: OneInfoLine + std::fmt::Debug,
    MessageHead<T, OneHeader>: ParseBodyHeaders,
{
    pub fn into_bytes(self) -> BytesMut {
        BytesMut::from(self)
    }

    pub fn try_into_one(self) -> Result<OneOne<T>, HttpStateError<T>> {
        OneOne::<T>::try_from(self)
    }

    pub fn is_partial(&self) -> bool {
        matches!(self, Self::ContentLengthPartial(_))
    }

    pub fn is_unparsed(&self) -> bool {
        matches!(self, Self::Unparsed(_))
    }
}

impl<T> From<HttpStateError<T>> for BytesMut
where
    T: OneInfoLine + std::fmt::Debug,
    MessageHead<T, OneHeader>: ParseBodyHeaders,
{
    fn from(value: HttpStateError<T>) -> Self {
        match value {
            HttpStateError::Unparsed(buf) => buf,
            HttpStateError::ContentLengthPartial(boxed)
            | HttpStateError::ChunkReaderPartial(boxed) => {
                let (oneone, buf) = *boxed;
                let mut data = oneone.into_bytes();
                data.unsplit(buf);
                data
            }
            _ => BytesMut::new(),
        }
    }
}

#[derive(Debug, Error, Default)]
#[error("incorrect state| {0}")]
pub struct IncorrectState(pub(crate) String);

impl<T> TryFrom<HttpStateError<T>> for OneOne<T>
where
    T: OneInfoLine,
    MessageHead<T, OneHeader>: ParseBodyHeaders,
{
    type Error = HttpStateError<T>;

    fn try_from(value: HttpStateError<T>) -> Result<Self, Self::Error> {
        match value {
            HttpStateError::ContentLengthPartial(boxed) => {
                let (mut oneone, buf) = *boxed;
                let len = buf.len();
                oneone.set_body(Body::Raw(buf));
                update_content_length(&mut oneone, len);
                Ok(oneone)
            }
            _ => Err(value),
        }
    }
}
