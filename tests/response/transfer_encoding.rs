use buffer_plz::{Cursor, Event};
use bytes::BytesMut;
use header_plz::info_line::{request::Request, response::Response};
use oneone_plz::{oneone::OneOne, state::State};
use protocol_traits_plz::Frame;
use protocol_traits_plz::Step;

#[test]
fn test_response_te_gzip() {
    let chunks: &[&[u8]] = &[b"HTTP/1.1 200 OK\r\nTransfer-Encoding: gzip\r\n\r\n",
    b"\x1f\x8b\x08\x00\x1f\x30\xa0\x65\x00\xff\x05\x40\xc1\x09\x00\x40\x08\x5a\xc5\xe1\xce\x28\xb0\x82",
    b"\xfb\xb5\xbd\x24\xa5\x45\x1f\xe2\x17\xe7\x19\xd3\x90\xd8\x52\x0f\x00\x00\x00"];

    let mut buf = BytesMut::from(chunks[0]);
    let mut cbuf = Cursor::new(&mut buf);
    let mut state: State<Response> = State::new();

    for &chunk in &chunks[1..] {
        state = state.next(Event::Read(&mut cbuf)).unwrap();
        cbuf.as_mut().extend_from_slice(chunk);
    }

    state = state.next(Event::End(&mut cbuf)).unwrap();
    assert!(matches!(state, State::End(_)));
    let response = state.into_frame().unwrap();

    let expected = "HTTP/1.1 200 OK\r\n\
                    Content-Length: 15\r\n\r\n\
                    hello my friend";
    assert_eq!(response.into_data(), expected);
}
