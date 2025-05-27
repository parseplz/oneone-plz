use body_plz::{
    reader::{chunked_reader::ChunkReaderState, content_length_reader::read_content_length},
    variants::{Body, chunked::ChunkType},
};
use buffer_plz::Event;
use bytes::BytesMut;
use header_plz::{
    body_headers::{parse::ParseBodyHeaders, transfer_types::TransferType},
    const_headers::{CLOSE, WS_EXT},
    info_line::InfoLine,
    message_head::MessageHead,
    reader::find_message_head_end,
};
use protocol_traits_plz::Step;

use crate::{
    convert::{convert_one_dot_one_body, error::DecompressError},
    error::HttpReadError,
    oneone::OneOne,
};

#[cfg_attr(test, derive(PartialEq, Eq))]
#[derive(Debug)]
pub enum State<T>
where
    T: InfoLine,
{
    ReadMessageHead,
    ReadBodyContentLength(OneOne<T>, usize),
    ReadBodyContentLengthExtra(OneOne<T>),
    ReadBodyChunked(OneOne<T>, ChunkReaderState),
    ReadBodyChunkedExtra(OneOne<T>),
    ReadBodyClose(OneOne<T>),
    End(OneOne<T>),
}

impl<T> State<T>
where
    T: InfoLine,
    MessageHead<T>: ParseBodyHeaders,
{
    #[allow(clippy::new_without_default)]
    pub fn new() -> State<T> {
        State::<T>::ReadMessageHead
    }

    /* Steps:
     *      1. Build OneOne from headers
     *
     *      2. Match body_headers
     *          a. None => End
     *          b. Some, match transfer_type
     *              1. ContentLength,
     *                  a. size == 0    => End
     *                  b. size != 0    => ReadBodyContentLength(size)
     *              2. Chunked          => ReadBodyChunked
     *              3. Close            => ReadBodyClose
     *              3. Unknown          => End
     *
     *      3. Default => End
     */

    #[allow(clippy::result_large_err)]
    fn build_oneone(headers: BytesMut) -> Result<Self, HttpReadError<T>> {
        let mut one = OneOne::new(headers)?;
        let next_state = match one.body_headers() {
            None => Self::End(one),
            Some(body_headers) => match body_headers.transfer_type {
                Some(tt) => match tt {
                    TransferType::ContentLength(size) => {
                        if size == 0 {
                            Self::End(one)
                        } else {
                            Self::ReadBodyContentLength(one, size)
                        }
                    }
                    TransferType::Chunked => {
                        one.set_body(Body::Chunked(Vec::new()));
                        Self::ReadBodyChunked(one, ChunkReaderState::ReadSize)
                    }
                    TransferType::Close => Self::ReadBodyClose(one),
                    TransferType::Unknown => Self::End(one),
                },
                None => Self::End(one),
            },
        };
        Ok(next_state)
    }
}

