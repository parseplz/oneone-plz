use header_plz::{
    body_headers::parse::ParseBodyHeaders, const_headers::CONTENT_LENGTH, info_line::InfoLine,
    message_head::MessageHead,
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
            one.header_map_as_mut()
                .change_header_value_on_pos(pos, &len_string);
        }
        None => {
            // 2. else add new cl
            let content_length_header = (CONTENT_LENGTH, len_string.as_str()).into();
            one.header_map_as_mut().add_header(content_length_header);
        }
    }
}
