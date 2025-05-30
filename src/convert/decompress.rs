use std::io::copy;

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

pub fn decompress_body(
    mut main_body: BytesMut,
    extra_body: Option<BytesMut>,
    encodings: &[ContentEncoding],
) -> Result<BytesMut, DecompressError> {
    let buf = BytesMut::with_capacity(
        2 * (main_body.len() + extra_body.as_ref().map(|b| b.len()).unwrap_or(0)),
    );
    let mut buf_writer = buf.writer();

    if let Some(extra) = extra_body {
        let main_org_len = main_body.len();

        // 1. concat extra and try
        main_body.reserve(extra.len());
        main_body.unsplit(extra);

        if decompress(&main_body[..], &mut buf_writer, encodings).is_ok() {
            return Ok(buf_writer.into_inner());
        }

        // 2. failed split extra from original
        let extra = main_body.split_off(main_org_len);
        // 3. decompress main
        decompress(&main_body[..], &mut buf_writer, encodings)?;
        let mut main_decompressed = buf_writer.get_mut().split();
        // writer.get_mut().clear();

        let extra_decompressed = match decompress(&extra[..], &mut buf_writer, encodings) {
            Ok(_) => buf_writer.get_mut().split(), // extra compressed separately
            Err(_) => extra,                       // extra clear
        };
        main_decompressed.unsplit(extra_decompressed);
        Ok(main_decompressed)
    } else {
        decompress(&main_body[..], &mut buf_writer, encodings)?;
        Ok(buf_writer.into_inner())
    }
}

fn decompress(
    data: &[u8],
    writer: &mut Writer<BytesMut>,
    encodings: &[ContentEncoding],
) -> Result<(), DecompressError> {
    for encoding in encodings {
        match encoding {
            ContentEncoding::Brotli => decompress_brotli(data, writer),
            ContentEncoding::Gzip => decompress_gzip(data, writer),
            ContentEncoding::Deflate => decompress_deflate(data, writer),
            ContentEncoding::Identity | ContentEncoding::Chunked => continue,
            ContentEncoding::Zstd | ContentEncoding::Compress => decompress_zstd(data, writer),
        }?;
    }
    Ok(())
}
