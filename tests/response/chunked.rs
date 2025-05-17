use crate::response::parse_full_response;
use buffer_plz::{Cursor, Event};
use bytes::BytesMut;
use header_plz::info_line::{request::Request, response::Response};
use oneone_plz::{oneone::OneOne, state::State};
use protocol_traits_plz::Frame;
use protocol_traits_plz::Step;

#[test]
fn test_response_chunked_one() {
    let input = "HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n5\r\nhello\r\n5\r\nworld\r\n0\r\n\r\n";
    let response = parse_full_response(input.as_bytes());
    let verify = "HTTP/1.1 200 OK\r\n\
                  Content-Length: 10\r\n\r\n\
                  helloworld";
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
    let mut state: State<Response> = State::new();

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
fn test_response_te_chunked_large() {
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
                    Content-Length: 1100\r\n\r\n\
                    "
    .to_string()
        + &"hello world".repeat(100);
    let response = state.into_frame().unwrap();
    assert_eq!(response.status_code(), "200");
    assert_eq!(response.into_data(), expected);
}
