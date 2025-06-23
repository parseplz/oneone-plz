use super::*;

#[test]
fn test_response_state_chunked_one() {
    let input = "HTTP/1.1 200 OK\r\n\
                 Transfer-Encoding: chunked\r\n\r\n\
                 6\r\n\
                 hello \r\n\
                 5\r\n\
                 world\r\n\
                 0\r\n\r\n";
    let verify = "HTTP/1.1 200 OK\r\n\
                  Content-Length: 11\r\n\r\n\
                  hello world";
    let result = parse_full_single::<Response>(input.as_bytes());
    assert_eq!(result.into_bytes(), verify);
}

#[test]
fn test_response_state_chunked_multiple() {
    let chunks: &[&[u8]] = &[
        b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n5\r\nhello\r\n",
        b"5\r\nworld\r\n",
        b"0\r\n\r\n",
    ];
    let result = parse_full_multiple::<Response>(chunks);
    let verify = "HTTP/1.1 200 OK\r\n\
                    Content-Length: 10\r\n\r\n\
                    helloworld";
    assert_eq!(result.into_bytes(), verify);
}

#[test]
fn test_response_state_chunked_multiple_large() {
    let chunks: &[&[u8]] = &[
        b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n",
        &b"B\r\nhello world\r\n".repeat(100),
        b"0\r\n\r\n",
    ];

    let result = parse_full_multiple::<Response>(chunks);
    let verify = "HTTP/1.1 200 OK\r\n\
                    Content-Length: 1100\r\n\r\n"
        .to_string()
        + &"hello world".repeat(100);
    assert_eq!(result.status_code(), "200");
    assert_eq!(result.into_bytes(), verify);
}

#[test]
fn test_response_state_chunked_extra_single() {
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
    let mut state = poll_first::<Response>(&mut cbuf);
    let event = Event::End(&mut cbuf);
    state = state.try_next(event).unwrap();
    assert!(matches!(state, State::ReadBodyChunkedExtraEnd(..)));
    let result = state.try_into_frame().unwrap();
    let verify = "HTTP/1.1 200 OK\r\n\
                  Content-Length: 22\r\n\r\n\
                  hello world extra data";
    assert_eq!(result.into_bytes(), verify);
}

#[test]
fn test_response_state_chunked_extra_multiple() {
    let chunks: &[&[u8]] = &[
        b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n",
        b"6\r\nhello \r\n",
        b"0\r\n\r\n",
        b"extra data ",
        b"added",
    ];

    let result = parse_full_multiple::<Response>(chunks);
    let verify = "HTTP/1.1 200 OK\r\n\
                  Content-Length: 22\r\n\r\n\
                  hello extra data added";
    assert_eq!(result.into_bytes(), verify);
}

#[test]
fn test_response_state_chunked_extra_finished_single() {
    let input = "HTTP/1.1 200 OK\r\n\
                 Transfer-Encoding: chunked\r\n\r\n\
                 6\r\n\
                 hello \r\n\
                 0\r\n\r\n";

    let mut buf = BytesMut::from(input);
    let mut cbuf = Cursor::new(&mut buf);
    let mut state: State<Response> = State::new();
    let event = Event::Read(&mut cbuf);
    state = state.try_next(event).unwrap();
    assert!(matches!(state, State::End(_)));

    cbuf.as_mut().extend_from_slice(b"extra data added");
    state = state.try_next(Event::End(&mut cbuf)).unwrap();
    let result = state.try_into_frame().unwrap();

    let verify = "HTTP/1.1 200 OK\r\n\
                  Content-Length: 22\r\n\r\n\
                  hello extra data added";
    assert_eq!(result.into_bytes(), verify);
}

#[test]
fn test_response_state_chunked_extra_finished_multiple() {
    let input = "HTTP/1.1 200 OK\r\n\
                 Transfer-Encoding: chunked\r\n\r\n\
                 6\r\n\
                 hello \r\n\
                 0\r\n\r\n";

    let mut buf = BytesMut::from(input);
    let mut cbuf = Cursor::new(&mut buf);
    let mut state: State<Response> = State::new();
    let event = Event::Read(&mut cbuf);
    state = state.try_next(event).unwrap();
    assert!(matches!(state, State::End(_)));

    // extra 1
    cbuf.as_mut().extend_from_slice(b"extra data");
    state = state.try_next(Event::Read(&mut cbuf)).unwrap();
    assert!(matches!(state, State::ReadBodyChunkedExtra(_)));

    // extra 2
    cbuf.as_mut().extend_from_slice(b" added");
    state = state.try_next(Event::End(&mut cbuf)).unwrap();

    let result = state.try_into_frame().unwrap();
    let verify = "HTTP/1.1 200 OK\r\n\
                  Content-Length: 22\r\n\r\n\
                  hello extra data added";
    assert_eq!(result.into_bytes(), verify);
}

#[test]
fn test_response_state_chunked_partial() {
    let input = "HTTP/1.1 200 OK\r\n\
                 Transfer-Encoding: chunked\r\n\r\n\
                 6\r\n\
                 hello \r\n\
                 5";

    let mut buf = BytesMut::from(input);
    let verify = buf.clone();

    let mut cbuf = Cursor::new(&mut buf);
    let state = poll_first::<Response>(&mut cbuf);
    if let Err(e) = state.try_next(Event::End(&mut cbuf)) {
        matches!(e, HttpReadError::ChunkReaderPartial(_, _));
        assert_eq!(verify, BytesMut::from(e));
    } else {
        panic!()
    }
}
