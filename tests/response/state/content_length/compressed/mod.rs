use super::*;
use flate2::Compression;
use std::io::Write;
mod content_encoding;
mod transfer_encoding;

pub fn compressed_data() -> BytesMut {
    let data = b"hello world";
    let brotli_compressed = compress_brotli(data);
    let deflate_compressed = compress_deflate(&brotli_compressed);
    let gzip_compressed = compress_gzip(&deflate_compressed);
    let zstd_compressed = compress_zstd(&gzip_compressed);
    BytesMut::from(zstd_compressed.as_slice())
}

pub fn compress_brotli(data: &[u8]) -> Vec<u8> {
    let mut compressed = Vec::new();
    {
        let mut writer =
            brotli::CompressorWriter::new(&mut compressed, 4096, 0, 22);
        writer
            .write_all(data)
            .expect("br write");
        writer.flush().expect("br flush");
    }
    compressed
}

pub fn compress_deflate(data: &[u8]) -> Vec<u8> {
    let mut compressed = Vec::new();
    let mut encoder =
        flate2::write::ZlibEncoder::new(&mut compressed, Compression::fast());
    encoder
        .write_all(data)
        .expect("deflate write");
    encoder
        .finish()
        .expect("deflate finish");
    compressed
}

pub fn compress_gzip(data: &[u8]) -> Vec<u8> {
    let mut compressed = Vec::new();
    let mut encoder =
        flate2::write::GzEncoder::new(&mut compressed, Compression::fast());
    encoder
        .write_all(data)
        .expect("gzip write");
    encoder.finish().expect("gzip finish");
    compressed
}

pub fn compress_zstd(data: &[u8]) -> Vec<u8> {
    zstd::encode_all(data, 1).expect("zstd encode")
}
