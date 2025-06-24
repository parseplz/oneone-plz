use oneone_plz::oneone::update::UpdateHttp;

use super::*;
mod update;

#[test]
fn test_response_methods() {
    let buf = BytesMut::from("HTTP/1.1 200 OK\r\n\r\n");
    let res = OneOne::<Response>::update(buf).unwrap();
    assert_eq!(res.content_length(), 0);
}
