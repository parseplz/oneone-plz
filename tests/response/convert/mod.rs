use super::*;

mod content_length;

#[test]
fn test_response_convert_no_cl() {
    let input = "HTTP/1.1 200 OK\r\n\
               Host: reqbin.com\r\n\
               Content-Type: text/plain\r\n\r\n\
               MozillaDeveloperNetwork";
    let verify = "HTTP/1.1 200 OK\r\n\
                  Host: reqbin.com\r\n\
                  Content-Type: text/plain\r\n\
                  Content-Length: 23\r\n\r\n\
                  MozillaDeveloperNetwork";

    let result = poll_state_result_with_end::<Response>(input.as_bytes())
        .unwrap()
        .try_into_frame()
        .unwrap()
        .into_bytes();
    assert_eq!(result, verify);
}
