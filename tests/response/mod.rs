#![allow(warnings)]
use buffer_plz::{Cursor, Event};
use bytes::BytesMut;
use header_plz::Request;
use header_plz::Response;
use oneone_plz::{oneone::OneOne, state::State};
use protocol_traits_plz::Frame;
use protocol_traits_plz::Step;

use crate::parse_full_single;
use crate::poll_first;
mod chunked;
mod content_length;
mod headers;
mod transfer_encoding;

// #[test]
// FIX
fn test_response_te_unknown() {
    let input = "HTTP/1.1 200 OK\r\nTransfer-Encoding: rot13\r\n\r\nZLRAPBQRQFGEVAT";
    let response = parse_full_single::<Response>(input.as_bytes());
}

//#[test]
// FIX: corrupt deflate stream
fn test_response_content_length_deflate() {
    let input = b"HTTP/1.1 200 OK\r\n\
                Content-Length: 29\r\n\
                Content-Encoding: deflate\r\n\r\n\
                \x78\x9c\x05\x80\x41\x09\x00\x00\x08\xc4\xaa\x18\x4e\xc1\xc7\xe0\xc0\x8f\xf5\xc7\x0e\xa4\x3e\x47\x0b\x1a\x0b\x04\x5d";
    let response = parse_full_single::<Response>(input);
    let verify = "HTTP/1.1 200 OK\r\n\
                  Content-Length: 11\r\n\r\n\
                  hello world";
    assert_eq!(response.into_bytes(), verify);
}
