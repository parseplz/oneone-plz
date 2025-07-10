use super::*;
use bytes::BufMut;
use flate2::Compression;
use std::io::{Read, Write};
mod transfer_encoding;

use super::*;
mod basic;
mod error;

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
    let mut deflater = flate2::read::ZlibEncoder::new(&compressed[..], Compression::fast());
    let mut compressed = Vec::new();
    deflater.read_to_end(&mut compressed).unwrap();

    // gzip
    let mut gz = flate2::read::GzEncoder::new(&compressed[..], Compression::fast());
    let mut compressed = Vec::new();
    gz.read_to_end(&mut compressed).unwrap();

    // zstd
    zstd::encode_all(&compressed[..], 1).unwrap()
}
