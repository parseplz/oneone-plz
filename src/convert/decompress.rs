use std::io::{BufRead, BufReader, Read, copy};

use brotli::Decompressor;
use bytes::{BufMut, BytesMut, buf::Writer};
use flate2::bufread::{DeflateDecoder, GzDecoder};
use header_plz::body_headers::content_encoding::ContentEncoding;

use super::error::DecompressError;

/* Description:
 *      Decompress data based on the Content-Encoding.
 *
 * Steps:
 *      Iterate over the encodings and decompress the data based on the
 *      encoding.
 */

fn decompress_brotli(data: &[u8], writer: &mut Writer<BytesMut>) -> Result<u64, DecompressError> {
    let mut reader = Decompressor::new(data, data.len());
    copy(&mut reader, writer).map_err(DecompressError::Brotli)
}

fn decompress_deflate(data: &[u8], writer: &mut Writer<BytesMut>) -> Result<u64, DecompressError> {
    let mut reader = DeflateDecoder::new(data);
    copy(&mut reader, writer).map_err(DecompressError::Deflate)
}

fn decompress_gzip(data: &[u8], writer: &mut Writer<BytesMut>) -> Result<u64, DecompressError> {
    let mut reader = GzDecoder::new(data);
    copy(&mut reader, writer).map_err(DecompressError::Gzip)
}

fn decompress_zstd(data: &[u8], writer: &mut Writer<BytesMut>) -> Result<u64, DecompressError> {
    let mut reader = zstd::stream::read::Decoder::new(data).map_err(DecompressError::Zstd)?;
    copy(&mut reader, writer).map_err(DecompressError::Zstd)
}

pub fn decompress(
    mut data: BytesMut,
    encodings: &[ContentEncoding],
) -> Result<BytesMut, DecompressError> {
    let mut result = BytesMut::with_capacity(data.len() * 2);
    let mut writer = result.writer();
    for encoding in encodings {
        match encoding {
            ContentEncoding::Brotli => decompress_brotli(&data[..], &mut writer),
            ContentEncoding::Gzip => decompress_gzip(&data, &mut writer),
            ContentEncoding::Deflate => decompress_deflate(&data, &mut writer),
            ContentEncoding::Identity | ContentEncoding::Chunked => continue,
            ContentEncoding::Zstd | ContentEncoding::Compress => {
                decompress_zstd(&data, &mut writer)
            }
        }?;
    }
    Ok(writer.into_inner())
}
