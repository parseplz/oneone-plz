use super::*;

#[test]
fn test_request_convert_chunked_to_raw() {
    let input = "POST /echo HTTP/1.1\r\n\
                 Host: reqbin.com\r\n\
                 Trailer: Some\r\n\
                 Transfer-Encoding: chunked\r\n\r\n\
                 7\r\n\
                 Mozilla\r\n\
                 9\r\n\
                 Developer\r\n\
                 7\r\n\
                 Network\r\n\
                 0\r\n\
                 Header: Val\r\n\
                 \r\n";
    let verify = "POST /echo HTTP/1.1\r\n\
                  Host: reqbin.com\r\n\
                  Header: Val\r\n\
                  Content-Length: 23\r\n\r\n\
                  MozillaDeveloperNetwork";
    let mut result = poll_oneone_only_read::<OneRequestLine>(input.as_bytes());
    let mut buf = BytesMut::new();
    result.try_decompress(&mut buf).unwrap();
    assert_eq!(result.into_bytes(), verify);
}

#[test]
fn test_request_convert_chunked_to_raw_extra_body() {
    let input = "POST /echo HTTP/1.1\r\n\
                 Host: reqbin.com\r\n\
                 Trailer: Some\r\n\
                 Transfer-Encoding: chunked\r\n\r\n\
                 7\r\n\
                 Mozilla\r\n\
                 9\r\n\
                 Developer\r\n\
                 7\r\n\
                 Network\r\n\
                 0\r\n\
                 Header: Val\r\n\
                 \r\n";
    let verify = "POST /echo HTTP/1.1\r\n\
                  Host: reqbin.com\r\n\
                  Header: Val\r\n\
                  Content-Length: 23\r\n\r\n\
                  MozillaDeveloperNetwork";
    let mut result = poll_oneone_only_read::<OneRequestLine>(input.as_bytes());
    let mut buf = BytesMut::new();
    result.try_decompress(&mut buf).unwrap();
    assert_eq!(result.into_bytes(), verify);
}
