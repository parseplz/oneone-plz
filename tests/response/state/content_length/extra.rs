use super::*;

#[test]
fn test_response_state_content_length_extra() {
    let input = "HTTP/1.1 200 OK\r\n\
                 Content-Length: 5\r\n\r\n\
                 hello world more data";
    let mut buf = BytesMut::from(input.as_bytes());
    let mut cbuf = Cursor::new(&mut buf);
    let mut state = poll_first::<Response>(&mut cbuf);
    state = state.try_next(Event::End(&mut cbuf)).unwrap();
    let response = state.try_into_frame().unwrap();
    let verify = "HTTP/1.1 200 OK\r\n\
                  Content-Length: 21\r\n\r\n\
                  hello world more data";
    assert_eq!(response.into_bytes(), verify);
}

#[test]
fn test_response_state_content_length_extra_multiple() {
    let chunks: &[&[u8]] = &[
        b"HTTP/1.1 200 OK\r\nContent-Length: 10\r\n\r\nh",
        b"ello world more data",
    ];
    let response = parse_full_multiple::<Response>(chunks);
    let verify = "HTTP/1.1 200 OK\r\n\
                  Content-Length: 21\r\n\r\n\
                  hello world more data";
    assert_eq!(response.into_bytes(), verify);
}

#[test]
fn test_response_state_content_length_extra_finished_end_single() {
    let chunks: &[&[u8]] = &[
        b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\n\r\nhello",
        b" world more data added",
    ];
    let response = parse_full_multiple::<Response>(chunks);
    let verify = "HTTP/1.1 200 OK\r\n\
                  Content-Length: 27\r\n\r\n\
                  hello world more data added";
    assert_eq!(response.into_bytes(), verify);
}

#[test]
fn test_response_state_content_length_extra_finished_read_end_multiple() {
    let chunks: &[&[u8]] = &[
        b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\n\r\nhello",
        b" world more data ",
        b"added",
    ];
    let response = parse_full_multiple::<Response>(chunks);
    let verify = "HTTP/1.1 200 OK\r\n\
                  Content-Length: 27\r\n\r\n\
                  hello world more data added";
    assert_eq!(response.into_bytes(), verify);
}
