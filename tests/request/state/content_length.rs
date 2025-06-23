use super::*;

#[test]
fn test_request_state_post_success() {
    let input = "POST /echo HTTP/1.1\r\n\
                 Host: reqbin.com\r\n\
                 content-length: 7\r\n\r\n\
                 Mozilla";
    let result = parse_full_single::<Request>(input.as_bytes());
    assert_eq!(result.method_as_string(), "POST");
    assert_eq!(result.uri_as_string(), "/echo");
}

#[test]
fn test_request_state_post_empty_body() {
    // Test when the request/response has an empty body.
    let input = "POST /empty HTTP/1.1\r\n\
                 Host: reqbin.com\r\n\
                 Content-Length: 0\r\n\r\n";
    let mut buf = BytesMut::from(input);
    let org_range = buf.as_ptr_range();
    let mut cbuf = Cursor::new(&mut buf);
    let state = poll_first::<Request>(&mut cbuf);
    if let State::End(_) = state {
        let result = state.try_into_frame().unwrap().into_bytes();
        let result_range = result.as_ptr_range();
        assert_eq!(org_range, result_range);
    } else {
        panic!("Expected State::End, found {:?}", state);
    }
}
