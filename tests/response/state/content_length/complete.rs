use super::*;

#[test]
fn test_response_state_content_length() {
    let input = "HTTP/1.1 200 OK\r\n\
                 Content-Length: 5\r\n\r\n\
                 hello";
    let response = poll_oneone_only_read::<Response>(input.as_bytes());
    assert_eq!(response.status_code(), "200");
    assert_eq!(response.into_bytes(), input);
}

#[test]
fn test_response_state_content_length_zero() {
    let input = "HTTP/1.1 307 OK\r\n\
                 Location: /index.html\r\n\
                 Content-Length: 0\r\n\r\n";
    let response = poll_oneone_only_read::<Response>(input.as_bytes());
    assert_eq!(response.status_code(), "307");
    assert_eq!(response.into_bytes(), input);
}

#[test]
fn test_response_state_content_length_large() {
    let mut input = "HTTP/1.1 200 OK\r\n\
                     Content-Length: 1100\r\n\r\n"
        .to_string();
    input.push_str(&"hello world".repeat(100));
    let verify = input.clone();
    let response = poll_oneone_only_read::<Response>(input.as_bytes());
    assert_eq!(response.status_code(), "200");
    assert_eq!(response.into_bytes(), verify);
}
