use std::io::copy;

use bytes::{BufMut, BytesMut, buf::Writer};
use header_plz::{
    body_headers::{
        content_encoding::ContentEncoding, encoding_info::EncodingInfo,
        parse::ParseBodyHeaders,
    },
    message_head::{MessageHead, info_line::InfoLine},
};
pub mod error;
use crate::{
    convert::decompress::error::DecompressErrorStruct, oneone::OneOne,
};
use error::DecompressError;

pub fn decompress_body<T>(
    one: &mut OneOne<T>,
    mut main_body: BytesMut,
    extra_body: Option<BytesMut>,
    encodings: &[EncodingInfo],
    buf: &mut BytesMut,
) -> Result<(BytesMut, Option<BytesMut>), DecompressErrorStruct>
where
    T: InfoLine,
    MessageHead<T>: ParseBodyHeaders,
{
    // Start
    let capacity = 2
        * (main_body.len()
            + extra_body
                .as_ref()
                .map(|b| b.len())
                .unwrap_or(0));
    buf.reserve(capacity);
    let mut buf_writer = buf.writer();

    if let Some(extra) = extra_body {
        // 1. concat extra and try => single compression
        let main_org_len = main_body.len();
        main_body.unsplit(extra);
        match decompress(one, &main_body[..], &mut buf_writer, encodings) {
            Ok(out) => {
                return Ok((out, None));
            }
            Err(e) => {
                if e.is_unknown_encoding() {
                    return Err(e);
                } else {
                    let _ = buf_writer
                        .get_mut()
                        .try_reclaim(capacity);
                    buf_writer.get_mut().clear();
                }
            }
        }

        // 2. split extra from original
        let extra = main_body.split_off(main_org_len);

        // 3. Try main
        let main_decompressed = match decompress(
            one,
            &main_body[..],
            &mut buf_writer,
            encodings,
        ) {
            Ok(buf) => buf,
            Err(mut e) => {
                e.extra_body = Some(extra);
                return Err(e);
            }
        };

        // 4. extra decompressed separately
        let extra_decompressed =
            match decompress(one, &extra[..], &mut buf_writer, encodings) {
                Ok(out) => out,  // compressed separately
                Err(_) => extra, // clear text ?
            };
        Ok((main_decompressed, Some(extra_decompressed)))
    } else {
        let body =
            decompress(one, &main_body[..], &mut buf_writer, encodings)?;
        Ok((body, None))
    }
}

pub fn decompress<T>(
    one: &mut OneOne<T>,
    compressed: &[u8],
    writer: &mut Writer<&mut BytesMut>,
    encoding_info: &[EncodingInfo],
) -> Result<BytesMut, DecompressErrorStruct>
where
    T: InfoLine,
    MessageHead<T>: ParseBodyHeaders,
{
    let mut input: &[u8] = compressed;
    let mut output: BytesMut = writer.get_mut().split();

    for encoding_info in encoding_info.iter().rev() {
        for (index, encoding) in encoding_info
            .encodings()
            .iter()
            .rev()
            .enumerate()
        {
            let result = match encoding {
                ContentEncoding::Brotli => decompress_brotli(input, writer),
                ContentEncoding::Chunked => continue,
                ContentEncoding::Deflate => decompress_deflate(input, writer),
                ContentEncoding::Gzip => decompress_gzip(input, writer),
                ContentEncoding::Identity => {
                    copy(&mut input, writer).map_err(DecompressError::Identity)
                }
                ContentEncoding::Zstd | ContentEncoding::Compress => {
                    decompress_zstd(input, writer)
                }
                ContentEncoding::Unknown(e) => {
                    Err(DecompressError::Unknown(e.to_string()))
                }
            };

            match result {
                Ok(_) => {
                    output = writer.get_mut().split();
                    input = &output[..];
                }
                Err(e) => {
                    writer.get_mut().clear();
                    copy(&mut input, writer).unwrap();
                    output = writer.get_mut().split();
                    // truncate till compression in header
                    let index = encoding_info.encodings().len() - index;
                    if index > 0 {
                        if let Some(encoding) =
                            encoding_info.encodings().get(index)
                        {
                            one.truncate_header_value_on_position(
                                encoding_info.header_index,
                                encoding,
                            );
                        }
                    }
                    return Err(DecompressErrorStruct::new(output, None, e));
                }
            }
        }

        // remove the header in index
        one.remove_header_on_position(encoding_info.header_index);
    }
    Ok(output)
}

#[inline]
fn decompress_brotli(
    data: &[u8],
    writer: &mut Writer<&mut BytesMut>,
) -> Result<u64, DecompressError> {
    dbg!(String::from_utf8_lossy(data));
    let mut reader = brotli::Decompressor::new(data, data.len());
    copy(&mut reader, writer).map_err(DecompressError::Brotli)
}

#[inline]
fn decompress_deflate(
    data: &[u8],
    writer: &mut Writer<&mut BytesMut>,
) -> Result<u64, DecompressError> {
    let mut reader = flate2::bufread::ZlibDecoder::new(data);
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
    let mut reader = zstd::stream::read::Decoder::new(data)
        .map_err(DecompressError::Zstd)?;
    copy(&mut reader, writer).map_err(DecompressError::Zstd)
}
