use super::*;

#[test]
fn test_request_try_from_bytes_content_length_no_body() {
    let req = "POST / HTTP/1.1\r\n\r\n";
    let input = BytesMut::from(req);
    let result = OneOne::<Request>::try_from(input).unwrap();
    assert_eq!(result.into_bytes(), req);
}

#[test]
fn test_request_try_from_bytes_content_length_no_cl() {
    let input = BytesMut::from("POST / HTTP/1.1\r\n\r\nHello");
    let result = OneOne::<Request>::try_from(input).unwrap();
    let verify =
        BytesMut::from("POST / HTTP/1.1\r\nContent-Length: 5\r\n\r\nHello");
    assert_eq!(result.into_bytes(), verify);
}

#[test]
fn test_request_try_from_bytes_content_length_less() {
    let input =
        BytesMut::from("POST / HTTP/1.1\r\nContent-Length: 10\r\n\r\na");
    let result = OneOne::<Request>::try_from(input).unwrap();
    let verify =
        BytesMut::from("POST / HTTP/1.1\r\nContent-Length: 1\r\n\r\na");
    assert_eq!(result.into_bytes(), verify);
}

#[test]
fn test_request_try_from_bytes_content_length_more() {
    let input =
        BytesMut::from("POST / HTTP/1.1\r\nContent-Length: 0\r\n\r\nHello");
    let result = OneOne::<Request>::try_from(input).unwrap();
    let verify =
        BytesMut::from("POST / HTTP/1.1\r\nContent-Length: 5\r\n\r\nHello");
    assert_eq!(result.into_bytes(), verify);
}
