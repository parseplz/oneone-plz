use super::*;

#[test]
fn test_response_state_close_body() {
    let input = "HTTP/1.1 200 OK\r\n\r\n\
                 HELLO WORLD";

    let mut result =
        poll_state_result_with_end::<Response>(input.as_bytes().as_ref())
            .unwrap()
            .try_into_frame()
            .unwrap();

    let mut buf = BytesMut::new();
    result.decode(&mut buf).unwrap();

    let verify = "HTTP/1.1 200 OK\r\n\
                  Content-Length: 11\r\n\r\n\
                  HELLO WORLD";

    assert_eq!(result.into_bytes(), verify);
}

#[test]
fn test_response_state_close_body_multiple() {
    let chunks: &[&[u8]] =
        &[b"HTTP/1.1 200 OK\r\n\r\n", b"hello world ", b"more data"];
    let verify = "HTTP/1.1 200 OK\r\n\
                  Content-Length: 21\r\n\r\n\
                  hello world more data";
    let mut result = poll_oneone_multiple::<Response>(chunks);
    let mut buf = BytesMut::new();
    result.decode(&mut buf).unwrap();
    assert_eq!(result.into_bytes(), verify);
}
