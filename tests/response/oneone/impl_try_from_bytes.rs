use super::*;

#[test]
fn test_response_try_from_bytes_content_length_no_cl() {
    let input = BytesMut::from("HTTP/1.1 200 OK\r\n\r\nHello");
    let result = OneOne::<Response>::try_from(input).unwrap();
    let verify = BytesMut::from("HTTP/1.1 200 OK\r\nContent-Length: 5\r\n\r\nHello");
    assert_eq!(result.into_bytes(), verify);
}

#[test]
fn test_response_try_from_bytes_content_length_less() {
    let input = BytesMut::from("HTTP/1.1 200 OK\r\nContent-Length: 10\r\n\r\na");
    let result = OneOne::<Response>::try_from(input).unwrap();
    let verify = BytesMut::from("HTTP/1.1 200 OK\r\nContent-Length: 1\r\n\r\na");
    assert_eq!(result.into_bytes(), verify);
}

#[test]
fn test_response_try_from_bytes_content_length_more() {
    let input = BytesMut::from("HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\nHello");
    let result = OneOne::<Response>::try_from(input).unwrap();
    let verify = BytesMut::from("HTTP/1.1 200 OK\r\nContent-Length: 5\r\n\r\nHello");
    assert_eq!(result.into_bytes(), verify);
}
