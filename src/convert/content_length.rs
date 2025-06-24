use header_plz::{
    body_headers::parse::ParseBodyHeaders,
    const_headers::CONTENT_LENGTH,
    message_head::{MessageHead, info_line::InfoLine},
};

use crate::oneone::OneOne;

pub fn update_content_length<T>(one: &mut OneOne<T>, len: usize)
where
    T: InfoLine,
    MessageHead<T>: ParseBodyHeaders,
{
    let len_string = len.to_string();
    // 1. If cl present update cl
    match one.has_header_key(CONTENT_LENGTH) {
        Some(pos) => {
            one.update_header_value_on_position(pos, &len_string);
        }
        None => {
            // 2. else add new cl
            one.add_header(CONTENT_LENGTH, len_string.as_str());
        }
    }
}
