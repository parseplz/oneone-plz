use super::*;

#[test]
fn test_request_state_partial_info_line_only() {
    let input = "GET /echo HTTP/1.1\r\n";
    let result = poll_state_result_with_end::<Request>(input.as_bytes());
    assert!(matches!(result, Err(HttpStateError::Unparsed(_))));
}

#[test]
fn test_request_state_read_message_head() {
    let input = "GET /partial HTTP/1.1\r\n\
                 Host: example.com\r\n";
    let (_, result) = poll_state_once::<Request>(input.as_bytes());
    assert!(matches!(result, State::ReadMessageHead));
}
