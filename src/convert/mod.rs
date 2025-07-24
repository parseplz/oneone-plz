use body_plz::variants::Body;
use bytes::BytesMut;

pub mod chunked;
use chunked::chunked_to_raw;
use decompression_plz::{DecompressTrait, decompress};
use header_plz::{
    InfoLine,
    body_headers::{
        content_encoding::ContentEncoding, encoding_info::EncodingInfo,
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
    extra_body: Option<BytesMut>,
    buf: &mut BytesMut,
) -> Result<(), std::io::Error>
where
    T: InfoLine,
    MessageHead<T>: ParseBodyHeaders,
{
    // 1. If chunked body convert chunked to CL
    if let Body::Chunked(_) = one.get_body() {
        chunked_to_raw(one, buf);
    }
    decompress(one, extra_body, buf);
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
        let einfo_list =
            vec![EncodingInfo::new(0, vec![ContentEncoding::Chunked])];
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
