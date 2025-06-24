use super::*;
mod complete;
mod compressed;
mod extra;
mod partial;

#[test]
fn test_response_state_content_length_no_body_no_fix() {
    let input = "HTTP/1.1 200 OK\r\n\
                 Content-Length: 5\r\n\r\n";
    let result = poll_state::<Response>(input.as_bytes());
    if let Err(e) = result {
        matches!(e, HttpReadError::ContentLengthPartial(_, _));
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
fn test_response_state_content_length_missing_cl_header_with_body() {
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
fn test_response_state_content_length_missing_cl_header_with_extra_body() {
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
