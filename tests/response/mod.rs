#![allow(warnings)]
use buffer_plz::{Cursor, Event};
use bytes::BytesMut;
use header_plz::info_line::{request::Request, response::Response};
use oneone_plz::{oneone::OneOne, state::State};
use protocol_traits_plz::Frame;
use protocol_traits_plz::Step;
mod chunked;
mod content_length;
mod headers;
mod transfer_encoding;

pub fn parse_full_response(input: &[u8]) -> OneOne<Response> {
    let mut buf = BytesMut::from(input);
    let mut cbuf = Cursor::new(&mut buf);
    let mut state: State<Response> = State::new();
    let event = Event::Read(&mut cbuf);
    state = state.next(event).unwrap();
    assert!(matches!(state, State::End(_)));
    state.into_frame().unwrap()
}

// #[test]
// Fix
fn test_response_te_unknown() {
    let input = "HTTP/1.1 200 OK\r\nTransfer-Encoding: rot13\r\n\r\nZLRAPBQRQFGEVAT";
    let response = parse_full_response(input.as_bytes());
}

// Fix: Body not parsed

// #[test]
// Fix: Body and Extra body not parsed
fn test_response_missing_cl_with_extra_body() {
    let input = "HTTP/1.1 200 OK\r\n\r\n\
                 HELLO WORLD\n\
                 MORE DATA";

    let response = parse_full_response(input.as_bytes());
    dbg!(response);
}
