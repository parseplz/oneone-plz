use super::*;
use oneone_plz::oneone::build::BuildMessage;

#[test]
fn request_build_post_no_body_post() {
    let buf = BytesMut::from("POST / HTTP/1.1\r\n\r\n");
    let req = OneOne::<Request>::build(buf).unwrap();
    let verify = "POST / HTTP/1.1\r\nContent-Length: 0\r\n\r\n";
    assert_eq!(req.into_bytes(), verify);
}

#[test]
fn request_build_with_content_length_less() {
    let buf = BytesMut::from("POST / HTTP/1.1\r\nContent-Length: 10\r\n\r\na");
    let req = OneOne::<Request>::build(buf).unwrap();
    let verify = "POST / HTTP/1.1\r\nContent-Length: 1\r\n\r\na";
    assert_eq!(req.into_bytes(), verify);
}
