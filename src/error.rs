use body_plz::{reader::chunked_reader::ChunkReaderError, variants::Body};
use bytes::BytesMut;
use decompression_plz::DecompressTrait;
use header_plz::{
    OneHeader, OneInfoLine,
    body_headers::parse::ParseBodyHeaders,
    message_head::{MessageHead, error::MessageHeadError},
};
use http_plz::OneOne;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error<T>
where
    T: OneInfoLine,
{
    // Parsing error
    #[error("header read| {0}")]
    InfoLine(#[from] MessageHeadError),
    #[error("chunkreader| {0}")]
    ChunkState(ChunkReaderError, Box<OneOne<T>>),
    // Partial
    #[error("partial| header")]
    Unparsed(BytesMut),
    #[error("partial| content length")]
    ContentLengthPartial(Box<(OneOne<T>, BytesMut)>),
    #[error("partial| chunked")]
    ChunkReaderPartial(Box<(OneOne<T>, BytesMut)>),
}

impl<T> Error<T>
where
    T: OneInfoLine + std::fmt::Debug,
    MessageHead<T, OneHeader>: ParseBodyHeaders,
{
    pub fn into_bytes(self) -> BytesMut {
        BytesMut::from(self)
    }

    pub fn try_into_msg(self) -> Result<OneOne<T>, Error<T>> {
        OneOne::<T>::try_from(self)
    }

    pub fn is_parse_error(&self) -> bool {
        use Error::*;
        matches!(self, InfoLine(_)) | matches!(self, ChunkState(..))
    }

    pub fn is_partial(&self) -> bool {
        use Error::*;
        matches!(self, ContentLengthPartial(_))
            | matches!(self, ChunkReaderPartial(_))
            | matches!(self, ChunkState(..))
    }

    pub fn is_unparsed(&self) -> bool {
        use Error::*;
        matches!(self, Unparsed(_)) | matches!(self, InfoLine(_))
    }
}

impl<T> From<Error<T>> for BytesMut
where
    T: OneInfoLine + std::fmt::Debug,
    MessageHead<T, OneHeader>: ParseBodyHeaders,
{
    fn from(value: Error<T>) -> Self {
        use Error::*;
        match value {
            InfoLine(err) => err.into_bytes(),
            Unparsed(buf) => buf,
            ContentLengthPartial(boxed) | ChunkReaderPartial(boxed) => {
                let (oneone, buf) = *boxed;
                let mut data = oneone.into_bytes();
                data.unsplit(buf);
                data
            }
            ChunkState(_, boxed) => boxed.into_bytes(),
        }
    }
}

#[derive(Debug, Error, Default)]
#[error("incorrect state| {0}")]
pub struct IncorrectState(pub(crate) String);

impl<T> TryFrom<Error<T>> for OneOne<T>
where
    T: OneInfoLine,
    MessageHead<T, OneHeader>: ParseBodyHeaders,
{
    type Error = Error<T>;

    fn try_from(value: Error<T>) -> Result<Self, Self::Error> {
        match value {
            Error::ContentLengthPartial(boxed)
            | Error::ChunkReaderPartial(boxed) => {
                let (mut oneone, buf) = *boxed;
                oneone.set_body(Body::Raw(buf));
                Ok(oneone)
            }
            Error::ChunkState(_, boxed) => Ok(*boxed),
            _ => Err(value),
        }
    }
}
