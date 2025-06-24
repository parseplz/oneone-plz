use body_plz::variants::Body;
use bytes::BytesMut;
use header_plz::{
    body_headers::{BodyHeader, parse::ParseBodyHeaders},
    const_headers::CONTENT_LENGTH,
    message_head::{MessageHead, info_line::InfoLine},
};

use crate::oneone::OneOne;

pub fn add_body_and_update_cl<T>(
    one: &mut OneOne<T>,
    body: BytesMut,
    body_headers: Option<BodyHeader>,
) where
    T: InfoLine,
    MessageHead<T>: ParseBodyHeaders,
{
    if !body.is_empty() {
        update_content_length(one, body.len());
    }

    if let Some(bh) = body_headers {
        one.body_headers_as_mut().replace(bh);
    }
    one.set_body(Body::Raw(body));
}

pub fn update_content_length<T>(one: &mut OneOne<T>, len: usize)
where
    T: InfoLine,
    MessageHead<T>: ParseBodyHeaders,
{
    let len_string = len.to_string();
    match one.has_header_key(CONTENT_LENGTH) {
        Some(pos) => one.update_header_value_on_position(pos, &len_string),
        None => one.add_header(CONTENT_LENGTH, len_string.as_str()),
    }
}
