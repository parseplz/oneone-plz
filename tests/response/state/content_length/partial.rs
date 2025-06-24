use super::*;

#[test]
fn test_response_state_content_length_partial_no_fix() {
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
fn test_response_state_content_length_partial_fix() {
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
fn test_response_state_content_length_partial_two() {
    let res = "HTTP/1.1 200 OK\r\n\
                   Host: reqbin.com\r\n\
                   Content-Type: text/plain\r\n\
                   Content-Length: 100\r\n\r\n\
                   h";

    let result = poll_state::<Response>(res.as_bytes());
    if let Err(HttpReadError::ContentLengthPartial(oneone, buf)) = result {
        let data = oneone.into_bytes();
        assert_eq!(data, &res[..res.len() - 1]);
        assert_eq!(buf, "h");
    } else {
        panic!()
    }
}
