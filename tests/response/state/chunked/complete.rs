use super::*;

#[test]
fn test_response_state_chunked_single() {
    let input = "HTTP/1.1 200 OK\r\n\
                 Transfer-Encoding: chunked\r\n\r\n\
                 6\r\n\
                 hello \r\n\
                 5\r\n\
                 world\r\n\
                 0\r\n\r\n";
    let verify = "HTTP/1.1 200 OK\r\n\
                  Content-Length: 11\r\n\r\n\
                  hello world";
    let mut result = poll_oneone_only_read::<Response>(input.as_bytes());
    let mut buf = BytesMut::new();
    result.decode(&mut buf).unwrap();
    assert_eq!(result.into_bytes(), verify);
}

#[test]
fn test_response_state_chunked_multiple() {
    let chunks: &[&[u8]] = &[
        b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n5\r\nhello\r\n",
        b"5\r\nworld\r\n",
        b"0\r\n\r\n",
    ];
    let verify = "HTTP/1.1 200 OK\r\n\
                    Content-Length: 10\r\n\r\n\
                    helloworld";
    let mut result = poll_oneone_multiple::<Response>(chunks);
    let mut buf = BytesMut::new();
    result.decode(&mut buf).unwrap();
    assert_eq!(result.into_bytes(), verify);
}

#[test]
fn test_response_state_chunked_multiple_large() {
    let chunks: &[&[u8]] = &[
        b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n",
        &b"B\r\nhello world\r\n".repeat(100),
        b"0\r\n\r\n",
    ];

    let mut result = poll_oneone_multiple::<Response>(chunks);
    let verify = "HTTP/1.1 200 OK\r\n\
                    Content-Length: 1100\r\n\r\n"
        .to_string()
        + &"hello world".repeat(100);
    assert_eq!(result.status_code(), "200");

    let mut buf = BytesMut::new();
    result.decode(&mut buf).unwrap();
    assert_eq!(result.into_bytes(), verify);
}