impl<T> Step<OneOne<T>> for State<T>
where
    T: InfoLine,
    MessageHead<T>: ParseBodyHeaders,
{
    type StateError = HttpReadError<T>;
    type FrameError = DecompressError;

    fn next(mut self, event: Event) -> Result<Self, Self::StateError> {
        match (self, event) {
            /* ReadHeader , Read
             *      a. if read_header() is true,
             *          1. split buf at current position to get raw headers.
             *          2. Build OneOne
             *          3. if there is remaining data in buf, call next() with
             *         remaining data.
             *      b. false, remain in same state.
             */
            (State::ReadMessageHead, Event::Read(buf)) => match find_message_head_end(buf) {
                true => {
                    let raw_headers = buf.split_at_current_pos();
                    self = State::build_oneone(raw_headers)?;
                    if buf.len() > 0 {
                        self = self.next(Event::Read(buf))?;
                    }
                    Ok(self)
                }
                false => Ok(Self::ReadMessageHead),
            },

            // ReadHeader , End -> Failed [partial]
            (State::ReadMessageHead, Event::End(buf)) => {
                Err(HttpReadError::Unparsed(buf.into_inner()))?
            }

            /* ReadBodyContentLength(size) , event
             *      match content_length_read(buf, size)
             *      1. true =>  a. if no extra data, State::End
             *                  b. else, match event
             *                      1. Read =>
             *                         State::ReadBodyContentLengthExtra
             *                      2. End => call next on
             *                              State::ReadBodyContentLengthExtra
             *
             *      2. false => match event
             *                  a. Read => remain in same state
             *                  b. End => Err(ContentLengthPartial) [partial]
             */
            (State::ReadBodyContentLength(mut oneone, mut size), mut event) => match event {
                Event::Read(ref mut buf) | Event::End(ref mut buf) => {
                    match read_content_length(buf, &mut size) {
                        true => {
                            oneone.set_body(Body::Raw(buf.split_at_current_pos()));
                            if buf.len() > 0 {
                                let next_state = State::ReadBodyContentLengthExtra(oneone);
                                match event {
                                    Event::Read(_) => Ok(next_state),
                                    Event::End(_) => next_state.next(event),
                                }
                            } else {
                                Ok(State::End(oneone))
                            }
                        }
                        false => match event {
                            Event::Read(_) => Ok(State::ReadBodyContentLength(oneone, size)),
                            Event::End(buf) => Err(HttpReadError::ContentLengthPartial(
                                oneone,
                                buf.split_at_current_pos(),
                            ))?,
                        },
                    }
                }
            },

            /* ReadBodyContentLengthExtra(oneone), Read
             *      i.e.More data than needed
             *      remain in same state i.e. read until EOF is reached
             */
            (State::ReadBodyContentLengthExtra(oneone), Event::Read(_)) => {
                Ok(State::ReadBodyContentLengthExtra(oneone))
            }

            /* ReadBodyContentLengthExtra(oneone), End
             *      1. add extra data to body
             *      2. transition to State::End
             */
            (State::ReadBodyContentLengthExtra(mut oneone), Event::End(buf)) => {
                let extra_body = buf.into_inner();
                if let Some(Body::Raw(raw)) = oneone.body_as_mut() {
                    raw.unsplit(extra_body);
                } else {
                    oneone.set_body(Body::Raw(extra_body));
                }
                Ok(State::End(oneone))
            }
            /* Chunked , Read
             *      1. Call next() on chunk_state with buf
             *      2. if Some(chunk) is returned, add to body.
             *      3. match chunk_state
             *          a. LastChunk, check trailer header present
             *              1. true     => ChunkReaderState::ReadTrailers
             *              2. false    => ChunkReaderState::EndCRLF
             *          b. End,
             *              1. if no extra data, State::End
             *              2. if extra data, State::ReadBodyChunkedExtra
             *          c. Failed, State::Failed
             *          d. other states, continue
             *      4. if None, remain in same state.
             */
            (State::ReadBodyChunked(mut oneone, mut chunk_state), Event::Read(buf)) => loop {
                match chunk_state.next(buf) {
                    Some(chunk_to_add) => {
                        oneone.body_as_mut().unwrap().push_chunk(chunk_to_add);
                        match chunk_state {
                            ChunkReaderState::LastChunk => {
                                chunk_state = if oneone.has_trailers() {
                                    ChunkReaderState::ReadTrailers
                                } else {
                                    ChunkReaderState::EndCRLF
                                };
                                continue;
                            }
                            ChunkReaderState::End => {
                                return if buf.len() > 0 {
                                    Ok(State::ReadBodyChunkedExtra(oneone))
                                } else {
                                    Ok(State::End(oneone))
                                };
                            }
                            ChunkReaderState::Failed(e) => return Err(e.into()),
                            _ => continue,
                        }
                    }
                    None => {
                        return Ok(State::ReadBodyChunked(oneone, chunk_state));
                    }
                }
            },

            // Chunked , End [partial]
            (State::ReadBodyChunked(mut oneone, chunked_state), Event::End(buf)) => Err(
                HttpReadError::ChunkReaderNotEnoughData(oneone, buf.split_at_current_pos()),
            ),

            /* ReadBodyChunkedExtra(OneOne<T>), Read
             *      remain in same state i.e. read until EOF is reached
             */
            (State::ReadBodyChunkedExtra(oneone), Event::Read(_)) => {
                Ok(State::ReadBodyChunkedExtra(oneone))
            }

            /* ReadBodyChunkedExtra(OneOne<T>), End => End
             *      1. add extra data to body
             *      2. transition to State::End
             */
            (State::ReadBodyChunkedExtra(mut oneone), Event::End(buf)) => {
                let extra_chunk = ChunkType::Extra(buf.into_inner());
                oneone.body_as_mut().unwrap().push_chunk(extra_chunk);
                Ok(State::End(oneone))
            }

            /* ReadBodyClose, Read => State::ReadBodyClose
             *      1. Remain in same state.
             *      2. Read until Event::End
             */
            (State::ReadBodyClose(oneone), Event::Read(_)) => Ok(Self::ReadBodyClose(oneone)),

            /* ReadBodyClose, End => State::End
             *      Split the buf until filled and set the body to Raw
             */
            (State::ReadBodyClose(mut oneone), Event::End(buf)) => {
                oneone.set_body(Body::Raw(buf.into_inner()));
                Ok(State::End(oneone))
            }

            /* End, Event - extra data after parsing completed
             *      1. match body
             *          a. None                 => State::ReadBodyClose
             *          b. Some(Body::Raw)      => State::ReadBodyContentLengthExtra
             *          c. Some(Body::Chunked)  => State::ReadBodyChunkedExtra
             *
             *      2. match event
             *          a. Read => above selected state
             *          b. End  => call next() again to transition to End from
             *                     the above selected state
             */
            (State::End(oneone), event) => {
                self = match oneone.body() {
                    None => State::ReadBodyClose(oneone),
                    Some(Body::Raw(_)) => State::ReadBodyContentLengthExtra(oneone),
                    Some(Body::Chunked(_)) => State::ReadBodyChunkedExtra(oneone),
                };
                match event {
                    Event::Read(_) => Ok(self),
                    Event::End(_) => self.next(event),
                }
            }
        }
    }

    fn is_ended(&self) -> bool {
        matches!(self, Self::End(_))
    }

    /* Description:
     *      Method to convert State to OneOne<T>
     *
     * Steps:
     *      1. if State is End
     *      2. if body is present call convert_one_dot_one() to decompress
     *         or dechunk oneone.
     *      3. Change Connection keep-alive header to close
     *      4. If has Proxy-Connection header, remove it
     *      5. Remove ws extension header
     *      Ok(OneOne<T>)
     */

    fn into_frame(self) -> Result<OneOne<T>, DecompressError> {
        if let Self::End(mut one) = self {
            if one.body().is_some() {
                one = convert_one_dot_one_body(one)?;
            }
            if let Some(pos) = one.has_connection_keep_alive() {
                one.header_map_as_mut()
                    .change_header_value_on_pos(pos, CLOSE);
            }
            if let Some(pos) = one.has_proxy_connection() {
                one.header_map_as_mut().remove_header_on_pos(pos);
            }
            one.header_map_as_mut().remove_header_on_key(WS_EXT);
            return Ok(one);
        }
        unreachable!();
    }
}

