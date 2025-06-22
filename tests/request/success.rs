use super::*;

#[test]
fn test_request_state_get_success() {
    let input = "GET /echo HTTP/1.1\r\n\
                   Host: reqbin.com\r\n\r\n";

    let request = parse_full_single::<Request>(input.as_bytes());
    assert_eq!(request.message_head().infoline().method(), b"GET");
}
