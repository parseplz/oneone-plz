use std::borrow::Cow;

use header_plz::{
    Request,
    methods::{CONNECT, Method},
};

use super::OneOne;

// OneOne request methods
impl OneOne<Request> {
    pub fn is_connect_request(&self) -> bool {
        matches!(self.message_head.infoline().method(), CONNECT)
    }

    pub fn method_as_string(&self) -> Cow<str> {
        String::from_utf8_lossy(self.message_head.infoline().method())
    }

    pub fn method_as_enum(&self) -> Method {
        self.message_head.infoline().method().into()
    }

    pub fn uri_as_string(&self) -> Cow<str> {
        self.message_head.infoline().uri_as_string()
    }
}