#[cfg(test)]
mod tests {
    use buffer_plz::Cursor;
    use bytes::BytesMut;
    use header_plz::info_line::{request::Request, response::Response};
    use protocol_traits_plz::Frame;

    use super::*;

    #[test]
    fn test_oneone_state_read_chunked_convert() {
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
    fn test_oneone_state_get_success() {
        let req = "GET /echo HTTP/1.1\r\n\
                   Host: reqbin.com\r\n\r\n";
        let mut buf: BytesMut = req.into();
        let mut cbuf = Cursor::new(&mut buf);
        let mut state: State<Request> = State::new();
        let event = Event::Read(&mut cbuf);
        state = state.next(event).unwrap();
        match state {
            State::End(one) => {
                assert_eq!(one.message_head().infoline().method(), b"GET");
            }
            _ => {
                panic!()
            }
        }
    }

    #[test]
    fn test_oneone_state_get_partial() {
        let req = "GET /echo HTTP/1.1\r\n";
        let mut buf: BytesMut = req.into();
        let mut cbuf = Cursor::new(&mut buf);
        let mut state: State<Request> = State::new();
        let event = Event::Read(&mut cbuf);
        state = state.next(event).unwrap();
        assert!(matches!(state, State::ReadMessageHead));
        assert_eq!(cbuf.position(), 17);
        let event = Event::End(&mut cbuf);
        let result = state.next(event);
        assert!(matches!(result, Err(HttpReadError::Unparsed(_))));
    }

    #[test]
    fn test_oneone_state_post_success() {
        let req = "POST /echo HTTP/1.1\r\n\
                   Host: reqbin.com\r\n\
                   content-length: 7\r\n\r\n\
                   Mozilla";
        let mut buf: BytesMut = req.into();
        let mut cbuf = Cursor::new(&mut buf);
        let mut state: State<Request> = State::new();
        let event = Event::Read(&mut cbuf);
        state = state.next(event).unwrap();
        match state {
            State::End(one) => {
                assert_eq!(one.method_as_string(), "POST");
                assert_eq!(one.uri_as_string(), "/echo");
            }
            _ => {
                panic!()
            }
        }
    }

    #[test]
    fn test_oneone_state_response_success() {
        let req = "HTTP/1.1 200 OK\r\n\
                   Host: reqbin.com\r\n\
                   content-length: 12\r\n\r\n\
                   Hello, World";
        let mut buf: BytesMut = req.into();
        let org_range = buf.as_ptr_range();
        let mut cbuf = Cursor::new(&mut buf);
        let mut state: State<Response> = State::new();
        let event = Event::Read(&mut cbuf);
        state = state.next(event).unwrap();
        match state {
            State::End(one) => {
                assert_eq!(one.status_code(), "200");
                let result = one.into_data();
                assert_eq!(result.as_ptr_range(), org_range);
            }
            _ => {
                panic!()
            }
        }
    }

    #[test]
    fn test_oneone_state_partial_header() {
        let req = "GET /partial HTTP/1.1\r\n\
                   Host: example.com\r\n";
        let mut buf = BytesMut::from(req);
        let mut cbuf = Cursor::new(&mut buf);
        let mut state: State<Request> = State::new();
        let event = Event::Read(&mut cbuf);
        state = state.next(event).unwrap();
        assert!(matches!(state, State::ReadMessageHead));
        assert_eq!(cbuf.position(), 39);
    }

    #[test]
    fn test_oneone_state_chunked_no_trailer() {
        let req = "POST /chunked HTTP/1.1\r\n\
                   Host: reqbin.com\r\n\
                   Transfer-Encoding: chunked\r\n\r\n\
                   7\r\n\
                   Mozilla\r\n\
                   0\r\n\r\n";
        let verify = "POST /chunked HTTP/1.1\r\n\
                      Host: reqbin.com\r\n\
                      Content-Length: 7\r\n\r\n\
                      Mozilla";
        let mut buf = BytesMut::from(req);
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
                panic!("Expected State::End, found {:?}", state);
            }
        }
    }

    #[test]
    fn test_oneone_state_empty_body() {
        // Test when the request/response has an empty body.
        let req = "POST /empty HTTP/1.1\r\n\
                   Host: reqbin.com\r\n\
                   Content-Length: 0\r\n\r\n";
        let mut buf = BytesMut::from(req);
        let org_range = buf.as_ptr_range();
        let mut cbuf = Cursor::new(&mut buf);
        let mut state: State<Request> = State::new();
        let event = Event::Read(&mut cbuf);
        state = state.next(event).unwrap();
        match state {
            State::End(_) => {
                let result = state.into_frame().unwrap().into_data();
                let result_range = result.as_ptr_range();
                assert_eq!(org_range, result_range);
            }
            _ => {
                panic!("Expected State::End, found {:?}", state);
            }
        }
    }

    #[test]
    fn test_oneone_state_chunked_truncated() {
        let req = "POST /truncated HTTP/1.1\r\n\
                   Transfer-Encoding: chunked\r\n\r\n\
                   7\r\n\
                   Mozilla\r\n\
                   0\r\n";
        let mut buf = BytesMut::from(req);
        let mut cbuf = Cursor::new(&mut buf);
        let mut state: State<Request> = State::new();
        let event = Event::Read(&mut cbuf);
        state = state.next(event).unwrap();
        // Incomplete data, expect ReadBodyChunked state with remaining data.
        assert!(matches!(state, State::ReadBodyChunked(_, _)));
    }

    #[test]
    fn test_oneone_state_read_body_close() {
        let req = "HTTP/1.1 200 OK\r\n\
                   Host: reqbin.com\r\n\
                   Content-Type: text/plain\r\n\r\n\
                   HolaAmigo";
        let mut buf = BytesMut::from(req);
        let mut cbuf = Cursor::new(&mut buf);
        let mut state: State<Response> = State::new();
        let event = Event::Read(&mut cbuf);
        state = state.next(event).unwrap();
        assert!(matches!(state, State::ReadBodyClose(_)));
        let event = Event::End(&mut cbuf);
        state = state.next(event).unwrap();
        assert!(matches!(state, State::End(_)));
        let one = state.into_frame().unwrap();
        assert_eq!(one.status_code(), "200");
        let result = one.into_data();
        let verify = "HTTP/1.1 200 OK\r\n\
                      Host: reqbin.com\r\n\
                      Content-Type: text/plain\r\n\
                      Content-Length: 9\r\n\r\n\
                      HolaAmigo";
        assert_eq!(result, verify);
    }
}
