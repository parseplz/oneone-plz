use super::*;

#[test]
fn test_request_state_partial_info_line_only() {
    let input = "GET /echo HTTP/1.1\r\n";
    let mut buf: BytesMut = input.into();
    let mut cbuf = Cursor::new(&mut buf);
    let state = poll_first::<Request>(&mut cbuf);
    assert!(matches!(state, State::ReadMessageHead));
    assert_eq!(cbuf.position(), 17);
    let event = Event::End(&mut cbuf);
    let result = state.try_next(event);
    assert!(matches!(result, Err(HttpReadError::Unparsed(_))));
}

#[test]
fn test_request_state_partial_header() {
    let input = "GET /partial HTTP/1.1\r\n\
                 Host: example.com\r\n";
    let mut buf = BytesMut::from(input);
    let mut cbuf = Cursor::new(&mut buf);
    let state = poll_first::<Request>(&mut cbuf);
    assert!(matches!(state, State::ReadMessageHead));
    assert_eq!(cbuf.position(), 39);
}
