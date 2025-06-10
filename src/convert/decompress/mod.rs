use std::io::copy;

use bytes::{BufMut, BytesMut, buf::Writer};
use header_plz::{
    body_headers::{content_encoding::ContentEncoding, parse::ParseBodyHeaders},
    const_headers::TRANSFER_ENCODING,
    info_line::InfoLine,
    message_head::MessageHead,
};
pub mod error;
use crate::{convert::decompress::error::DEStruct, oneone::OneOne};
use error::DecompressError;
use std::io::Error;
use thiserror::Error;

pub fn apply_compression<T>(
    one: &mut OneOne<T>,
    encodings: &[ContentEncoding],
    mut body: BytesMut,
    mut extra_body: Option<BytesMut>,
    buf: &mut BytesMut,
    ct_header: &str,
    ct_header_short: &str,
) -> Result<BytesMut, DEStruct>
where
    T: InfoLine,
    MessageHead<T>: ParseBodyHeaders,
{
    match decompress_body(body, extra_body.take(), encodings, buf) {
        Ok(dbody) => {
            if !one.header_map_as_mut().remove_header_on_key(ct_header) {
                one.header_map_as_mut()
                    .remove_header_on_key(ct_header_short);
            }
            Ok(dbody)
        }
        Err(e) => {
            let ce = ContentEncoding::from(&e);
            let pos = encodings.iter().position(|e| *e == ce).unwrap();
            let to_remove = &encodings[..pos];
            one.header_map_as_mut()
                .remove_applied_compression(ct_header, to_remove);
            Err(e)
        }
    }
}

pub fn decompress_body(
    mut main_body: BytesMut,
    extra_body: Option<BytesMut>,
    encodings: &[ContentEncoding],
    buf: &mut BytesMut,
) -> Result<BytesMut, DEStruct> {
    // Start
    let capacity = 2 * (main_body.len() + extra_body.as_ref().map(|b| b.len()).unwrap_or(0));
    buf.reserve(capacity);
    let mut buf_writer = buf.writer();

    // 1. concat extra and try
    if let Some(extra) = extra_body {
        let main_org_len = main_body.len();
        main_body.unsplit(extra);
        match decompress(&main_body[..], &mut buf_writer, encodings) {
            Ok(out) => return Ok(out),
            Err(e) => {
                if e.is_unknown_encoding() {
                    return Err(e);
                } else {
                    let _ = buf_writer.get_mut().try_reclaim(capacity);
                    buf_writer.get_mut().clear();
                }
            }
        }

        // 2. split extra from original
        let extra = main_body.split_off(main_org_len);
        // 3. Try main
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
    writer: &mut Writer<&mut BytesMut>,
    encodings: &[ContentEncoding],
) -> Result<BytesMut, DEStruct> {
    let mut input: &[u8] = compressed;
    let mut output = writer.get_mut().split();

    for enc in encodings.iter().rev() {
        let result = match enc {
            ContentEncoding::Brotli => decompress_brotli(input, writer),
            ContentEncoding::Gzip => decompress_gzip(input, writer),
            ContentEncoding::Deflate => decompress_deflate(input, writer),
            ContentEncoding::Identity | ContentEncoding::Chunked => continue,
            ContentEncoding::Zstd | ContentEncoding::Compress => decompress_zstd(input, writer),
            ContentEncoding::Unknown(e) => Err(DecompressError::Unknown(e.to_string())),
        };

        match result {
            Ok(_) => {
                output = writer.get_mut().split();
                input = &output[..];
            }
            Err(e) => {
                output = writer.get_mut().split();
                return Err(DEStruct::from((output, e)));
            }
        }
    }
    Ok(output)
}

#[inline]
fn decompress_brotli(
    data: &[u8],
    writer: &mut Writer<&mut BytesMut>,
) -> Result<u64, DecompressError> {
    let mut reader = brotli::Decompressor::new(data, data.len());
    copy(&mut reader, writer).map_err(DecompressError::Brotli)
}

#[inline]
fn decompress_deflate(
    data: &[u8],
    writer: &mut Writer<&mut BytesMut>,
) -> Result<u64, DecompressError> {
    let mut reader = flate2::bufread::DeflateDecoder::new(data);
    copy(&mut reader, writer).map_err(DecompressError::Deflate)
}

#[inline]
fn decompress_gzip(
    data: &[u8],
    writer: &mut Writer<&mut BytesMut>,
) -> Result<u64, DecompressError> {
    let mut reader = flate2::bufread::GzDecoder::new(data);
    copy(&mut reader, writer).map_err(DecompressError::Gzip)
}

#[inline]
fn decompress_zstd(
    data: &[u8],
    writer: &mut Writer<&mut BytesMut>,
) -> Result<u64, DecompressError> {
    let mut reader = zstd::stream::read::Decoder::new(data).map_err(DecompressError::Zstd)?;
    copy(&mut reader, writer).map_err(DecompressError::Zstd)
}

#[cfg(test)]
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
        let compressed = zstd::encode_all(&compressed[..], 1).unwrap();

        let mut result = BytesMut::new();
        let mut_result = &mut result;
        let mut writer = mut_result.writer();
        let out = decompress(&compressed[..], &mut writer, &encodings).unwrap();
        assert_eq!(&out[..], &data[..]);
    }
}
