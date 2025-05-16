use std::borrow::Cow;

use body_plz::body_struct::Body;
use header_plz::info_line::response::Response;

use super::OneOne;

// OneOne response methods
impl OneOne<Response> {
    pub fn status_code(&self) -> Cow<str> {
        String::from_utf8_lossy(self.message_head.infoline().status())
    }

    pub fn content_length(&self) -> usize {
        if let Some(Body::Raw(body)) = &self.body {
            return body.len();
        }
        0
    }
}
