use body_plz::variants::Body;
use bytes::BytesMut;
pub mod content_length;

pub mod chunked;
pub mod decompress;
use chunked::chunked_to_raw;
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

use crate::{convert::content_length::add_body_and_update_cl, oneone::OneOne};

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
            one.remove_header_on_position(index);
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
                        Some((header_index, 0)) => one.remove_header_on_position(header_index),
                        // has other values
                        Some((header_index, value_index)) => {
                            // if last in header truncate
                            if einfo_list[header_index].encodings().len() == value_index + 1 {
                                one.truncate_header_value_on_position(header_index, CHUNKED);
                                // else create new string and assign
                            } else {
                                let value = einfo_list[header_index]
                                    .encodings()
                                    .iter()
                                    .filter(|e| !matches!(e, ContentEncoding::Chunked))
                                    .map(AsRef::as_ref)
                                    .collect::<Vec<_>>()
                                    .join(", ");
                                one.update_header_value_on_position(header_index, &value)
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
                dbg!(&result_body);
                dbg!(&result_extra_body);
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
mod tests {
    use super::*;

    #[test]
    fn test_is_only_te_chunked_single() {
        let einfo_list = vec![EncodingInfo::new(0, vec![ContentEncoding::Chunked])];
        assert_eq!(is_only_te_chunked(&einfo_list), Some(0));
    }

    #[test]
    fn test_is_only_te_chunked_multi() {
        let einfo_list = vec![
            EncodingInfo::new(0, vec![ContentEncoding::Chunked]),
            EncodingInfo::new(1, vec![ContentEncoding::Deflate]),
        ];
        assert_eq!(is_only_te_chunked(&einfo_list), None);
    }
}
