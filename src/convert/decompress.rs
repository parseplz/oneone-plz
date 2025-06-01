use std::io::copy;

use bytes::{BufMut, BytesMut, buf::Writer};
use header_plz::body_headers::content_encoding::ContentEncoding;

use std::io::Error;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum DecompressError {
    #[error("brotli| {0}")]
    Brotli(Error),
    #[error("deflate| {0}")]
    Deflate(Error),
    #[error("gzip| {0}")]
    Gzip(Error),
    #[error("zstd| {0}")]
    Zstd(Error),
}

pub fn decompress_body(
    mut main_body: BytesMut,
    extra_body: Option<BytesMut>,
    encodings: &[ContentEncoding],
) -> Result<BytesMut, DecompressError> {
    let capacity = 2 * (main_body.len() + extra_body.as_ref().map(|b| b.len()).unwrap_or(0));
    let mut buf = BytesMut::with_capacity(capacity);
    let mut buf_writer = buf.writer();

    if let Some(extra) = extra_body {
        let main_org_len = main_body.len();

        // 1. concat extra and try
        main_body.unsplit(extra);

        match decompress(&main_body[..], &mut buf_writer, encodings) {
            Ok(out) => return Ok(out),
            Err(e) => {
                buf_writer.get_mut().try_reclaim(capacity);
                buf_writer.get_mut().clear();
            }
        }

        // 2. split extra from original
        let extra = main_body.split_off(main_org_len);
        // 3. decompress main
        let mut main_decompressed = decompress(&main_body[..], &mut buf_writer, encodings)?;

        let extra_decompressed = match decompress(&extra[..], &mut buf_writer, encodings) {
            Ok(out) => out,  // compressed separately
            Err(_) => extra, // clear
        };
        main_decompressed.unsplit(extra_decompressed);
        Ok(main_decompressed)
    } else {
        decompress(&main_body[..], &mut buf_writer, encodings)
    }
}

pub fn decompress(
    compressed: &[u8],
    writer: &mut Writer<BytesMut>,
    encodings: &[ContentEncoding],
) -> Result<BytesMut, DecompressError> {
    let mut input: &[u8] = compressed;
    let mut output = writer.get_mut().split();

    for &enc in encodings.iter().rev() {
        match enc {
            ContentEncoding::Brotli => decompress_brotli(input, writer),
            ContentEncoding::Gzip => decompress_gzip(input, writer),
            ContentEncoding::Deflate => decompress_deflate(input, writer),
            ContentEncoding::Identity | ContentEncoding::Chunked => continue,
            ContentEncoding::Zstd | ContentEncoding::Compress => decompress_zstd(input, writer),
        }?;
        output = writer.get_mut().split();
        input = &output[..];
    }
    Ok(output)
}

#[inline]
fn decompress_brotli(data: &[u8], writer: &mut Writer<BytesMut>) -> Result<u64, DecompressError> {
    let mut reader = brotli::Decompressor::new(data, data.len());
    copy(&mut reader, writer).map_err(DecompressError::Brotli)
}

#[inline]
fn decompress_deflate(data: &[u8], writer: &mut Writer<BytesMut>) -> Result<u64, DecompressError> {
    let mut reader = flate2::bufread::DeflateDecoder::new(data);
    copy(&mut reader, writer).map_err(DecompressError::Deflate)
}

#[inline]
fn decompress_gzip(data: &[u8], writer: &mut Writer<BytesMut>) -> Result<u64, DecompressError> {
    let mut reader = flate2::bufread::GzDecoder::new(data);
    copy(&mut reader, writer).map_err(DecompressError::Gzip)
}

#[inline]
fn decompress_zstd(data: &[u8], writer: &mut Writer<BytesMut>) -> Result<u64, DecompressError> {
    let mut reader = zstd::stream::read::Decoder::new(data).map_err(DecompressError::Zstd)?;
    copy(&mut reader, writer).map_err(DecompressError::Zstd)
}

mod tests {
    use std::io::{Read, Write};

    use flate2::{
        Compression,
        read::{DeflateEncoder, GzEncoder},
    };

    use super::*;

    #[test]
    fn test_decompress() {
        let data = b"hello world";
        let encodings = [
            ContentEncoding::Brotli,
            ContentEncoding::Deflate,
            ContentEncoding::Gzip,
            ContentEncoding::Zstd,
        ];
        let mut compressed = BytesMut::new();
        let mut buf_writer = compressed.writer();
        // brotli
        let mut br = brotli::CompressorWriter::new(&mut buf_writer, 4096, 11, 22);
        br.write_all(&data[..]);
        br.flush();
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
        let compressed = zstd::encode_all(&compressed[..], 1).unwrap();

        let mut result = BytesMut::new();
        let mut writer = result.writer();
        let out = decompress(&compressed[..], &mut writer, &encodings).unwrap();
        assert_eq!(&out[..], &data[..]);
    }
}
