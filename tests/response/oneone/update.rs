use super::*;
use oneone_plz::oneone::update::BuildHttp;

#[test]
fn response_update() {
    let buf = BytesMut::from("HTTP/1.1 200 OK\r\n\r\nhello");
    let req = OneOne::<Response>::build(buf).unwrap();
    let verify = "HTTP/1.1 200 OK\r\nContent-Length: 5\r\n\r\nhello";
    assert_eq!(req.into_bytes(), verify);
}
