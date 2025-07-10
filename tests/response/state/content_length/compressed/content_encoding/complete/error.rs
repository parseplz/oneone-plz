use super::*;

#[test]
fn test_response_state_cl_decompress_ce_complete_error_single_ce() {
    let input = "HTTP/1.1 200 OK\r\n\
                  Content-Encoding: gzip\r\n\
                  Content-Length: 11\r\n\r\n\
                  hello world";
    let result = poll_oneone_only_read::<Response>(input.as_bytes());
    assert_eq!(result.into_bytes(), input);
}

#[test]
fn test_response_state_cl_decompress_ce_complete_error_multiple_ce() {
    let input = "HTTP/1.1 200 OK\r\n\
                 Content-Encoding: gzip, br, deflate, gzip, zstd\r\n\
                 Content-Length: 11\r\n\r\n\
                 hello world";
    let result = poll_oneone_only_read::<Response>(input.as_bytes());
    assert_eq!(result.into_bytes(), input);
}

#[test]
fn test_response_state_cl_decompress_ce_complete_error_multiple_ce_single_header() {
    let compressed = compressed_data();
    let mut input: Vec<u8> = format!(
        "HTTP/1.1 200 OK\r\n\
        Host: reqbin.com\r\n\
        Content-Type: text/plain\r\n\
        Content-Length: {}\r\n\
        Content-Encoding: gzip, br, deflate, gzip, zstd\r\n\r\n",
        compressed.len()
    )
    .into();
    input.extend_from_slice(&compressed[..]);
    let result = poll_oneone_only_read::<Response>(&input);
    let verify = "HTTP/1.1 200 OK\r\n\
                  Host: reqbin.com\r\n\
                  Content-Type: text/plain\r\n\
                  Content-Length: 11\r\n\
                  Content-Encoding: gzip\r\n\r\n\
                  hello world";
    assert_eq!(result.into_bytes(), verify);
}

#[test]
fn test_response_state_cl_decompress_ce_complete_error_multiple_ce_multiple_header() {
    let compressed = compressed_data();
    let mut input: Vec<u8> = format!(
        "HTTP/1.1 200 OK\r\n\
        Content-Encoding: gzip\r\n\
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
                  Content-Encoding: gzip\r\n\
                  Host: reqbin.com\r\n\
                  Content-Type: text/plain\r\n\
                  Content-Length: 11\r\n\r\n\
                  hello world";
    assert_eq!(result.into_bytes(), verify);
}
