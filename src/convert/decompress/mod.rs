use std::io::copy;

use bytes::{BufMut, BytesMut, buf::Writer};
use header_plz::{
    body_headers::{
        content_encoding::ContentEncoding, encoding_info::EncodingInfo, parse::ParseBodyHeaders,
    },
    message_head::{MessageHead, info_line::InfoLine},
};
pub mod error;
use crate::{convert::decompress::error::DEStruct, oneone::OneOne};
use error::DecompressError;

pub fn decompress_body<T>(
    one: &mut OneOne<T>,
    mut main_body: BytesMut,
    extra_body: Option<BytesMut>,
    encodings: &[EncodingInfo],
    buf: &mut BytesMut,
) -> Result<(BytesMut, Option<BytesMut>), DEStruct>
where
    T: InfoLine,
    MessageHead<T>: ParseBodyHeaders,
{
    // Start
    let capacity = 2 * (main_body.len() + extra_body.as_ref().map(|b| b.len()).unwrap_or(0));
    buf.reserve(capacity);
    let mut buf_writer = buf.writer();

    // 1. concat extra and try
    if let Some(extra) = extra_body {
        let main_org_len = main_body.len();
        main_body.unsplit(extra);
        match decompress(one, &main_body[..], &mut buf_writer, encodings) {
            Ok(out) => return Ok((out, None)),
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
        let main_decompressed = match decompress(one, &main_body[..], &mut buf_writer, encodings) {
            Ok(buf) => buf,
            Err(mut e) => {
                e.extra_body = Some(extra);
                return Err(e);
            }
        };

        // 4. extra decompressed separately
        let extra_decompressed = match decompress(one, &extra[..], &mut buf_writer, encodings) {
            Ok(out) => out,  // compressed separately
            Err(_) => extra, // clear text ?
        };
        Ok((main_decompressed, Some(extra_decompressed)))
    } else {
        let body = decompress(one, &main_body[..], &mut buf_writer, encodings)?;
        Ok((body, None))
    }
}

pub fn decompress<T>(
    one: &mut OneOne<T>,
    compressed: &[u8],
    writer: &mut Writer<&mut BytesMut>,
    encoding_info: &[EncodingInfo],
) -> Result<BytesMut, DEStruct>
where
    T: InfoLine,
    MessageHead<T>: ParseBodyHeaders,
{
    let mut input: &[u8] = compressed;
    let mut output = writer.get_mut().split();

    for einfo in encoding_info.iter().rev() {
        for (index, encoding) in einfo.encodings().iter().rev().enumerate() {
            let result = match encoding {
                ContentEncoding::Brotli => decompress_brotli(input, writer),
                ContentEncoding::Gzip => decompress_gzip(input, writer),
                ContentEncoding::Deflate => decompress_deflate(input, writer),
                ContentEncoding::Identity | ContentEncoding::Chunked => continue,
                ContentEncoding::Zstd | ContentEncoding::Compress => decompress_zstd(input, writer),
                ContentEncoding::Unknown(e) => Err(DecompressError::Unknown(e.to_string())),
            };

            output = writer.get_mut().split();
            match result {
                Ok(_) => {
                    input = &output[..];
                }
                Err(e) => {
                    // truncate till compression in header
                    let index = einfo.encodings().len() - index;
                    if index > 0 {
                        if let Some(encoding) = einfo.encodings().get(index) {
                            one.header_map_as_mut()
                                .truncate_header_value_on_position(einfo.header_index, encoding);
                        }
                    }
                    return Err(DEStruct::new(output, None, e));
                }
            }
        }
        // remove the header in index
        one.header_map_as_mut()
            .remove_header_on_position(einfo.header_index);
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
        let writer = mut_result.writer();

        let encodings = [
            ContentEncoding::Brotli,
            ContentEncoding::Deflate,
            ContentEncoding::Gzip,
            ContentEncoding::Zstd,
        ];
        //let out = decompress(&compressed[..], &mut writer, &encodings).unwrap();
        //assert_eq!(&out[..], &data[..]);
    }
}
