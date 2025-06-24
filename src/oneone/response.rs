use std::borrow::Cow;

use header_plz::Response;

use super::OneOne;

// OneOne response methods
impl OneOne<Response> {
    pub fn status_code(&self) -> Cow<str> {
        String::from_utf8_lossy(self.message_head.infoline().status())
    }
}
