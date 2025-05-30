use crate::oneone::OneOne;
use body_plz::variants::{
    Body,
    chunked::{ChunkType, total_chunk_size, total_chunk_size_unchecked},
};
use bytes::BytesMut;

use header_plz::{
    body_headers::parse::ParseBodyHeaders, const_headers::TRAILER, info_line::InfoLine,
    message_head::MessageHead,
};
/* Description:
 *      Convert chunked body to content length.
 *
 * Steps:
 *      1. Combine ChunkType::Chunk into one body.
 *      2. If trailer is present,
 *          a. remove trailer header
 *          b. add trailer to header_map.
 */

pub fn chunked_to_raw<T>(mut one: OneOne<T>) -> OneOne<T>
where
    T: InfoLine,
    MessageHead<T>: ParseBodyHeaders,
{
    let body = one.get_body().into_chunks();
    let mut new_body = BytesMut::with_capacity(total_chunk_size(&body));
    body.into_iter().for_each(|chunk| match chunk {
        // 1. Combine ChunkType::Chunk into one body.
        ChunkType::Chunk(data) => new_body.extend_from_slice(&data[..data.len() - 2]),
        // 2. If trailer is present,
        ChunkType::Trailers(trailer) => {
            // 2.a. Remove trailer header
            one.header_map_as_mut().remove_header_on_key(TRAILER);
            // 2.b. Add trailer to header_map
            let mut trailer_header = trailer.into_header_vec();
            one.header_map_as_mut()
                .headers_as_mut()
                .append(&mut trailer_header);
        }
        ChunkType::Extra(data) => new_body.extend_from_slice(&data),
        _ => {}
    });
    one.set_body(Body::Raw(new_body));
    one
}

// Partial chunked body
pub fn convert_chunked_unchecked(mut vec_body: Vec<ChunkType>) -> Option<BytesMut> {
    let mut iter = vec_body.into_iter().map(|c| c.into_data());
    let mut body = iter.next()?;

    for chunk in iter {
        body.unsplit(chunk);
    }

    Some(body)
}

#[cfg(test)]
mod test {
    use buffer_plz::{Cursor, Event};
    use header_plz::info_line::request::Request;
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
    fn test_convert_chunked_extra() {
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
}
