#![allow(warnings)]
use buffer_plz::{Cursor, Event};
use bytes::BytesMut;
use header_plz::info_line::{request::Request, response::Response};
use oneone_plz::{oneone::OneOne, state::State};
use protocol_traits_plz::Step;
//use oneone_plz::oneone::update::UpdateHttp;
use protocol_traits_plz::Frame;

// tests from https://github.com/parseplz/caido/blob/main/networking/proxy-client-tests/src/mock_target.rs#L93

/*
   b"/found" => send!(stream, b"HTTP/1.1 200 OK\r\n\r\n"),

   b"/cl-basic" => send!(stream, b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\n\r\nhello"),

   b"/cl-zero" => send!(stream, b"HTTP/1.1 307 OK\r\nLocation: /index.html\r\nContent-Length: 0\r\n\r\n"),

   b"/cl-brotli" => {
   send!(stream, b"HTTP/1.1 200 OK\r\nContent-Length: 15\r\nContent-Encoding: br\r\n\r\n");
   send!(stream, &[0x0b, 0x05, 0x80, 0x68, 0x65, 0x6c, 0x6c, 0x6f, 0x20, 0x77, 0x6f, 0x72, 0x6c, 0x64, 0x03,]);
   }

   b"/cl-gzip" => {
   send!(stream, b"HTTP/1.1 200 OK\r\nContent-Length: 41\r\nContent-Encoding: gzip\r\n\r\n");
   send!(stream, &[
   0x1f, 0x8b, 0x08, 0x00, 0x7e, 0x6c, 0xea, 0x65, 0x00, 0xff, 0x05, 0x80, 0x41, 0x09, 0x00, 0x00,
   0x08, 0xc4, 0xaa, 0x18, 0x4e, 0xc1, 0xc7, 0xe0, 0xc0, 0x8f, 0xf5, 0xc7, 0x0e, 0xa4, 0x3e, 0x47,
   0x0b, 0x85, 0x11, 0x4a, 0x0d, 0x0b, 0x00, 0x00, 0x00,
   ]);
   }

   b"/cl-deflate" => {
   send!(stream, b"HTTP/1.1 200 OK\r\nContent-Length: 29\r\nContent-Encoding: deflate\r\n\r\n");
   send!(stream, &[
   0x78, 0x9c, 0x05, 0x80, 0x41, 0x09, 0x00, 0x00, 0x08, 0xc4, 0xaa, 0x18, 0x4e, 0xc1, 0xc7, 0xe0,
   0xc0, 0x8f, 0xf5, 0xc7, 0x0e, 0xa4, 0x3e, 0x47, 0x0b, 0x1a, 0x0b, 0x04, 0x5d,
   ]);
   }

   b"/cl-zstd" => {
   send!(stream, b"HTTP/1.1 200 OK\r\nContent-Length: 24\r\nContent-Encoding: zstd\r\n\r\n");
   send!(stream, &[
   0x28, 0xb5, 0x2f, 0xfd, 0x24, 0x0b, 0x59, 0x00, 0x00, 0x68, 0x65, 0x6c, 0x6c, 0x6f, 0x20, 0x77,
   0x6f, 0x72, 0x6c, 0x64, 0x68, 0x69, 0x1e, 0xb2,
   ]);
   }

   b"/cl-large-response" => {
   send!(stream, b"HTTP/1.1 200 OK\r\nContent-Length: 1100\r\n\r\n");
   send!(stream, &b"hello world".repeat(100));
   }

   b"/head" => send!(stream, b"HTTP/1.1 200 OK\r\nContent-Length: 10000\r\n\r\n"),

   b"/partial-headers" => {
   send!(stream,b"HTTP/1.1 200 OK\r\nDate: Mon, 18 Jul 2016 16:06:00 GMT\r\n");
   sleep!();
   send!(stream, b"Server: Apache\r\n");
   sleep!();
   send!(stream, b"x-frame-options: DENY\r\n\r\n");
   }

   b"/te-chunked-one" => send!(stream, b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n5\r\nhello\r\n5\r\nworld\r\n0\r\n\r\n"),

   b"/te-chunked-multiple" =>  {
       send!(stream, b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n5\r\nhello\r\n");
       sleep!();
       send!(stream, b"5\r\nworld\r\n");
       sleep!();
       send!(stream, b"0\r\n\r\n");
   }

   ####################
   b"/te-gzip" =>  {
       send!(stream, b"HTTP/1.1 200 OK\r\nTransfer-Encoding: gzip\r\n\r\n");
       sleep!();
       send!(stream, &[0x1f,0x8b,0x08,0x00,0x1f,0x30,0xa0,0x65,0x00,0xff,0x05,0x40,0xc1,0x09,0x00,0x40,0x08,0x5a,0xc5,0xe1,0xce,0x28,0xb0,0x82]);
       sleep!();
       send!(stream, &[0xfb,0xb5,0xbd,0x24,0xa5,0x45,0x1f,0xe2,0x17,0xe7,0x19,0xd3,0x90,0xd8,0x52,0x0f,0x00,0x00,0x00]);
   }

   b"/te-chunked-large" => {
   send!(stream, b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n");
   send!(stream, &b"B\r\nhello world\r\n".repeat(100));
   send!(stream, b"0\r\n\r\n");
   }

b"/te-unknown" => {
    send!(stream, b"HTTP/1.1 200 OK\r\nTransfer-Encoding: rot13\r\n\r\nZLRAPBQRQFGEVAT");
    bail!("Force close connection")
}
b"/missing-cl-with-body" => {
    send!(stream, b"HTTP/1.1 200 OK\r\n\r\nHELLO WORLD");
    bail!("Force close connection")
},
    b"/missing-cl-extra" => {
        send!(stream, b"HTTP/1.1 200 OK\r\n\r\nHELLO WORLD");
        sleep!();
        send!(stream, b"\n\nmore data");
        bail!("Force close connection")
    },
    b"/force-close" => {
        send!(stream, b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\n\r\nhello");
        bail!("Force close connection")
    },
    b"/no-content" => {
        send!(stream, b"HTTP/1.1 204 OK\r\nX-Test: test\r\n\r\n");
    }
b"/no-content-with-content" => {
    send!(stream, b"HTTP/1.1 204 OK\r\nX-Test: test\r\n\r\n");
    sleep!();
    send!(stream, b"some data");
}
b"/switching-protocol" => {
    send!(stream, b"HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\n\r\n");
}
b"/not-modified" => {
    send!(stream, b"HTTP/1.1 304 OK\r\nX-Test: test\r\n\r\n");
}


#################

b"/cl-extra" => {
    send!(stream, b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\n\r\nhello");
    sleep!();
    send!(stream, b"\n\nmore data");
}


b"/cl-brotli-extra" => {
    send!(stream, b"HTTP/1.1 200 OK\r\nContent-Length: 15\r\nContent-Encoding: br\r\n\r\n");
    send!(stream, &[0x0b, 0x05, 0x80, 0x68, 0x65, 0x6c, 0x6c, 0x6f, 0x20, 0x77, 0x6f, 0x72, 0x6c, 0x64, 0x03,]);
    sleep!();
    send!(stream, b"\n\nmore data");
}


   b"/te-chunked-extra" =>  {
   send!(stream, b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n5\r\nhello\r\n5\r\nworld\r\n0\r\n\r\n");
   sleep!();
   send!(stream, b"\n\nmore data");
   }
*/

