use body_plz::variants::{Body, chunked::ChunkType};
use bytes::BytesMut;
use decompression_plz::MultiDecompressErrorReason;
use decompression_plz::decompress;
use header_plz::{
    Header, InfoLine,
    body_headers::{BodyHeader, parse::ParseBodyHeaders},
    const_headers::{
        CLOSE, CONNECTION, KEEP_ALIVE, PROXY_CONNECTION, TRAILER, WS_EXT,
    },
    error::HeaderReadError,
    message_head::MessageHead,
};
use protocol_traits_plz::Frame;
pub mod impl_decompress;
pub mod impl_try_from_bytes;

pub mod build;
mod request;
mod response;

#[cfg_attr(any(test, debug_assertions), derive(Debug, PartialEq, Eq))]
pub struct OneOne<T>
where
    T: InfoLine,
{
    message_head: MessageHead<T>,
    body_headers: Option<BodyHeader>,
    body: Option<Body>,
    extra_body: Option<BytesMut>,
}

impl<T> OneOne<T>
where
    T: InfoLine,
    MessageHead<T>: ParseBodyHeaders,
{
    pub fn new(
        message_head: MessageHead<T>,
        body_headers: Option<BodyHeader>,
    ) -> Self {
        OneOne {
            message_head,
            body_headers,
            body: None,
            extra_body: None,
        }
    }

    // parse from message_head
    pub fn try_from_message_head_buf(
        buf: BytesMut,
    ) -> Result<Self, HeaderReadError> {
        let message_head = MessageHead::<T>::try_from(buf)?;
        let body_headers = message_head.parse_body_headers();
        Ok(OneOne::<T>::new(message_head, body_headers))
    }

    // Header Related methods
    pub fn message_head(&self) -> &MessageHead<T> {
        &self.message_head
    }

    pub fn has_header_key(&self, key: &str) -> Option<usize> {
        self.message_head
            .header_map()
            .header_key_position(key)
    }

    pub fn add_header(&mut self, key: &str, value: &str) {
        let header: Header = (key, value).into();
        self.message_head
            .header_map_as_mut()
            .add_header(header);
    }

    pub fn append_headers(&mut self, mut headers: Vec<Header>) {
        self.message_head
            .header_map_as_mut()
            .headers_as_mut()
            .append(&mut headers);
    }

    pub fn update_header_value_on_position(
        &mut self,
        pos: usize,
        value: &str,
    ) {
        self.message_head
            .header_map_as_mut()
            .update_header_value_on_position(pos, value);
    }

    pub fn update_header_value_on_key(
        &mut self,
        key: &str,
        value: &str,
    ) -> bool {
        self.message_head
            .header_map_as_mut()
            .update_header_value_on_key(key, value)
    }

    pub fn remove_header_on_position(&mut self, pos: usize) {
        self.message_head
            .header_map_as_mut()
            .remove_header_on_position(pos);
    }

    pub fn remove_header_on_key(&mut self, key: &str) -> bool {
        self.message_head
            .header_map_as_mut()
            .remove_header_on_key(key)
    }

    pub fn truncate_header_value_on_position<E>(
        &mut self,
        pos: usize,
        truncate_at: E,
    ) where
        E: AsRef<str>,
    {
        self.message_head
            .header_map_as_mut()
            .truncate_header_value_on_position(pos, truncate_at);
    }

    pub fn has_trailers(&self) -> bool {
        self.message_head
            .header_map()
            .header_key_position(TRAILER)
            .is_some()
    }

    // Body Headers Related
    pub fn body(&self) -> &Option<Body> {
        &self.body
    }

    pub fn body_headers(&self) -> &Option<BodyHeader> {
        &self.body_headers
    }

    pub fn body_as_mut(&mut self) -> Option<&mut Body> {
        self.body.as_mut()
    }

    pub fn set_extra_body(&mut self, extra_body: BytesMut) {
        self.extra_body = Some(extra_body);
    }

    // checkers
    pub fn has_connection_keep_alive(&self) -> Option<usize> {
        self.message_head
            .header_map()
            .has_key_and_value(CONNECTION, KEEP_ALIVE)
    }

    pub fn has_proxy_connection(&self) -> Option<usize> {
        self.message_head
            .header_map()
            .header_key_position(PROXY_CONNECTION)
    }

    // Normalize
    pub fn normalize(&mut self) {
        if let Some(pos) = self.has_connection_keep_alive() {
            self.update_header_value_on_position(pos, CLOSE);
        }
        if let Some(pos) = self.has_proxy_connection() {
            self.remove_header_on_position(pos);
        }
        self.remove_header_on_key(WS_EXT);
    }

    pub fn decode(
        &mut self,
        buf: &mut BytesMut,
    ) -> Result<(), MultiDecompressErrorReason> {
        decompress(self, buf)
    }
}

impl<T> Frame for OneOne<T>
where
    T: InfoLine,
{
    fn into_bytes(self) -> BytesMut {
        let mut header = self.message_head.into_bytes();
        if let Some(body) = self.body {
            let body = match body {
                Body::Raw(body) => body,
                Body::Chunked(items) => {
                    partial_chunked_to_raw(items).unwrap_or_default()
                }
            };
            header.unsplit(body);
        }
        header
    }
}

pub fn partial_chunked_to_raw(vec_body: Vec<ChunkType>) -> Option<BytesMut> {
    let mut iter = vec_body
        .into_iter()
        .map(|c| c.into_bytes());
    let mut body = iter.next()?;

    for chunk in iter {
        body.unsplit(chunk);
    }

    Some(body)
}
