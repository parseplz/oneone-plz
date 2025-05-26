use body_plz::body_struct::Body;

mod chunked;
mod decompress;
use chunked::convert_chunked;
use decompress::*;
pub mod error;
use error::*;
use header_plz::{
    body_headers::{BodyHeader, parse::ParseBodyHeaders},
    const_headers::{CONTENT_ENCODING, CONTENT_LENGTH, TRANSFER_ENCODING},
    info_line::InfoLine,
    message_head::MessageHead,
};

use crate::oneone::OneOne;

/* Description:
 *      Convert raw h11 to decompressed/dechunked h11.
 *
 * Steps:
 *      1. If chunked body convert chunked to CL, by calling
 *         convert_chunked() and remove Transfer-Encoding header.
 *
 *      2. If transfer encoding and content encoding is present, decompress
 *         the body by calling decompress_data() with body and vec of
 *         encodings.
 *
 *      3. Remove their corresponding headers.
 *
 *      4. Update content length header.
 */

pub fn convert_one_dot_one_body<T>(mut one: OneOne<T>) -> Result<OneOne<T>, DecompressError>
where
    T: InfoLine,
    MessageHead<T>: ParseBodyHeaders,
{
    // 1. If chunked body convert chunked to CL
    if let Some(Body::Chunked(_)) = one.body() {
        let body = one.get_body().into_chunks();
        one = convert_chunked(one, body);
        one.header_map_as_mut()
            .remove_header_on_key(TRANSFER_ENCODING);
    }
    let mut body = one.get_body().into_data().unwrap();

    // 2. Transfer Encoding
    if let Some(BodyHeader {
        transfer_encoding: Some(encodings),
        ..
    }) = one.body_headers()
    {
        body = decompress_data(body, encodings)?;
        one.header_map_as_mut()
            .remove_header_on_key(TRANSFER_ENCODING);
    }

    // 2. Content Encoding
    if let Some(BodyHeader {
        content_encoding: Some(encodings),
        ..
    }) = one.body_headers()
    {
        body = decompress_data(body, encodings)?;
        // 3. Remove Content-Encoding
        one.header_map_as_mut()
            .remove_header_on_key(CONTENT_ENCODING);
    }

    // 4. Update Content-Length
    if !body.is_empty() {
        update_content_length(&mut one, body.len());
    }

    one.set_body(Body::Raw(body));
    Ok(one)
}

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

#[cfg(test)]
mod test {
    use buffer_plz::{Cursor, Event};
    use bytes::BytesMut;
    use header_plz::info_line::{request::Request, response::Response};
    use protocol_traits_plz::{Frame, Step};

    use crate::{error::HttpReadError, state::State};

    use super::*;

    #[test]
    fn test_convert_no_cl() {
        let res = "HTTP/1.1 200 OK\r\n\
                   Host: reqbin.com\r\n\
                   Content-Type: text/plain\r\n\r\n\
                   MozillaDeveloperNetwork";
        let verify = "HTTP/1.1 200 OK\r\n\
                      Host: reqbin.com\r\n\
                      Content-Type: text/plain\r\n\
                      Content-Length: 23\r\n\r\n\
                      MozillaDeveloperNetwork";

        let mut buf: BytesMut = res.into();
        let mut cbuf = Cursor::new(&mut buf);
        let mut state: State<Response> = State::new();
        let event = Event::Read(&mut cbuf);
        state = state.next(event).unwrap();
        let event = Event::End(&mut cbuf);
        state = state.next(event).unwrap();
        match state {
            State::End(_) => {
                let data = state.into_frame().unwrap().into_data();
                assert_eq!(data, verify);
            }
            _ => {
                panic!()
            }
        }
    }
    #[test]
    fn test_convert_cl_partial() {
        let res = "HTTP/1.1 200 OK\r\n\
                   Host: reqbin.com\r\n\
                   Content-Type: text/plain\r\n\
                   Content-Length: 100\r\n\r\n\
                   h";

        let mut buf: BytesMut = res.into();
        let mut cbuf = Cursor::new(&mut buf);
        let mut state: State<Response> = State::new();
        let event = Event::Read(&mut cbuf);
        state = state.next(event).unwrap();
        let event = Event::End(&mut cbuf);
        let result = state.next(event);
        if let Err(HttpReadError::ContentLengthPartial(oneone, buf)) = result {
            let data = oneone.into_data();
            assert_eq!(data, &res[..res.len() - 1]);
            assert_eq!(buf, "h");
        } else {
            panic!()
        }
    }
}
