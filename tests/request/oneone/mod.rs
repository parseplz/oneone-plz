use super::*;
mod build;
mod impl_try_from_bytes;
mod impl_try_from_state;

#[test]
fn test_request_is_connect_request_true() {
    let input = BytesMut::from("CONNECT www.google.com:80 HTTP/1.1\r\n\r\n");
    let result = OneOne::<Request>::try_from(input).unwrap();
    assert!(result.is_connect_request());
}

#[test]
fn test_request_is_connect_request_false() {
    let input = BytesMut::from("GET www.google.com:80 HTTP/1.1\r\n\r\n");
    let result = OneOne::<Request>::try_from(input).unwrap();
    assert!(!result.is_connect_request());
}
