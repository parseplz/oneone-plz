#![allow(warnings)]
use buffer_plz::{Cursor, Event};
use bytes::BytesMut;
use header_plz::info_line::{request::Request, response::Response};
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
// Fix
fn test_response_te_unknown() {
    let input = "HTTP/1.1 200 OK\r\nTransfer-Encoding: rot13\r\n\r\nZLRAPBQRQFGEVAT";
    let response = parse_full_single::<Response>(input.as_bytes());
}
