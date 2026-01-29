use super::*;

#[test]
fn test_request_state_post_success() {
    let input = "POST /echo HTTP/1.1\r\n\
                 Host: reqbin.com\r\n\
                 content-length: 7\r\n\r\n\
                 Mozilla";
    let result = poll_oneone_only_read::<OneRequestLine>(input.as_bytes());
    assert_eq!(result.method_as_string(), "POST");
    assert_eq!(result.uri_as_string(), "/echo");
}

#[test]
fn test_request_state_post_empty_body() {
    // Test when the request/response has an empty body.
    let input = "POST /empty HTTP/1.1\r\n\
                 Host: reqbin.com\r\n\
                 Content-Length: 0\r\n\r\n";
    let (mut buf, state) = poll_state_once::<OneRequestLine>(input.as_bytes());

    if let State::End(_) = state {
        let result = state
            .try_into_frame()
            .unwrap()
            .into_bytes();
        assert_eq!(result, input.as_bytes());
    } else {
        panic!("Expected State::End, found {:?}", state);
    }
}
