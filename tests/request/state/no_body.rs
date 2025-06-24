use super::*;

#[test]
fn test_request_state_get_success() {
    let input = "GET /echo HTTP/1.1\r\n\
                 Host: reqbin.com\r\n\r\n";

    let result = poll_oneone_only_read::<Request>(input.as_bytes());
    assert_eq!(result.message_head().infoline().method(), b"GET");
}
