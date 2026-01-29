use super::*;

#[test]
fn test_response_state_content_length_partial_no_fix() {
    let input = "HTTP/1.1 200 OK\r\n\
                 Content-Length: 5\r\n\r\n\
                 h";

    let err = poll_state_result_with_end::<OneResponseLine>(input.as_bytes());

    if let Err(e) = err {
        assert!(e.is_partial());
        assert!(matches!(e, HttpStateError::ContentLengthPartial(_)));
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
    let result =
        poll_state_result_with_end::<OneResponseLine>(input.as_bytes());
    if let Err(e) = result {
        assert!(e.is_partial());
        assert!(matches!(e, HttpStateError::ContentLengthPartial(_)));
        let verify = "HTTP/1.1 200 OK\r\n\
                      Content-Length: 1\r\n\r\n\
                      h";
        assert_eq!(
            verify,
            OneOne::<OneResponseLine>::try_from(e)
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

    let result = poll_state_result_with_end::<OneResponseLine>(res.as_bytes());
    if let Err(HttpStateError::ContentLengthPartial(boxed)) = result {
        let (one, buf) = *boxed;
        let data = one.into_bytes();
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
    let result =
        poll_state_result_with_end::<OneResponseLine>(input.as_bytes());
    if let Err(e) = result {
        assert!(e.is_partial());
        matches!(e, HttpStateError::ContentLengthPartial(_));
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
    let result =
        poll_state_result_with_end::<OneResponseLine>(input.as_bytes());
    if let Err(e) = result {
        assert!(e.is_partial());
        matches!(e, HttpStateError::ContentLengthPartial(_));
        assert_eq!(
            verify,
            OneOne::<OneResponseLine>::try_from(e)
                .unwrap()
                .into_bytes()
        );
    } else {
        panic!()
    }
}
