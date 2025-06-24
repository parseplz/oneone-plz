use super::*;

#[test]
fn test_response_state_close_body() {
    let input = "HTTP/1.1 200 OK\r\n\r\n\
                 HELLO WORLD";

    let result = poll_state_result_with_end::<Response>(input.as_bytes().as_ref())
        .unwrap()
        .try_into_frame()
        .unwrap()
        .into_bytes();
    let verify = "HTTP/1.1 200 OK\r\n\
                  Content-Length: 11\r\n\r\n\
                  HELLO WORLD";

    assert_eq!(result, verify);
}

#[test]
fn test_response_state_close_body_multiple() {
    let chunks: &[&[u8]] = &[b"HTTP/1.1 200 OK\r\n\r\n", b"hello world ", b"more data"];
    let response = poll_oneone_multiple::<Response>(chunks);
    let verify = "HTTP/1.1 200 OK\r\n\
                  Content-Length: 21\r\n\r\n\
                  hello world more data";
    assert_eq!(response.into_bytes(), verify);
}
