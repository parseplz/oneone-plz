use super::*;

#[test]
fn test_response_state_error_compression() {
    let input = "HTTP/1.1 200 OK\r\n\
                  Content-Encoding: gzip\r\n\
                  Content-Length: 11\r\n\r\n\
                  hello world";
    let result = poll_state_result_with_end::<Response>(input.as_bytes())
        .unwrap()
        .try_into_frame()
        .unwrap()
        .into_bytes();
    assert_eq!(result, input);
}
