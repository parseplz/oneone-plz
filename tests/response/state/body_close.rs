use super::*;

#[test]
fn test_response_state_body_close() {
    let input = "HTTP/1.1 200 OK\r\n\
                 Host: reqbin.com\r\n\
                 Content-Type: text/plain\r\n\r\n\
                 HolaAmigo";
    let mut buf = BytesMut::from(input);
    let mut cbuf = Cursor::new(&mut buf);

    let mut state = poll_first::<Response>(&mut cbuf);
    let event = Event::End(&mut cbuf);
    state = state.try_next(event).unwrap();
    assert!(matches!(state, State::End(_)));
    let one = state.try_into_frame().unwrap();
    assert_eq!(one.status_code(), "200");
    let result = one.into_bytes();
    let verify = "HTTP/1.1 200 OK\r\n\
                  Host: reqbin.com\r\n\
                  Content-Type: text/plain\r\n\
                  Content-Length: 9\r\n\r\n\
                  HolaAmigo";

    assert_eq!(result, verify);
}
