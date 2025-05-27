use crate::{parse_full_single, poll_first};
use buffer_plz::{Cursor, Event};
use bytes::BytesMut;
use header_plz::info_line::{request::Request, response::Response};
use oneone_plz::error::HttpReadError;
use oneone_plz::{oneone::OneOne, state::State};
use protocol_traits_plz::Frame;
use protocol_traits_plz::Step;

#[test]
fn test_response_chunked_one() {
    let input = "HTTP/1.1 200 OK\r\n\
                 Transfer-Encoding: chunked\r\n\r\n\
                 6\r\n\
                 hello \r\n\
                 5\r\n\
                 world\r\n\
                 0\r\n\r\n";
    let response = parse_full_single::<Response>(input.as_bytes());
    let verify = "HTTP/1.1 200 OK\r\n\
                  Content-Length: 11\r\n\r\n\
                  hello world";
    assert_eq!(response.into_data(), verify);
}

#[test]
fn test_response_chunked_multiple() {
    let chunks: &[&[u8]] = &[
        b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n5\r\nhello\r\n",
        b"5\r\nworld\r\n",
        b"0\r\n\r\n",
    ];

    let mut buf = BytesMut::from(chunks[0]);
    let mut cbuf = Cursor::new(&mut buf);
    let mut state = poll_first::<Response>(&mut cbuf);
    //let mut state: State<Response> = State::new();

    for &chunk in &chunks[1..] {
        state = state.next(Event::Read(&mut cbuf)).unwrap();
        assert!(matches!(state, State::ReadBodyChunked(_, _)));
        cbuf.as_mut().extend_from_slice(chunk);
    }

    state = state.next(Event::Read(&mut cbuf)).unwrap();
    assert!(matches!(state, State::End(_)));

    let response = state.into_frame().unwrap();
    assert_eq!(response.status_code(), "200");

    let expected = "HTTP/1.1 200 OK\r\n\
                    Content-Length: 10\r\n\r\n\
                    helloworld";
    assert_eq!(response.into_data(), expected);
}

#[test]
fn test_response_chunked_large() {
    let chunks: &[&[u8]] = &[
        b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n",
        &b"B\r\nhello world\r\n".repeat(100),
        b"0\r\n\r\n",
    ];

    let mut buf = BytesMut::from(chunks[0]);
    let mut cbuf = Cursor::new(&mut buf);
    let mut state: State<Response> = State::new();

    for &chunk in &chunks[1..] {
        state = state.next(Event::Read(&mut cbuf)).unwrap();
        cbuf.as_mut().extend_from_slice(chunk);
    }

    state = state.next(Event::Read(&mut cbuf)).unwrap();
    assert!(matches!(state, State::End(_)));

    let expected = "HTTP/1.1 200 OK\r\n\
                    Content-Length: 1100\r\n\r\n"
        .to_string()
        + &"hello world".repeat(100);
    let response = state.into_frame().unwrap();
    assert_eq!(response.status_code(), "200");
    assert_eq!(response.into_data(), expected);
}

#[test]
fn test_response_chunked_extra_single() {
    let input = "HTTP/1.1 200 OK\r\n\
                 Transfer-Encoding: chunked\r\n\r\n\
                 6\r\n\
                 hello \r\n\
                 6\r\n\
                 world \r\n\
                 0\r\n\r\n\
                 extra data";
    let mut buf = BytesMut::from(input);
    let mut cbuf = Cursor::new(&mut buf);
    let mut state: State<Response> = State::new();
    let event = Event::Read(&mut cbuf);
    state = state.next(event).unwrap();
    let event = Event::End(&mut cbuf);
    state = state.next(event).unwrap();
    assert!(matches!(state, State::End(_)));
    let response = state.into_frame().unwrap();
    let verify = "HTTP/1.1 200 OK\r\n\
                  Content-Length: 22\r\n\r\n\
                  hello world extra data";
    assert_eq!(response.into_data(), verify);
}

#[test]
fn test_response_chunked_extra_multiple() {
    let chunks: &[&[u8]] = &[
        b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n",
        b"6\r\nhello \r\n",
        b"0\r\n\r\n",
        b"extra data ",
    ];

    let mut buf = BytesMut::from(chunks[0]);
    let mut cbuf = Cursor::new(&mut buf);
    let mut state: State<Response> = State::new();

    for &chunk in &chunks[1..] {
        state = state.next(Event::Read(&mut cbuf)).unwrap();
        cbuf.as_mut().extend_from_slice(chunk);
    }

    cbuf.as_mut().extend_from_slice(b"added");
    state = state.next(Event::End(&mut cbuf)).unwrap();

    let response = state.into_frame().unwrap();
    let verify = "HTTP/1.1 200 OK\r\n\
                  Content-Length: 22\r\n\r\n\
                  hello extra data added";
    assert_eq!(response.into_data(), verify);
}

#[test]
fn test_response_chunked_extra_finished_single() {
    let input = "HTTP/1.1 200 OK\r\n\
                 Transfer-Encoding: chunked\r\n\r\n\
                 6\r\n\
                 hello \r\n\
                 0\r\n\r\n";

    let mut buf = BytesMut::from(input);
    let mut cbuf = Cursor::new(&mut buf);
    let mut state: State<Response> = State::new();
    let event = Event::Read(&mut cbuf);
    state = state.next(event).unwrap();
    assert!(matches!(state, State::End(_)));

    cbuf.as_mut().extend_from_slice(b"extra data added");
    state = state.next(Event::End(&mut cbuf)).unwrap();
    let response = state.into_frame().unwrap();
    let verify = "HTTP/1.1 200 OK\r\n\
                  Content-Length: 22\r\n\r\n\
                  hello extra data added";
    assert_eq!(response.into_data(), verify);
}

#[test]
fn test_response_chunked_extra_finished_multiple() {
    let input = "HTTP/1.1 200 OK\r\n\
                 Transfer-Encoding: chunked\r\n\r\n\
                 6\r\n\
                 hello \r\n\
                 0\r\n\r\n";

    let mut buf = BytesMut::from(input);
    let mut cbuf = Cursor::new(&mut buf);
    let mut state: State<Response> = State::new();
    let event = Event::Read(&mut cbuf);
    state = state.next(event).unwrap();
    assert!(matches!(state, State::End(_)));

    // extra 1
    cbuf.as_mut().extend_from_slice(b"extra data");
    state = state.next(Event::Read(&mut cbuf)).unwrap();
    assert!(matches!(state, State::ReadBodyChunkedExtra(_)));

    // extra 2
    cbuf.as_mut().extend_from_slice(b" added");
    state = state.next(Event::End(&mut cbuf)).unwrap();

    let response = state.into_frame().unwrap();
    let verify = "HTTP/1.1 200 OK\r\n\
                  Content-Length: 22\r\n\r\n\
                  hello extra data added";
    assert_eq!(response.into_data(), verify);
}

#[test]
fn test_chunked_partial() {
    let mut buf: BytesMut = "HTTP/1.1 200 OK\r\n\
                 Transfer-Encoding: chunked\r\n\r\n\
                 6\r\n\
                 hello \r\n\
                 5"
    .into();
    let verify = buf.clone();

    let mut cbuf = Cursor::new(&mut buf);
    let mut state = poll_first::<Response>(&mut cbuf);
    if let Err(e) = state.next(Event::End(&mut cbuf)) {
        matches!(e, HttpReadError::ChunkReaderNotEnoughData(_, _));
        assert_eq!(verify, BytesMut::from(e));
    } else {
        panic!()
    }
}
