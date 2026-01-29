use http_plz::ParseMessage;

use super::*;

#[test]
fn request_build_post_no_body_post() {
    let buf = BytesMut::from("POST / HTTP/1.1\r\n\r\n");
    let req = OneRequest::parse(buf).unwrap();
    let verify = "POST / HTTP/1.1\r\ncontent-length: 0\r\n\r\n";
    assert_eq!(req.into_bytes(), verify);
}

#[test]
fn request_build_with_content_length_less() {
    let buf = BytesMut::from("POST / HTTP/1.1\r\nContent-Length: 10\r\n\r\na");
    let req = OneRequest::parse(buf).unwrap();
    let verify = "POST / HTTP/1.1\r\nContent-Length: 1\r\n\r\na";
    assert_eq!(req.into_bytes(), verify);
}
