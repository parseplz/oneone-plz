use super::*;

#[test]
fn test_request_state_chunked_all() {
    let input = "POST /echo HTTP/1.1\r\n\
                 Host: reqbin.com\r\n\
                 Trailer: Some\r\n\
                 Transfer-Encoding: chunked\r\n\r\n\
                 7\r\n\
                 Mozilla\r\n\
                 9\r\n\
                 Developer\r\n\
                 7\r\n\
                 Network\r\n\
                 0\r\n\
                 Header: Val\r\n\
                 \r\n";
    let verify = "POST /echo HTTP/1.1\r\n\
                  Host: reqbin.com\r\n\
                  Header: Val\r\n\
                  Content-Length: 23\r\n\r\n\
                  MozillaDeveloperNetwork";
    let result = poll_oneone_only_read::<Request>(input.as_bytes());
    assert_eq!(result.into_bytes(), verify);
}

#[test]
fn test_request_state_chunked_no_trailer() {
    let input = "POST /chunked HTTP/1.1\r\n\
                 Host: reqbin.com\r\n\
                 Transfer-Encoding: chunked\r\n\r\n\
                 7\r\n\
                 Mozilla\r\n\
                 0\r\n\r\n";
    let verify = "POST /chunked HTTP/1.1\r\n\
                  Host: reqbin.com\r\n\
                  Content-Length: 7\r\n\r\n\
                  Mozilla";
    let result = poll_oneone_only_read::<Request>(input.as_bytes());
    assert_eq!(result.into_bytes(), verify);
}

#[test]
fn test_request_state_chunked_incomplete() {
    let input = "POST /truncated HTTP/1.1\r\n\
                 Transfer-Encoding: chunked\r\n\r\n\
                 7\r\n\
                 Mozilla\r\n\
                 0\r\n";
    let mut buf = BytesMut::from(input);
    let mut cbuf = Cursor::new(&mut buf);
    let mut state: State<Request> = State::new();
    let event = Event::Read(&mut cbuf);
    state = state.try_next(event).unwrap();
    // Incomplete data, expect ReadBodyChunked state with remaining data.
    assert!(matches!(state, State::ReadBodyChunked(_, _)));
}
