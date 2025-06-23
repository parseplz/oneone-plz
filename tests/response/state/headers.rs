use oneone_plz::error::HttpReadError;

use super::*;

#[test]
fn test_response_state_message_head_single() {
    let input = "HTTP/1.1 200 OK\r\n\
                 Value: 10000\r\n\r\n";

    let response = parse_full_single::<Response>(input.as_bytes());
    assert_eq!(response.status_code(), "200");
    let result = response.into_bytes();
    assert_eq!(result, input);
}

#[test]
fn test_response_state_message_head_multiple() {
    let input = "HTTP/1.1 200 OK\r\n\
                 Date: Mon, 18 Jul 2016 16:06:00 GMT\r\n";
    let mut buf = BytesMut::from(input);
    let mut cbuf = Cursor::new(&mut buf);
    let mut state: State<Response> = State::new();
    let event = Event::Read(&mut cbuf);
    state = state.try_next(event).unwrap();
    assert!(matches!(state, State::ReadMessageHead));
    cbuf.as_mut().extend_from_slice(b"Server: Apache\r\n");
    let event = Event::Read(&mut cbuf);
    state = state.try_next(event).unwrap();
    assert!(matches!(state, State::ReadMessageHead));

    cbuf.as_mut()
        .extend_from_slice(b"x-frame-options: DENY\r\n\r\n");
    let event = Event::Read(&mut cbuf);
    state = state.try_next(event).unwrap();
    assert!(matches!(state, State::End(_)));

    let result = state.try_into_frame().unwrap();
    assert_eq!(result.status_code(), "200");
    let verify = "HTTP/1.1 200 OK\r\n\
                  Date: Mon, 18 Jul 2016 16:06:00 GMT\r\n\
                  Server: Apache\r\n\
                  x-frame-options: DENY\r\n\r\n";
    assert_eq!(result.into_bytes(), verify);
}

#[test]
fn test_response_state_message_head_multiple_two() {
    let chunks: &[&[u8]] = &[
        b"HTTP/1.1 200 OK\r\nDate: Mon, 18 Jul 2016 16:06:00 GMT\r\n",
        b"Server: Apache\r\n",
        b"x-frame-options: DENY\r\n\r\n",
    ];
    let result = parse_full_multiple::<Response>(chunks);
    assert_eq!(result.status_code(), "200");
    let verify = "\
        HTTP/1.1 200 OK\r\n\
        Date: Mon, 18 Jul 2016 16:06:00 GMT\r\n\
        Server: Apache\r\n\
        x-frame-options: DENY\r\n\r\n";
    assert_eq!(result.into_bytes(), verify);
}

#[test]
fn test_response_state_switching_protocol() {
    let input = "HTTP/1.1 101 Switching Protocols\r\n\
                Upgrade: websocket\r\n\
                Connection: Upgrade\r\n\r\n";
    let response = parse_full_single::<Response>(input.as_bytes());
    assert_eq!(response.status_code(), "101");
}

#[test]
fn test_response_state_not_modified() {
    let input = "HTTP/1.1 304 OK\r\n\
                 X-Test: test\r\n\r\n";
    let response = parse_full_single::<Response>(input.as_bytes());
    assert_eq!(response.status_code(), "304");
}

#[test]
fn test_response_state_message_head_parital() {
    let input = "HTTP/1.1 304 OK\r\n";
    let mut buf = BytesMut::from(input);
    let mut cbuf = Cursor::new(&mut buf);
    let event = Event::End(&mut cbuf);
    let state: State<Response> = State::new();
    if let Err(HttpReadError::Unparsed(buf)) = state.try_next(event) {
        assert_eq!(buf, input);
    } else {
        panic!()
    }
}

/*
#[test]
fn test_response_no_content() {
    let input = "HTTP/1.1 204 OK\r\nX-Test: test\r\n\r\n";
    let response = parse_full_single::<Response>(input.as_bytes());
    todo!()
}
*/
