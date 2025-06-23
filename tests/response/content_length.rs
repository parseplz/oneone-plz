use super::*;

#[test]
fn test_response_content_length_basic() {
    let input = "HTTP/1.1 200 OK\r\n\
                 Content-Length: 5\r\n\r\n\
                 hello";
    let response = parse_full_single::<Response>(input.as_bytes());
    assert_eq!(response.status_code(), "200");
    let result = response.into_bytes();
    assert_eq!(result, input);
}

#[test]
fn test_response_content_length_zero() {
    let input = "HTTP/1.1 307 OK\r\n\
                 Location: /index.html\r\n\
                 Content-Length: 0\r\n\r\n";
    let response = parse_full_single::<Response>(input.as_bytes());
    assert_eq!(response.status_code(), "307");
    let result = response.into_bytes();
    assert_eq!(result, input);
}

#[test]
fn test_response_content_length_brotli() {
    let input = b"HTTP/1.1 200 OK\r\n\
                Content-Length: 15\r\n\
                Content-Encoding: br\r\n\r\n\
                \x0b\x05\x80\x68\x65\x6c\x6c\x6f\x20\x77\x6f\x72\x6c\x64\x03";
    let response = parse_full_single::<Response>(input);
    assert_eq!(response.status_code(), "200");
    let verify = "HTTP/1.1 200 OK\r\n\
                  Content-Length: 11\r\n\r\n\
                  hello world";
    assert_eq!(response.into_bytes(), verify);
}

#[test]
fn test_response_content_length_gzip() {
    let input = b"HTTP/1.1 200 OK\r\n\
                Content-Length: 41\r\n\
                Content-Encoding: gzip\r\n\r\n\
                \x1f\x8b\x08\x00\x7e\x6c\xea\x65\x00\xff\x05\x80\x41\x09\x00\x00\x08\xc4\xaa\x18\x4e\xc1\xc7\xe0\xc0\x8f\xf5\xc7\x0e\xa4\x3e\x47\x0b\x85\x11\x4a\x0d\x0b\x00\x00\x00";
    let response = parse_full_single::<Response>(input);
    let verify = "HTTP/1.1 200 OK\r\n\
                  Content-Length: 11\r\n\r\n\
                  hello world";
    assert_eq!(response.into_bytes(), verify);
}

#[test]
fn test_response_content_length_zstd() {
    let input = b"HTTP/1.1 200 OK\r\n\
                Content-Length: 24\r\n\
                Content-Encoding: zstd\r\n\r\n\
                \x28\xb5\x2f\xfd\x24\x0b\x59\x00\x00\x68\x65\x6c\x6c\x6f\x20\x77\x6f\x72\x6c\x64\x68\x69\x1e\xb2";
    let response = parse_full_single::<Response>(input);
    let verify = "HTTP/1.1 200 OK\r\n\
                  Content-Length: 11\r\n\r\n\
                  hello world";
    assert_eq!(response.into_bytes(), verify);
}

#[test]
fn test_response_cl_large() {
    let mut input = "HTTP/1.1 200 OK\r\n\
                     Content-Length: 1100\r\n\r\n"
        .to_string();
    input.push_str(&"hello world".repeat(100));
    let _ = parse_full_single::<Response>(input.as_bytes());
}

#[test]
fn test_response_cl_extra_single() {
    let input = "HTTP/1.1 200 OK\r\n\
                 Content-Length: 5\r\n\r\n\
                 hello world more data";
    let mut buf = BytesMut::from(input.as_bytes());
    let mut cbuf = Cursor::new(&mut buf);
    let mut state = poll_first::<Response>(&mut cbuf);
    state = state.try_next(Event::End(&mut cbuf)).unwrap();
    let response = state.try_into_frame().unwrap();
    let verify = "HTTP/1.1 200 OK\r\n\
                  Content-Length: 21\r\n\r\n\
                  hello world more data";
    assert_eq!(response.into_bytes(), verify);
}

#[test]
fn test_response_cl_extra_multiple() {
    let chunks: &[&[u8]] = &[
        b"HTTP/1.1 200 OK\r\nContent-Length: 10\r\n\r\nh",
        b"ello world more data",
    ];
    let response = parse_full_multiple::<Response>(chunks);
    let verify = "HTTP/1.1 200 OK\r\n\
                  Content-Length: 21\r\n\r\n\
                  hello world more data";
    assert_eq!(response.into_bytes(), verify);
}

#[test]
fn test_response_cl_extra_finished_end_single() {
    let chunks: &[&[u8]] = &[
        b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\n\r\nhello",
        b" world more data added",
    ];
    let response = parse_full_multiple::<Response>(chunks);
    let verify = "HTTP/1.1 200 OK\r\n\
                  Content-Length: 27\r\n\r\n\
                  hello world more data added";
    assert_eq!(response.into_bytes(), verify);
}

