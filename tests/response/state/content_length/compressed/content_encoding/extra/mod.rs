use super::*;
mod error;

// if separately compressed extra can't be decompressed
// #[test]
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
    dbg!(String::from_utf8_lossy(&input));
    let result: State<Response> = poll_state_result_with_end(&input).unwrap();
    let response: OneOne<Response> = result.try_into_frame().unwrap();
    //dbg!(response);
}

#[test]
fn test_response_state_cl_decompress_ce_extra_brotli_single_compression() {
    let compressed = compress_brotli(b"hello world hola amigo");
    dbg!(compressed.len());
    let mut input: Vec<u8> = format!(
        "HTTP/1.1 200 OK\r\n\
        Host: reqbin.com\r\n\
        Content-Type: text/plain\r\n\
        Content-Length: 20\r\n\
        Content-Encoding: br\r\n\r\n",
    )
    .into();
    input.extend_from_slice(&compressed[..]);
    dbg!(String::from_utf8_lossy(&input));
    let result: State<Response> = poll_state_result_with_end(&input).unwrap();
    let response: OneOne<Response> = result.try_into_frame().unwrap();
    //dbg!(response);
}

#[test]
fn test_response_state_cl_decompress_ce_extra_gzip() {
    let compressed = compress_gzip(b"hello world");
    let mut input: Vec<u8> = format!(
        "HTTP/1.1 200 OK\r\n\
        Host: reqbin.com\r\n\
        Content-Type: text/plain\r\n\
        Content-Length: {}\r\n\
        Content-Encoding: gzip\r\n\r\n",
        compressed.len()
    )
    .into();
    input.extend_from_slice(&compressed[..]);
    let compressed = compress_gzip(b" hola amigo");
    input.extend_from_slice(&compressed[..]);
    dbg!(String::from_utf8_lossy(&input));
    let result: State<Response> = poll_state_result_with_end(&input).unwrap();
    let response: OneOne<Response> = result.try_into_frame().unwrap();
    //dbg!(response);
}
