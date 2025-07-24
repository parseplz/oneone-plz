use body_plz::variants::Body;
use decompression_plz::DecompressTrait;
use header_plz::{HeaderMap, InfoLine, body_headers::BodyHeader};

use crate::oneone::OneOne;

impl<T> DecompressTrait for OneOne<T>
where
    T: InfoLine,
{
    fn get_body(&mut self) -> Body {
        self.body.take().unwrap()
    }

    fn set_body(&mut self, body: Body) {
        self.body = Some(body);
    }

    fn body_headers_as_mut(&mut self) -> &mut Option<BodyHeader> {
        &mut self.body_headers
    }

    fn header_map(&self) -> &HeaderMap {
        self.message_head.header_map()
    }

    fn header_map_as_mut(&mut self) -> &mut HeaderMap {
        self.message_head.header_map_as_mut()
    }
}
