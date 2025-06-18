use crate::oneone::OneOne;
use body_plz::variants::{
    Body,
    chunked::{ChunkType, total_chunk_size},
};
use bytes::BytesMut;

use header_plz::{
    body_headers::parse::ParseBodyHeaders,
    const_headers::TRAILER,
    message_head::{MessageHead, info_line::InfoLine},
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

pub fn chunked_to_raw<T>(one: &mut OneOne<T>, buf: &mut BytesMut)
where
    T: InfoLine,
    MessageHead<T>: ParseBodyHeaders,
{
    let body = one.get_body().into_chunks();
    buf.reserve(total_chunk_size(&body));
    let mut new_body = buf.split();
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
}

// Partial chunked body
pub fn partial_chunked_to_raw(vec_body: Vec<ChunkType>) -> Option<BytesMut> {
    let mut iter = vec_body.into_iter().map(|c| c.into_bytes());
    let mut body = iter.next()?;

    for chunk in iter {
        body.unsplit(chunk);
    }

    Some(body)
}

#[cfg(test)]
mod test {
    use buffer_plz::{Cursor, Event};
    use header_plz::Request;
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
}
