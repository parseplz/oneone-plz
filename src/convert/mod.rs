use body_plz::variants::Body;
use bytes::BytesMut;
pub mod content_length;

pub mod chunked;
pub mod decompress;
use chunked::chunked_to_raw;
use content_length::update_content_length;
use decompress::error::DecompressError;
use decompress::*;
use header_plz::{
    InfoLine,
    body_headers::{
        BodyHeader,
        content_encoding::{CHUNKED, ContentEncoding},
        encoding_info::EncodingInfo,
        parse::ParseBodyHeaders,
    },
    message_head::MessageHead,
};

use crate::oneone::OneOne;

/* Description:
 *      Convert raw h11 to decompressed/dechunked h11.
 *
 * Steps:
 *      1. If chunked body convert chunked to CL, by calling
 *         convert_chunked() and remove Transfer-Encoding header.
 *
 *      2. If transfer encoding and content encoding is present, decompress
 *         the body by calling decompress_data() with body and vec of
 *         encodings.
 *
 *      3. Remove their corresponding headers.
 *
 *      4. Update content length header.
 */

pub fn convert_body<T>(
    one: &mut OneOne<T>,
    mut extra_body: Option<BytesMut>,
    buf: &mut BytesMut,
) -> Result<(), DecompressError>
where
    T: InfoLine,
    MessageHead<T>: ParseBodyHeaders,
{
    // 1. If chunked body convert chunked to CL
    if let Some(Body::Chunked(_)) = one.body() {
        chunked_to_raw(one, buf);
    }
    let mut body = one.get_body().into_bytes().unwrap();
    let body_headers = one.body_headers_as_mut().take();

    // 2. Transfer Encoding
    if let Some(BodyHeader {
        transfer_encoding: Some(einfo_list),
        ..
    }) = body_headers.as_ref()
    {
        // if only chunked present do nothing
        if let Some(index) = is_only_te_chunked(einfo_list) {
            one.header_map_as_mut().remove_header_on_position(index);
        } else {
            match decompress_body(one, body, extra_body.take(), einfo_list, buf) {
                Ok((result_body, result_extra_body)) => {
                    body = result_body;
                    extra_body = result_extra_body;
                }
                Err(e) => {
                    let (body, e) = e.into_body_and_error();
                    match body_headers.as_ref().unwrap().chunked_te_position() {
                        // if chunked is only value
                        Some((header_index, 0)) => one
                            .header_map_as_mut()
                            .remove_header_on_position(header_index),
                        // has other values
                        Some((header_index, value_index)) => {
                            // if last in header truncate
                            if einfo_list[header_index].encodings().len() == value_index + 1 {
                                one.header_map_as_mut()
                                    .truncate_header_value_on_position(header_index, CHUNKED);
                                // else create new string and assign
                            } else {
                                let value = einfo_list[header_index]
                                    .encodings()
                                    .iter()
                                    .filter(|e| !matches!(e, ContentEncoding::Chunked))
                                    .map(AsRef::as_ref)
                                    .collect::<Vec<_>>()
                                    .join(", ");
                                one.header_map_as_mut()
                                    .update_header_value_on_position(header_index, &value)
                            }
                        }
                        _ => (),
                    }
                    add_body_and_update_cl(one, body, body_headers);
                    return Err(e);
                }
            }
        }
    }

    // 3. Content Encoding
    if let Some(BodyHeader {
        content_encoding: Some(einfo_list),
        ..
    }) = body_headers.as_ref()
    {
        match decompress_body(one, body, extra_body.take(), einfo_list, buf) {
            Ok((result_body, result_extra_body)) => {
                body = result_body;
                extra_body = result_extra_body;
            }
            Err(e) => {
                let (body, e) = e.into_body_and_error();
                add_body_and_update_cl(one, body, body_headers);
                return Err(e);
            }
        }
    }

    if let Some(extra) = extra_body {
        body.unsplit(extra);
    }

    // 4. Update Content-Length
    add_body_and_update_cl(one, body, body_headers);
    Ok(())
}

