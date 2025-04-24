use body_plz::body_struct::{Body, ChunkedBody, total_chunk_size};
use bytes::BytesMut;

mod decompress;
use decompress::*;
pub mod error;
use error::*;
use header_plz::{
    body_headers::{BodyHeader, parse::ParseBodyHeaders},
    const_headers::{CONTENT_ENCODING, CONTENT_LENGTH, TRAILER, TRANSFER_ENCODING},
    header_struct::HeaderStruct,
    info_line::InfoLine,
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
 *          a. if header present, update it.
 *          b. else create it.
 */

pub fn convert_one_dot_one_body<T>(mut one: OneOne<T>) -> Result<OneOne<T>, DecompressError>
where
    T: InfoLine,
    HeaderStruct<T>: ParseBodyHeaders,
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

    // 4. Update Cl
    let len: String = body.len().to_string();

    // 4.a. If cl present update cl
    match one.has_header_key(CONTENT_LENGTH) {
        Some(pos) => {
            one.header_map_as_mut()
                .change_header_value_on_pos(pos, &len);
        }
        _ => {
            // 4.b. else add new cl
            let content_length_header = (CONTENT_LENGTH, len.as_str()).into();
            one.header_map_as_mut().add_header(content_length_header);
        }
    }

    one.set_body(Body::Raw(body));
    Ok(one)
}

/* Description:
 *      Convert chunked body to content length.
 *
 * Steps:
 *      1. Combine ChunkedBody::Chunk into one body.
 *      2. If trailer is present,
 *          a. remove trailer header
 *          b. add trailer to header_map.
 */

fn convert_chunked<T>(mut one: OneOne<T>, vec_body: Vec<ChunkedBody>) -> OneOne<T>
where
    T: InfoLine,
    HeaderStruct<T>: ParseBodyHeaders,
{
    let mut new_body = BytesMut::with_capacity(total_chunk_size(&vec_body));
    vec_body.into_iter().for_each(|body| match body {
        // 1. Combine ChunkedBody::Chunk into one body.
        ChunkedBody::Chunk(data) => new_body.extend_from_slice(&data[..data.len() - 2]),
        // 2. If trailer is present,
        ChunkedBody::Trailers(trailer) => {
            // 2.a. Remove trailer header
            one.header_map_as_mut().remove_header_on_key(TRAILER);
            // 2.b. Add trailer to header_map
            let mut trailer_header = trailer.into_header_vec();
            one.header_map_as_mut()
                .headers_as_mut()
                .append(&mut trailer_header);
        }
        _ => {}
    });
    one.set_body(Body::Raw(new_body));
    one
}

#[cfg(test)]
mod test {
    use buffer_plz::{Cursor, Event};
    use header_plz::info_line::{request::Request, response::Response};
    use protocol_traits_plz::{Frame, Step};

    use crate::state::State;

    use super::*;

    #[test]
    fn test_convert_chunked() {
        let req = "POST /echo HTTP/1.1\r\n\
                  Host: reqbin.com\r\n\
                  Trailer: Some\r\n\
                  Transfer-Encoding: chunked\r\n\r\n\
                  7\r\n\
                  Mozilla\r\n\
                  9\r\n\
                  Developer\r\n\
                  7\r\n\
                  Network\r\n\
                  0\r\n\
                  Header: Val\r\n\
                  \r\n";
        let verify = "POST /echo HTTP/1.1\r\n\
                      Host: reqbin.com\r\n\
                      Header: Val\r\n\
                      Content-Length: 23\r\n\r\n\
                      MozillaDeveloperNetwork";
        let mut buf: BytesMut = req.into();
        let mut cbuf = Cursor::new(&mut buf);
        let mut state: State<Request> = State::new();
        let event = Event::Read(&mut cbuf);
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
    fn test_convert_modify_cl() {
        let res = "HTTP/1.1 200 OK\r\n\
                   Host: reqbin.com\r\n\
                   Content-Type: text/plain\r\n\
                   Content-Length: 100\r\n\r\n\
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
}
