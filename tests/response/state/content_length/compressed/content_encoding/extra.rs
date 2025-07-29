use super::*;

const VERIFY: &str = "HTTP/1.1 200 OK\r\n\
                  Host: reqbin.com\r\n\
                  Content-Type: text/plain\r\n\
                  Content-Length: 22\r\n\r\n\
                  hello world hola amigo";

#[test]
fn test_response_state_cl_decompress_ce_extra_brotli_raw() {
    let compressed = compress_brotli(b"hello world");
    let mut input: Vec<u8> = format!(
        "HTTP/1.1 200 OK\r\n\
        Host: reqbin.com\r\n\
        Content-Type: text/plain\r\n\
        Content-Length: {}\r\n\
        Content-Encoding: br\r\n\r\n",
        compressed.len()
    )
    .into();
    input.extend_from_slice(&compressed[..]);
    input.extend_from_slice(b" hola amigo");
    let mut response: OneOne<Response> = poll_state_result_with_end(&input)
        .unwrap()
        .try_into_frame()
        .unwrap();
    let mut buf = BytesMut::new();
    response.decode(&mut buf);
    assert_eq!(response.into_bytes(), VERIFY);
}

// if separately compressed extra can't be decompressed
#[test]
fn test_response_state_cl_decompress_ce_extra_brotli_separate_compression() {
    let compressed = compress_brotli(b"hello world");
    let mut input: Vec<u8> = format!(
        "HTTP/1.1 200 OK\r\n\
        Host: reqbin.com\r\n\
        Content-Type: text/plain\r\n\
        Content-Length: {}\r\n\
        Content-Encoding: br\r\n\r\n",
        compressed.len()
    )
    .into();
    input.extend_from_slice(&compressed[..]);
    let compressed = compress_brotli(b" hola amigo");
    input.extend_from_slice(&compressed[..]);
    let mut response: OneOne<Response> = poll_state_result_with_end(&input)
        .unwrap()
        .try_into_frame()
        .unwrap();
    let mut buf = BytesMut::new();
    response.decode(&mut buf);
    assert_eq!(response.into_bytes(), VERIFY);
}

#[test]
fn test_response_state_cl_decompress_ce_extra_brotli_single_compression() {
    let compressed = compress_brotli(b"hello world hola amigo");
    let mut input: Vec<u8> = format!(
        "HTTP/1.1 200 OK\r\n\
        Host: reqbin.com\r\n\
        Content-Type: text/plain\r\n\
        Content-Length: {}\r\n\
        Content-Encoding: br\r\n\r\n",
        compressed.len()
    )
    .into();
    input.extend_from_slice(&compressed[..]);
    let mut response: OneOne<Response> = poll_state_result_with_end(&input)
        .unwrap()
        .try_into_frame()
        .unwrap();
    let mut buf = BytesMut::new();
    response.decode(&mut buf);
    assert_eq!(response.into_bytes(), VERIFY);
}
