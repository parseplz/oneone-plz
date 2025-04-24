use body_plz::body_struct::Body;
use bytes::BytesMut;
use header_plz::{
    body_headers::{BodyHeader, parse::ParseBodyHeaders},
    const_headers::{CONNECTION, KEEP_ALIVE, PROXY_CONNECTION, TRAILER},
    error::HeaderReadError,
    header_map::{HeaderMap, header::Header},
    header_struct::HeaderStruct,
    info_line::InfoLine,
};
use protocol_traits_plz::Frame;

mod request;
mod response;
mod update;

#[cfg_attr(any(test, debug_assertions), derive(Debug, PartialEq, Eq))]
pub struct OneOne<T>
where
    T: InfoLine,
{
    header_struct: HeaderStruct<T>,
    body_headers: Option<BodyHeader>,
    body: Option<Body>,
}

impl<T> OneOne<T>
where
    T: InfoLine,
    HeaderStruct<T>: ParseBodyHeaders,
{
    pub fn new(buf: BytesMut) -> Result<Self, HeaderReadError> {
        let header_struct = HeaderStruct::<T>::new(buf)?;
        let body_headers = header_struct.parse_body_headers();
        Ok(OneOne {
            header_struct,
            body_headers,
            body: None,
        })
    }

    // Header Related methods
    pub fn infoline_as_mut(&mut self) -> &mut T {
        self.header_struct.infoline_as_mut()
    }

    pub fn header_struct(&self) -> &HeaderStruct<T> {
        &self.header_struct
    }

    pub fn header_map_as_mut(&mut self) -> &mut HeaderMap {
        self.header_struct.header_map_as_mut()
    }

    pub fn has_header_key(&self, key: &str) -> Option<usize> {
        self.header_struct.header_map().has_header_key(key)
    }

    pub fn add_header(&mut self, key: &str, value: &str) {
        let header: Header = (key, value).into();
        self.header_struct.header_map_as_mut().add_header(header);
    }

    pub fn has_trailers(&self) -> bool {
        self.header_struct
            .header_map()
            .has_header_key(TRAILER)
            .is_some()
    }

    pub fn value_for_key(&self, key: &str) -> Option<&str> {
        self.header_struct.header_map().value_for_key(key)
    }

    // Body Headers Related
    pub fn body_headers(&self) -> &Option<BodyHeader> {
        &self.body_headers
    }

    pub fn body_headers_as_mut(&mut self) -> &mut Option<BodyHeader> {
        &mut self.body_headers
    }

    // Body Related
    pub fn set_body(&mut self, body: Body) {
        self.body = Some(body);
    }

    pub fn get_body(&mut self) -> Body {
        self.body.take().unwrap()
    }

    pub fn body(&self) -> Option<&Body> {
        self.body.as_ref()
    }

    pub fn body_as_mut(&mut self) -> Option<&mut Body> {
        self.body.as_mut()
    }

    pub fn has_connection_keep_alive(&self) -> Option<usize> {
        self.header_struct
            .header_map()
            .has_key_and_value(CONNECTION, KEEP_ALIVE)
    }

    pub fn has_proxy_connection(&self) -> Option<usize> {
        self.header_struct
            .header_map()
            .has_header_key(PROXY_CONNECTION)
    }
}

impl<T> Frame for OneOne<T>
where
    T: InfoLine,
{
    fn into_data(self) -> BytesMut {
        let mut header = self.header_struct.into_data();
        if let Some(Body::Raw(body)) = self.body {
            header.unsplit(body);
        }
        header
    }
}
