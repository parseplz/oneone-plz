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
            one.header_map_as_mut()
                .update_header_value_on_position(pos, &len_string);
        }
        None => {
            // 2. else add new cl
            let content_length_header = (CONTENT_LENGTH, len_string.as_str()).into();
            one.header_map_as_mut().add_header(content_length_header);
        }
    }
}

#[cfg(test)]
mod test {
    use bytes::BytesMut;
    use header_plz::Response;
    use protocol_traits_plz::Frame;

    use super::*;

    #[test]
    fn test_update_content_length_no_cl() {
        let input = "HTTP/1.1 200 OK\r\n\
                     Host: reqbin.com\r\n\
                     Content-Type: text/plain\r\n\r\n";

        let mut one: OneOne<Response> =
            OneOne::try_from_message_head_buf(BytesMut::from(input)).unwrap();
        update_content_length(&mut one, 23);
        let verify = "HTTP/1.1 200 OK\r\n\
                      Host: reqbin.com\r\n\
                      Content-Type: text/plain\r\n\
                      Content-Length: 23\r\n\r\n";
        assert_eq!(one.into_bytes(), verify);
    }

    #[test]
    fn test_update_content_length_with_cl() {
        let input = "HTTP/1.1 200 OK\r\n\
                     Host: reqbin.com\r\n\
                     Content-Type: text/plain\r\n\
                     Content-Length: 23\r\n\r\n";

        let mut one: OneOne<Response> =
            OneOne::try_from_message_head_buf(BytesMut::from(input)).unwrap();
        update_content_length(&mut one, 27);
        let verify = "HTTP/1.1 200 OK\r\n\
                      Host: reqbin.com\r\n\
                      Content-Type: text/plain\r\n\
                      Content-Length: 27\r\n\r\n";
        assert_eq!(one.into_bytes(), verify);
    }
}
