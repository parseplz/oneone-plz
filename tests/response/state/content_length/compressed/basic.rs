use super::*;

#[test]
fn test_response_state_content_length_brotli() {
    let input = b"HTTP/1.1 200 OK\r\n\
                Content-Length: 15\r\n\
                Content-Encoding: br\r\n\r\n\
                \x0b\x05\x80\x68\x65\x6c\x6c\x6f\x20\x77\x6f\x72\x6c\x64\x03";
    let response = poll_oneone_only_read::<Response>(input);
    assert_eq!(response.status_code(), "200");
    let verify = "HTTP/1.1 200 OK\r\n\
                  Content-Length: 11\r\n\r\n\
                  hello world";
    assert_eq!(response.into_bytes(), verify);
}

#[test]
fn test_response_state_content_length_gzip() {
    let input = b"HTTP/1.1 200 OK\r\n\
                Content-Length: 41\r\n\
                Content-Encoding: gzip\r\n\r\n\
                \x1f\x8b\x08\x00\x7e\x6c\xea\x65\x00\xff\x05\x80\x41\x09\x00\x00\x08\xc4\xaa\x18\x4e\xc1\xc7\xe0\xc0\x8f\xf5\xc7\x0e\xa4\x3e\x47\x0b\x85\x11\x4a\x0d\x0b\x00\x00\x00";
    let response = poll_oneone_only_read::<Response>(input);
    let verify = "HTTP/1.1 200 OK\r\n\
                  Content-Length: 11\r\n\r\n\
                  hello world";
    assert_eq!(response.into_bytes(), verify);
}

#[test]
fn test_response_state_content_length_zstd() {
    let input = b"HTTP/1.1 200 OK\r\n\
                Content-Length: 24\r\n\
                Content-Encoding: zstd\r\n\r\n\
                \x28\xb5\x2f\xfd\x24\x0b\x59\x00\x00\x68\x65\x6c\x6c\x6f\x20\x77\x6f\x72\x6c\x64\x68\x69\x1e\xb2";
    let response = poll_oneone_only_read::<Response>(input);
    let verify = "HTTP/1.1 200 OK\r\n\
                  Content-Length: 11\r\n\r\n\
                  hello world";
    assert_eq!(response.into_bytes(), verify);
}

#[test]
fn test_response_decompress_all_single() {
    let compressed = compressed_data();
    let mut input: Vec<u8> = format!(
        "HTTP/1.1 200 OK\r\n\
        Host: reqbin.com\r\n\
        Content-Type: text/plain\r\n\
        Content-Length: {}\r\n\
        Content-Encoding: br, deflate, gzip, zstd\r\n\r\n",
        compressed.len()
    )
    .into();
    input.extend_from_slice(&compressed[..]);
    let result = poll_oneone_only_read::<Response>(&input);
    let verify = "HTTP/1.1 200 OK\r\n\
                  Host: reqbin.com\r\n\
                  Content-Type: text/plain\r\n\
                  Content-Length: 11\r\n\r\n\
                  hello world";

    assert_eq!(result.into_bytes(), verify);
}

#[test]
fn test_response_decompress_all_multiple() {
    let compressed = compressed_data();
    let mut input: Vec<u8> = format!(
        "HTTP/1.1 200 OK\r\n\
        Content-Encoding: br\r\n\
        Host: reqbin.com\r\n\
        Content-Encoding: deflate\r\n\
        Content-Type: text/plain\r\n\
        Content-Encoding: gzip \r\n\
        Content-Length: {}\r\n\
        Content-Encoding: zstd\r\n\r\n",
        compressed.len()
    )
    .into();
    input.extend_from_slice(&compressed[..]);
    let result = poll_oneone_only_read::<Response>(&input);
    let verify = "HTTP/1.1 200 OK\r\n\
                  Host: reqbin.com\r\n\
                  Content-Type: text/plain\r\n\
                  Content-Length: 11\r\n\r\n\
                  hello world";

    assert_eq!(result.into_bytes(), verify);
}