fn parse_full_response(input: &[u8]) -> OneOne<Response> {
    let mut buf = BytesMut::from(input);
    let mut cbuf = Cursor::new(&mut buf);
    let mut state: State<Response> = State::new();
    let event = Event::Read(&mut cbuf);
    state = state.next(event).unwrap();
    assert!(matches!(state, State::End(_)));
    state.into_frame().unwrap()
}

#[test]
fn test_response_content_length_basic() {
    let input = "HTTP/1.1 200 OK\r\n\
                 Content-Length: 5\r\n\r\n\
                 hello";
    let response = parse_full_response(input.as_bytes());
    assert_eq!(response.status_code(), "200");
    let result = response.into_data();
    assert_eq!(result, input);
}

#[test]
fn test_response_content_length_zero() {
    let input = "HTTP/1.1 307 OK\r\n\
                 Location: /index.html\r\n\
                 Content-Length: 0\r\n\r\n";
    let response = parse_full_response(input.as_bytes());
    assert_eq!(response.status_code(), "307");
    let result = response.into_data();
    assert_eq!(result, input);
}

#[test]
fn test_response_content_length_brotli() {
    let input = b"HTTP/1.1 200 OK\r\n\
                Content-Length: 15\r\n\
                Content-Encoding: br\r\n\r\n\
                \x0b\x05\x80\x68\x65\x6c\x6c\x6f\x20\x77\x6f\x72\x6c\x64\x03";
    let response = parse_full_response(input);
    assert_eq!(response.status_code(), "200");
    let verify = "HTTP/1.1 200 OK\r\n\
                  Content-Length: 11\r\n\r\n\
                  hello world";
    assert_eq!(response.into_data(), verify);
}

#[test]
fn test_response_content_length_gzip() {
    let input = b"HTTP/1.1 200 OK\r\n\
                Content-Length: 41\r\n\
                Content-Encoding: gzip\r\n\r\n\
                \x1f\x8b\x08\x00\x7e\x6c\xea\x65\x00\xff\x05\x80\x41\x09\x00\x00\x08\xc4\xaa\x18\x4e\xc1\xc7\xe0\xc0\x8f\xf5\xc7\x0e\xa4\x3e\x47\x0b\x85\x11\x4a\x0d\x0b\x00\x00\x00";
    let response = parse_full_response(input);
    let verify = "HTTP/1.1 200 OK\r\n\
                  Content-Length: 11\r\n\r\n\
                  hello world";
    assert_eq!(response.into_data(), verify);
}

