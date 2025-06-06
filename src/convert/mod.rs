use body_plz::variants::Body;
use bytes::BytesMut;
pub mod content_length;

pub mod chunked;
pub mod decompress;
use chunked::chunked_to_raw;
use content_length::update_content_length;
use decompress::*;
use header_plz::{
    body_headers::{BodyHeader, parse::ParseBodyHeaders},
    const_headers::{CONTENT_ENCODING, TRANSFER_ENCODING},
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

pub fn convert_body<T>(
    mut one: OneOne<T>,
    mut extra_body: Option<BytesMut>,
    buf: &mut BytesMut,
) -> Result<OneOne<T>, DecompressError>
where
    T: InfoLine,
    MessageHead<T>: ParseBodyHeaders,
{
    // 1. If chunked body convert chunked to CL
    if let Some(Body::Chunked(_)) = one.body() {
        one = chunked_to_raw(one, buf);
    }
    let mut body = one.get_body().into_bytes().unwrap();

    // 2. Transfer Encoding
    if let Some(BodyHeader {
        transfer_encoding: Some(encodings),
        ..
    }) = one.body_headers()
    {
        body = decompress_body(body, extra_body.take(), encodings, buf)?;
    }
    one.header_map_as_mut()
        .remove_header_on_key(TRANSFER_ENCODING);

    // 2. Content Encoding
    if let Some(BodyHeader {
        content_encoding: Some(encodings),
        ..
    }) = one.body_headers()
    {
        body = decompress_body(body, extra_body.take(), encodings, buf)?;
        // 3. Remove Content-Encoding
        one.header_map_as_mut()
            .remove_header_on_key(CONTENT_ENCODING);
    }

    if let Some(extra) = extra_body {
        body.unsplit(extra);
    }

    // 4. Update Content-Length
    if !body.is_empty() {
        update_content_length(&mut one, body.len());
    }

    one.set_body(Body::Raw(body));
    Ok(one)
}

#[cfg(test)]
mod test {
    use crate::{error::HttpReadError, state::State};
    use buffer_plz::{Cursor, Event};
    use bytes::BytesMut;
    use header_plz::info_line::response::Response;
    use protocol_traits_plz::{Frame, Step};

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
        state = state.try_next(event).unwrap();
        let event = Event::End(&mut cbuf);
        state = state.try_next(event).unwrap();
        match state {
            State::End(_) => {
                let data = state.try_into_frame().unwrap().into_bytes();
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
        state = state.try_next(event).unwrap();
        let event = Event::End(&mut cbuf);
        let result = state.try_next(event);
        if let Err(HttpReadError::ContentLengthPartial(oneone, buf)) = result {
            let data = oneone.into_bytes();
            assert_eq!(data, &res[..res.len() - 1]);
            assert_eq!(buf, "h");
        } else {
            panic!()
        }
    }
}
