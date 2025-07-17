use super::*;

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

    let result = poll_state_result_with_end::<Response>(input.as_bytes())
        .unwrap()
        .try_into_frame()
        .unwrap()
        .into_bytes();
    let verify = "HTTP/1.1 200 OK\r\n\
                  Content-Length: 22\r\n\r\n\
                  hello world extra data";
    assert_eq!(result, verify);
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

    let result = poll_oneone_multiple::<Response>(chunks);
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

    let (mut buf, mut state) = poll_state_once::<Response>(input.as_bytes());
    assert!(matches!(state, State::End(_)));
    let mut cbuf = Cursor::new(&mut buf);

    cbuf.as_mut()
        .extend_from_slice(b"extra data added");
    let result = state
        .try_next(Event::End(&mut cbuf))
        .unwrap()
        .try_into_frame()
        .unwrap();
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

    let (mut buf, mut state) = poll_state_once::<Response>(input.as_bytes());
    let mut cbuf = Cursor::new(&mut buf);
    assert!(matches!(state, State::End(_)));

    // extra 1
    cbuf.as_mut()
        .extend_from_slice(b"extra data");
    state = state
        .try_next(Event::Read(&mut cbuf))
        .unwrap();
    assert!(matches!(state, State::ReadBodyChunkedExtra(_)));

    // extra 2
    cbuf.as_mut()
        .extend_from_slice(b" added");
    state = state
        .try_next(Event::End(&mut cbuf))
        .unwrap();

    let result = state.try_into_frame().unwrap();
    let verify = "HTTP/1.1 200 OK\r\n\
                  Content-Length: 22\r\n\r\n\
                  hello extra data added";
    assert_eq!(result.into_bytes(), verify);
}
