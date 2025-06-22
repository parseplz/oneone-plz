use super::*;

#[test]
fn test_request_partial_header() {
    let req = "GET /partial HTTP/1.1\r\n\
                   Host: example.com\r\n";
    let mut buf = BytesMut::from(req);
    let mut cbuf = Cursor::new(&mut buf);
    let state = poll_first::<Request>(&mut cbuf);
    assert!(matches!(state, State::ReadMessageHead));
    assert_eq!(cbuf.position(), 39);
}

#[test]
fn test_request_state_get_partial() {
    let req = "GET /echo HTTP/1.1\r\n";
    let mut buf: BytesMut = req.into();
    let mut cbuf = Cursor::new(&mut buf);
    let state = poll_first::<Request>(&mut cbuf);
    assert!(matches!(state, State::ReadMessageHead));
    assert_eq!(cbuf.position(), 17);
    let event = Event::End(&mut cbuf);
    let result = state.try_next(event);
    assert!(matches!(result, Err(HttpReadError::Unparsed(_))));
}
