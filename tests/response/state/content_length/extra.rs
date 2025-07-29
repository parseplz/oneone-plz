use super::*;

#[test]
fn test_response_state_content_length_extra() {
    let input = "HTTP/1.1 200 OK\r\n\
                 Content-Length: 5\r\n\r\n\
                 hello world more data";
    let verify = "HTTP/1.1 200 OK\r\n\
                  Content-Length: 21\r\n\r\n\
                  hello world more data";

    let mut response: OneOne<Response> =
        poll_state_result_with_end(input.as_bytes())
            .unwrap()
            .try_into_frame()
            .unwrap();

    let mut buf = BytesMut::new();
    response.decode(&mut buf).unwrap();

    assert_eq!(response.into_bytes(), verify);
}

#[test]
fn test_response_state_content_length_extra_multiple() {
    let chunks: &[&[u8]] = &[
        b"HTTP/1.1 200 OK\r\nContent-Length: 10\r\n\r\nh",
        b"ello world more data",
    ];
    let verify = "HTTP/1.1 200 OK\r\n\
                  Content-Length: 21\r\n\r\n\
                  hello world more data";
    let mut response = poll_oneone_multiple::<Response>(chunks);
    let mut buf = BytesMut::new();
    response.decode(&mut buf).unwrap();
    assert_eq!(response.into_bytes(), verify);
}

#[test]
fn test_response_state_content_length_extra_finished_end_single() {
    let chunks: &[&[u8]] = &[
        b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\n\r\nhello",
        b" world more data added",
    ];
    let verify = "HTTP/1.1 200 OK\r\n\
                  Content-Length: 27\r\n\r\n\
                  hello world more data added";

    let mut response = poll_oneone_multiple::<Response>(chunks);
    let mut buf = BytesMut::new();
    response.decode(&mut buf).unwrap();
    assert_eq!(response.into_bytes(), verify);
}

#[test]
fn test_response_state_content_length_extra_finished_read_end_multiple() {
    let chunks: &[&[u8]] = &[
        b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\n\r\nhello",
        b" world more data ",
        b"added",
    ];
    let verify = "HTTP/1.1 200 OK\r\n\
                  Content-Length: 27\r\n\r\n\
                  hello world more data added";
    let mut response = poll_oneone_multiple::<Response>(chunks);
    let mut buf = BytesMut::new();
    response.decode(&mut buf).unwrap();
    assert_eq!(response.into_bytes(), verify);
}
