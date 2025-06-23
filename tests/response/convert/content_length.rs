use oneone_plz::convert::content_length::update_content_length;

use super::*;

#[test]
fn test_response_convert_update_content_length_no_cl() {
    let input = "HTTP/1.1 200 OK\r\n\
                 Host: reqbin.com\r\n\
                 Content-Type: text/plain\r\n\r\n";

    let mut result = OneOne::<Response>::try_from_message_head_buf(BytesMut::from(input)).unwrap();
    update_content_length(&mut result, 23);
    let verify = "HTTP/1.1 200 OK\r\n\
                  Host: reqbin.com\r\n\
                  Content-Type: text/plain\r\n\
                  Content-Length: 23\r\n\r\n";
    assert_eq!(result.into_bytes(), verify);
}

#[test]
fn test_response_convert_update_content_length_with_cl() {
    let input = "HTTP/1.1 200 OK\r\n\
                 Host: reqbin.com\r\n\
                 Content-Type: text/plain\r\n\
                 Content-Length: 23\r\n\r\n";

    let mut result = OneOne::<Response>::try_from_message_head_buf(BytesMut::from(input)).unwrap();
    update_content_length(&mut result, 27);
    let verify = "HTTP/1.1 200 OK\r\n\
                  Host: reqbin.com\r\n\
                  Content-Type: text/plain\r\n\
                  Content-Length: 27\r\n\r\n";
    assert_eq!(result.into_bytes(), verify);
}
