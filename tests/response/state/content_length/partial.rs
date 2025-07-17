use super::*;

#[test]
fn test_response_state_content_length_partial_no_fix() {
    let input = "HTTP/1.1 200 OK\r\n\
                 Content-Length: 5\r\n\r\n\
                 h";

    let response = poll_state_result_with_end::<Response>(input.as_bytes());
    if let Err(e) = response {
        assert!(matches!(e, HttpStateError::ContentLengthPartial(_, _)));
        let verify = "HTTP/1.1 200 OK\r\n\
                      Content-Length: 5\r\n\r\n\
                      h";
        assert_eq!(verify, BytesMut::from(e));
    } else {
        panic!()
    }
}

#[test]
fn test_response_state_content_length_partial_fix() {
    let input = "HTTP/1.1 200 OK\r\n\
                 Content-Length: 5\r\n\r\n\
                 h";
    let result = poll_state_result_with_end::<Response>(input.as_bytes());
    if let Err(e) = result {
        assert!(matches!(e, HttpStateError::ContentLengthPartial(_, _)));
        let verify = "HTTP/1.1 200 OK\r\n\
                      Content-Length: 1\r\n\r\n\
                      h";
        assert_eq!(
            verify,
            OneOne::<Response>::try_from(e)
                .unwrap()
                .into_bytes()
        );
    } else {
        panic!()
    }
}

#[test]
fn test_response_state_content_length_partial_two() {
    let res = "HTTP/1.1 200 OK\r\n\
                   Host: reqbin.com\r\n\
                   Content-Type: text/plain\r\n\
                   Content-Length: 100\r\n\r\n\
                   h";

    let result = poll_state_result_with_end::<Response>(res.as_bytes());
    if let Err(HttpStateError::ContentLengthPartial(oneone, buf)) = result {
        let data = oneone.into_bytes();
        assert_eq!(data, &res[..res.len() - 1]);
        assert_eq!(buf, "h");
    } else {
        panic!()
    }
}

#[test]
fn test_response_state_content_length_no_body_no_fix() {
    let input = "HTTP/1.1 200 OK\r\n\
                 Content-Length: 5\r\n\r\n";
    let result = poll_state_result_with_end::<Response>(input.as_bytes());
    if let Err(e) = result {
        matches!(e, HttpStateError::ContentLengthPartial(_, _));
        let verify = "HTTP/1.1 200 OK\r\n\
                      Content-Length: 5\r\n\r\n";
        assert_eq!(verify, BytesMut::from(e));
    } else {
        panic!()
    }
}

#[test]
fn test_response_state_content_length_no_body_fix() {
    let input = "HTTP/1.1 200 OK\r\n\
                 Content-Length: 5\r\n\r\n";
    let verify = "HTTP/1.1 200 OK\r\n\
                  Content-Length: 0\r\n\r\n";
    let result = poll_state_result_with_end::<Response>(input.as_bytes());
    if let Err(e) = result {
        matches!(e, HttpStateError::ContentLengthPartial(_, _));
        assert_eq!(
            verify,
            OneOne::<Response>::try_from(e)
                .unwrap()
                .into_bytes()
        );
    } else {
        panic!()
    }
}
