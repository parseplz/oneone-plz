use std::io::Read;

use brotli::Decompressor;
use bytes::BytesMut;
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

pub fn decompress(
    mut data: BytesMut,
    encodings: &[ContentEncoding],
) -> Result<BytesMut, DecompressError> {
    for encoding in encodings {
        let result = match encoding {
            ContentEncoding::Brotli => decompress_brotli(&data),
            ContentEncoding::Deflate => decompress_deflate(&data),
            ContentEncoding::Gzip => decompress_gzip(&data),
            ContentEncoding::Zstd => decompress_zstd(&data),
            ContentEncoding::Identity | ContentEncoding::Chunked => continue,
            ContentEncoding::Compress => decompress_zstd(&data),
        }?;
        data.clear();
        data.reserve(result.len());
        data.extend_from_slice(&result);
    }
    Ok(data)
}

fn decompress_brotli(data: &[u8]) -> Result<Vec<u8>, DecompressError> {
    let mut uncompressed_data = Decompressor::new(data, data.len());
    let mut buf = Vec::new();
    uncompressed_data
        .read_to_end(&mut buf)
        .map_err(DecompressError::Brotli)?;
    Ok(buf)
}

fn decompress_deflate(data: &[u8]) -> Result<Vec<u8>, DecompressError> {
    let mut deflater = DeflateDecoder::new(data);
    let mut buf = Vec::new();
    deflater
        .read_to_end(&mut buf)
        .map_err(DecompressError::Deflate)?;
    Ok(buf)
}

fn decompress_gzip(data: &[u8]) -> Result<Vec<u8>, DecompressError> {
    let mut gz = GzDecoder::new(data);
    let mut buf = Vec::new();
    gz.read_to_end(&mut buf).map_err(DecompressError::Gzip)?;
    Ok(buf)
}

fn decompress_zstd(data: &[u8]) -> Result<Vec<u8>, DecompressError> {
    let mut decoder = zstd::stream::read::Decoder::new(data).map_err(DecompressError::Zstd)?;
    let mut buf = Vec::new();
    decoder
        .read_to_end(&mut buf)
        .map_err(DecompressError::Zstd)?;
    Ok(buf)
}
