use bytes::BufMut;
use std::io::{Read, Write};

use flate2::{
    Compression,
    read::{DeflateEncoder, GzEncoder},
};

use super::*;

fn compressed_data() -> Vec<u8> {
    let data = b"hello world";
    let mut compressed = BytesMut::new();
    let mut buf_writer = compressed.writer();
    // brotli
    let mut br = brotli::CompressorWriter::new(&mut buf_writer, 4096, 11, 22);
    let _ = br.write_all(&data[..]);
    let _ = br.flush();
    drop(br);
    compressed = buf_writer.into_inner();

    // deflate
    let mut deflater = DeflateEncoder::new(&compressed[..], Compression::fast());
    let mut compressed = Vec::new();
    deflater.read_to_end(&mut compressed).unwrap();

    // gzip
    let mut gz = GzEncoder::new(&compressed[..], Compression::fast());
    let mut compressed = Vec::new();
    gz.read_to_end(&mut compressed).unwrap();

    // zstd

    zstd::encode_all(&compressed[..], 1).unwrap()
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
    let result = parse_full_single::<Response>(&input);
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
    let result = parse_full_single::<Response>(&input);
    let verify = "HTTP/1.1 200 OK\r\n\
                  Host: reqbin.com\r\n\
                  Content-Type: text/plain\r\n\
                  Content-Length: 11\r\n\r\n\
                  hello world";

    assert_eq!(result.into_bytes(), verify);
}
