use crate::response::parse_full_response;

use protocol_traits_plz::Frame;
use protocol_traits_plz::Step;

#[test]
fn test_response_content_length_basic() {
    let input = "HTTP/1.1 200 OK\r\n\
                 Content-Length: 5\r\n\r\n\
                 hello";
    let response = parse_full_response(input.as_bytes());
    assert_eq!(response.status_code(), "200");
    let result = response.into_data();
    assert_eq!(result, input);
}

#[test]
fn test_response_content_length_zero() {
    let input = "HTTP/1.1 307 OK\r\n\
                 Location: /index.html\r\n\
                 Content-Length: 0\r\n\r\n";
    let response = parse_full_response(input.as_bytes());
    assert_eq!(response.status_code(), "307");
    let result = response.into_data();
    assert_eq!(result, input);
}

#[test]
fn test_response_content_length_brotli() {
    let input = b"HTTP/1.1 200 OK\r\n\
                Content-Length: 15\r\n\
                Content-Encoding: br\r\n\r\n\
                \x0b\x05\x80\x68\x65\x6c\x6c\x6f\x20\x77\x6f\x72\x6c\x64\x03";
    let response = parse_full_response(input);
    assert_eq!(response.status_code(), "200");
    let verify = "HTTP/1.1 200 OK\r\n\
                  Content-Length: 11\r\n\r\n\
                  hello world";
    assert_eq!(response.into_data(), verify);
}

#[test]
fn test_response_content_length_gzip() {
    let input = b"HTTP/1.1 200 OK\r\n\
                Content-Length: 41\r\n\
                Content-Encoding: gzip\r\n\r\n\
                \x1f\x8b\x08\x00\x7e\x6c\xea\x65\x00\xff\x05\x80\x41\x09\x00\x00\x08\xc4\xaa\x18\x4e\xc1\xc7\xe0\xc0\x8f\xf5\xc7\x0e\xa4\x3e\x47\x0b\x85\x11\x4a\x0d\x0b\x00\x00\x00";
    let response = parse_full_response(input);
    let verify = "HTTP/1.1 200 OK\r\n\
                  Content-Length: 11\r\n\r\n\
                  hello world";
    assert_eq!(response.into_data(), verify);
}

// #[test]
// FIX: corrupt deflate stream
fn test_response_content_length_deflate() {
    let input = b"HTTP/1.1 200 OK\r\n\
                Content-Length: 29\r\n\
                Content-Encoding: deflate\r\n\r\n\
                \x78\x9c\x05\x80\x41\x09\x00\x00\x08\xc4\xaa\x18\x4e\xc1\xc7\xe0\xc0\x8f\xf5\xc7\x0e\xa4\x3e\x47\x0b\x1a\x0b\x04\x5d";
    let response = parse_full_response(input);
    let verify = "HTTP/1.1 200 OK\r\n\
                  Content-Length: 11\r\n\r\n\
                  hello world";
    assert_eq!(response.into_data(), verify);
}

#[test]
fn test_response_content_length_zstd() {
    let input = b"HTTP/1.1 200 OK\r\n\
                Content-Length: 24\r\n\
                Content-Encoding: zstd\r\n\r\n\
                \x28\xb5\x2f\xfd\x24\x0b\x59\x00\x00\x68\x65\x6c\x6c\x6f\x20\x77\x6f\x72\x6c\x64\x68\x69\x1e\xb2";
    let response = parse_full_response(input);
    let verify = "HTTP/1.1 200 OK\r\n\
                  Content-Length: 11\r\n\r\n\
                  hello world";
    assert_eq!(response.into_data(), verify);
}

#[test]
fn test_response_cl_large() {
    let mut input = "HTTP/1.1 200 OK\r\n\
                     Content-Length: 1100\r\n\r\n"
        .to_string();
    input.push_str(&"hello world".repeat(100));
    let _ = parse_full_response(input.as_bytes());
}