pub fn add_body_and_update_cl<T>(
    one: &mut OneOne<T>,
    body: BytesMut,
    body_headers: Option<BodyHeader>,
) where
    T: InfoLine,
    MessageHead<T>: ParseBodyHeaders,
{
    if !body.is_empty() {
        update_content_length(one, body.len());
    }

    if let Some(bh) = body_headers {
        one.body_headers_as_mut().replace(bh);
    }
    one.set_body(Body::Raw(body));
}

pub fn is_only_te_chunked(einfo_list: &[EncodingInfo]) -> Option<usize> {
    if einfo_list.len() == 1
        && einfo_list[0].encodings().len() == 1
        && einfo_list[0].encodings()[0] == ContentEncoding::Chunked
    {
        Some(einfo_list[0].header_index)
    } else {
        None
    }
}

#[cfg(test)]
mod test {
    use crate::{error::HttpReadError, state::State};
    use buffer_plz::{Cursor, Event};
    use bytes::{BufMut, BytesMut};
    use header_plz::Response;
    use protocol_traits_plz::{Frame, Step};
    use std::io::{Read, Write};

    use flate2::{
        Compression,
        read::{DeflateEncoder, GzEncoder},
    };

    use super::*;

    #[test]
    fn test_convert_no_cl() {
        let res = "HTTP/1.1 200 OK\r\n\
                   Host: reqbin.com\r\n\
                   Content-Type: text/plain\r\n\r\n\
                   MozillaDeveloperNetwork";
        let verify = "HTTP/1.1 200 OK\r\n\
                      Host: reqbin.com\r\n\
                      Content-Type: text/plain\r\n\
                      Content-Length: 23\r\n\r\n\
                      MozillaDeveloperNetwork";

        let mut buf: BytesMut = res.into();
        let mut cbuf = Cursor::new(&mut buf);
        let mut state: State<Response> = State::new();
        let event = Event::Read(&mut cbuf);
        state = state.try_next(event).unwrap();
        let event = Event::End(&mut cbuf);
        state = state.try_next(event).unwrap();
        match state {
            State::End(_) => {
                let data = state.try_into_frame().unwrap().into_bytes();
                assert_eq!(data, verify);
            }
            _ => {
                panic!()
            }
        }
    }

    #[test]
    fn test_convert_cl_partial() {
        let res = "HTTP/1.1 200 OK\r\n\
                   Host: reqbin.com\r\n\
                   Content-Type: text/plain\r\n\
                   Content-Length: 100\r\n\r\n\
                   h";

        let mut buf: BytesMut = res.into();
        let mut cbuf = Cursor::new(&mut buf);
        let mut state: State<Response> = State::new();
        let event = Event::Read(&mut cbuf);
        state = state.try_next(event).unwrap();
        let event = Event::End(&mut cbuf);
        let result = state.try_next(event);
        if let Err(HttpReadError::ContentLengthPartial(oneone, buf)) = result {
            let data = oneone.into_bytes();
            assert_eq!(data, &res[..res.len() - 1]);
            assert_eq!(buf, "h");
        } else {
            panic!()
        }
    }

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
    fn test_decompress_all_single() {
        let compressed = compressed_data();
        let mut response: Vec<u8> = format!(
            "HTTP/1.1 200 OK\r\n\
            Host: reqbin.com\r\n\
            Content-Type: text/plain\r\n\
            Content-Length: {}\r\n\
            Content-Encoding: br, deflate, gzip, zstd\r\n\r\n",
            compressed.len()
        )
        .into();
        response.extend_from_slice(&compressed[..]);

        let mut buf = BytesMut::from(&response[..]);
        let mut cbuf = Cursor::new(&mut buf);
        let mut state: State<Response> = State::new();
        let event = Event::Read(&mut cbuf);
        state = state.try_next(event).unwrap();
        let event = Event::End(&mut cbuf);
        state = state.try_next(event).unwrap();
        let one: OneOne<Response> = state.try_into_frame().unwrap();

        let verify = "HTTP/1.1 200 OK\r\n\
            Host: reqbin.com\r\n\
            Content-Type: text/plain\r\n\
            Content-Length: 11\r\n\r\n\
            hello world";

        assert_eq!(one.into_bytes(), verify);
    }

    #[test]
    fn test_decompress_all_multiple() {
        let compressed = compressed_data();
        let mut response: Vec<u8> = format!(
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
        response.extend_from_slice(&compressed[..]);
        //parse_full_single(&response);
    }
}
