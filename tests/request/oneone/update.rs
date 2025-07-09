use super::*;
use oneone_plz::oneone::build::BuildFrame;

#[test]
fn request_update_post_no_body() {
    let buf = BytesMut::from("POST / HTTP/1.1\r\n\r\n");
    let req = OneOne::<Request>::build(buf).unwrap();
    let verify = "POST / HTTP/1.1\r\nContent-Length: 0\r\n\r\n";
    assert_eq!(req.into_bytes(), verify);
}

#[test]
fn request_update_with_content_length() {
    let buf = BytesMut::from("POST / HTTP/1.1\r\nContent-Length: 10\r\n\r\na");
    let req = OneOne::<Request>::build(buf).unwrap();
    let verify = "POST / HTTP/1.1\r\nContent-Length: 1\r\n\r\na";
    assert_eq!(req.into_bytes(), verify);
}

#[test]
fn request_update_no_content_length() {
    let buf = BytesMut::from("POST / HTTP/1.1\r\n\r\n");
    let req = OneOne::<Request>::build(buf).unwrap();
    let verify = "POST / HTTP/1.1\r\nContent-Length: 0\r\n\r\n";
    assert_eq!(req.into_bytes(), verify);
}