// #[test]
// FIX: corrupt deflate stream
fn test_response_content_length_deflate() {
    let input = b"HTTP/1.1 200 OK\r\n\
                Content-Length: 29\r\n\
                Content-Encoding: deflate\r\n\r\n\
                \x78\x9c\x05\x80\x41\x09\x00\x00\x08\xc4\xaa\x18\x4e\xc1\xc7\xe0\xc0\x8f\xf5\xc7\x0e\xa4\x3e\x47\x0b\x1a\x0b\x04\x5d";
    let response = parse_full_response(input);
    let verify = "HTTP/1.1 200 OK\r\n\
                  Content-Length: 11\r\n\r\n\
                  hello world";
    assert_eq!(response.into_data(), verify);
}

#[test]
fn test_response_content_length_zstd() {
    let input = b"HTTP/1.1 200 OK\r\n\
                Content-Length: 24\r\n\
                Content-Encoding: zstd\r\n\r\n\
                \x28\xb5\x2f\xfd\x24\x0b\x59\x00\x00\x68\x65\x6c\x6c\x6f\x20\x77\x6f\x72\x6c\x64\x68\x69\x1e\xb2";
    let response = parse_full_response(input);
    let verify = "HTTP/1.1 200 OK\r\n\
                  Content-Length: 11\r\n\r\n\
                  hello world";
    assert_eq!(response.into_data(), verify);
}

#[test]
fn test_response_cl_large() {
    let mut input = "HTTP/1.1 200 OK\r\n\
                     Content-Length: 1100\r\n\r\n"
        .to_string();
    input.push_str(&"hello world".repeat(100));
    let _ = parse_full_response(input.as_bytes());
}

#[test]
fn test_response_head() {
    let input = "HTTP/1.1 200 OK\r\n\
                 Content-Length: 10000\r\n\r\n";
    let mut buf = BytesMut::from(input);
    let mut cbuf = Cursor::new(&mut buf);
    let mut state: State<Response> = State::new();
    let event = Event::Read(&mut cbuf);
    state = state.next(event).unwrap();
    assert!(matches!(state, State::ReadBodyContentLength(_, 10000)));
    let event = Event::End(&mut cbuf);
    state = state.next(event).unwrap();
    assert!(matches!(state, State::End(_)));
    let response = state.into_frame().unwrap();
    assert_eq!(response.status_code(), "200");
    let result = response.into_data();
    assert_eq!(result, input);
}

#[test]
fn test_response_partial_headers() {
    let input = "HTTP/1.1 200 OK\r\nDate: Mon, 18 Jul 2016 16:06:00 GMT\r\n";
    let mut buf = BytesMut::from(input);
    let mut cbuf = Cursor::new(&mut buf);
    let mut state: State<Response> = State::new();
    let event = Event::Read(&mut cbuf);
    state = state.next(event).unwrap();
    assert!(matches!(state, State::ReadHeader));
    cbuf.as_mut().extend_from_slice(b"Server: Apache\r\n");
    let event = Event::Read(&mut cbuf);
    state = state.next(event).unwrap();
    assert!(matches!(state, State::ReadHeader));

    cbuf.as_mut()
        .extend_from_slice(b"x-frame-options: DENY\r\n\r\n");
    let event = Event::Read(&mut cbuf);
    state = state.next(event).unwrap();
    assert!(matches!(state, State::End(_)));

    let response = state.into_frame().unwrap();
    assert_eq!(response.status_code(), "200");
    let verify = "HTTP/1.1 200 OK\r\n\
                  Date: Mon, 18 Jul 2016 16:06:00 GMT\r\n\
                  Server: Apache\r\n\
                  x-frame-options: DENY\r\n\r\n";
    assert_eq!(response.into_data(), verify);
}

#[test]
fn test_response_partial_headers_new() {
    let chunks: &[&[u8]] = &[
        b"HTTP/1.1 200 OK\r\nDate: Mon, 18 Jul 2016 16:06:00 GMT\r\n",
        b"Server: Apache\r\n",
        b"x-frame-options: DENY\r\n\r\n",
    ];
    let mut buf = BytesMut::from(chunks[0]);
    let mut cbuf = Cursor::new(&mut buf);
    let mut state: State<Response> = State::new();

    for &chunk in &chunks[1..] {
        cbuf.as_mut().extend_from_slice(chunk);
        state = state.next(Event::Read(&mut cbuf)).unwrap();
    }

    assert!(matches!(state, State::End(_)));

    let response = state.into_frame().unwrap();
    assert_eq!(response.status_code(), "200");

    let expected = "\
        HTTP/1.1 200 OK\r\n\
        Date: Mon, 18 Jul 2016 16:06:00 GMT\r\n\
        Server: Apache\r\n\
        x-frame-options: DENY\r\n\r\n";

    assert_eq!(response.into_data(), expected);
}

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
fn test_response_te_gzip() {}

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

// #[test]
// Fix
fn test_response_te_unknown() {
    let input = "HTTP/1.1 200 OK\r\nTransfer-Encoding: rot13\r\n\r\nZLRAPBQRQFGEVAT";
    let response = parse_full_response(input.as_bytes());
}