#[test]
fn test_response_cl_extra_finished_read_end_multiple() {
    let chunks: &[&[u8]] = &[
        b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\n\r\nhello",
        b" world more data ",
        b"added",
    ];
    let response = parse_full_multiple::<Response>(chunks);
    let verify = "HTTP/1.1 200 OK\r\n\
                  Content-Length: 27\r\n\r\n\
                  hello world more data added";
    assert_eq!(response.into_bytes(), verify);
}

#[test]
fn test_response_cl_partial_no_fix() {
    let input = "HTTP/1.1 200 OK\r\n\
                 Content-Length: 5\r\n\r\n\
                 h";
    let mut buf = BytesMut::from(input.as_bytes());
    let mut cbuf = Cursor::new(&mut buf);
    let state = poll_first::<Response>(&mut cbuf);
    let response = state.try_next(Event::End(&mut cbuf));
    if let Err(e) = response {
        assert!(matches!(e, HttpReadError::ContentLengthPartial(_, _)));
        let verify = "HTTP/1.1 200 OK\r\n\
                      Content-Length: 5\r\n\r\n\
                      h";
        assert_eq!(verify, BytesMut::from(e));
    } else {
        panic!()
    }
}

#[test]
fn test_response_cl_partial_fix() {
    let input = "HTTP/1.1 200 OK\r\n\
                 Content-Length: 5\r\n\r\n\
                 h";
    let mut buf = BytesMut::from(input.as_bytes());
    let mut cbuf = Cursor::new(&mut buf);
    let state = poll_first::<Response>(&mut cbuf);
    if let Err(e) = state.try_next(Event::End(&mut cbuf)) {
        assert!(matches!(e, HttpReadError::ContentLengthPartial(_, _)));
        let verify = "HTTP/1.1 200 OK\r\n\
                      Content-Length: 1\r\n\r\n\
                      h";
        assert_eq!(
            verify,
            OneOne::<Response>::try_from(e).unwrap().into_bytes()
        );
    } else {
        panic!()
    }
}

#[test]
fn test_response_cl_no_body_no_fix() {
    let input = "HTTP/1.1 200 OK\r\n\
                 Content-Length: 5\r\n\r\n";
    let mut buf = BytesMut::from(input.as_bytes());
    let mut cbuf = Cursor::new(&mut buf);
    let state = poll_first::<Response>(&mut cbuf);
    if let Err(e) = state.try_next(Event::End(&mut cbuf)) {
        matches!(e, HttpReadError::ContentLengthPartial(_, _));
        let verify = "HTTP/1.1 200 OK\r\n\
                      Content-Length: 5\r\n\r\n";
        assert_eq!(verify, BytesMut::from(e));
    } else {
        panic!()
    }
}

#[test]
fn test_response_cl_no_body_fix() {
    let input = "HTTP/1.1 200 OK\r\n\
                 Content-Length: 5\r\n\r\n";
    let verify = "HTTP/1.1 200 OK\r\n\
                  Content-Length: 0\r\n\r\n";
    let mut buf = BytesMut::from(input.as_bytes());
    let mut cbuf = Cursor::new(&mut buf);
    let state = poll_first::<Response>(&mut cbuf);
    if let Err(e) = state.try_next(Event::End(&mut cbuf)) {
        matches!(e, HttpReadError::ContentLengthPartial(_, _));
        assert_eq!(
            verify,
            OneOne::<Response>::try_from(e).unwrap().into_bytes()
        );
    } else {
        panic!()
    }
}

#[test]
fn test_response_missing_cl_with_body() {
    let input = "HTTP/1.1 200 OK\r\n\r\n\
                 HELLO WORLD";

    let mut buf = BytesMut::from(input);
    let mut cbuf = Cursor::new(&mut buf);
    let mut state = poll_first::<Response>(&mut cbuf);
    state = state.try_next(Event::End(&mut cbuf)).unwrap();
    let response = state.try_into_frame().unwrap();
    let verify = "HTTP/1.1 200 OK\r\n\
                  Content-Length: 11\r\n\r\n\
                  HELLO WORLD";

    assert_eq!(response.into_bytes(), verify);
}

#[test]
fn test_response_missing_cl_with_extra_body() {
    let input = "HTTP/1.1 200 OK\r\n\r\n\
                 HELLO WORLD\n\
                 MORE DATA";

    let mut buf = BytesMut::from(input);
    let mut cbuf = Cursor::new(&mut buf);
    let mut state = poll_first::<Response>(&mut cbuf);
    state = state.try_next(Event::End(&mut cbuf)).unwrap();
    assert!(matches!(state, State::End(_)));
    let verify = "HTTP/1.1 200 OK\r\n\
                  Content-Length: 21\r\n\r\n\
                  HELLO WORLD\nMORE DATA";
    assert_eq!(state.try_into_frame().unwrap().into_bytes(), verify);
}
