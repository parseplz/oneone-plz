use body_plz::variants::Body;
use bytes::BytesMut;
use decompression_plz::DecompressTrait;
use header_plz::{
    InfoLine, abnf::HEADER_DELIMITER, body_headers::parse::ParseBodyHeaders,
    const_headers::CONTENT_LENGTH, message_head::MessageHead,
};

use crate::oneone::{OneOne, build::error::BuildMessageError};

/* Description:
 *      Build oneone from BytesMut.
 *      Used when request/response is modified in interceptor. No chunked body,
 *      as chunked is converted to Content-Length by convert_one_dot_one()
 *
 * Steps:
 *      1. Find HEADER_DELIMITER (2 * CRLF) in buf.
 *      2. Split buf at index.
 *      3. Build OneOne.
 *      4. if buf !empty, i.e. body is present.
 *          a. set body.
 *          b. If content-length header is present, update content-length by calling
 *          update_content_length().
 *          c. Else add, new content-length header.
 *
 * Error:
 *      BuildFrameError::UnableToFindCRLF  [1]
 *      BuildFrameError::HttpDecodeError   [3]
 */

impl<T> TryFrom<BytesMut> for OneOne<T>
where
    T: InfoLine + std::fmt::Debug,
    MessageHead<T>: ParseBodyHeaders,
{
    type Error = BuildMessageError;

    fn try_from(mut buf: BytesMut) -> Result<Self, Self::Error> {
        let index = buf
            .windows(4)
            .position(|window| window == HEADER_DELIMITER)
            .ok_or(BuildMessageError::UnableToFindCRLF)?;
        let message_head = buf.split_to(index + HEADER_DELIMITER.len());
        let mut one: OneOne<T> =
            OneOne::try_from_message_head_buf(message_head)?;
        if !buf.is_empty() {
            let len = buf.len().to_string();
            one.set_body(Body::Raw(buf));
            if !one.update_header_value_on_key(CONTENT_LENGTH, len.as_str()) {
                one.add_header(CONTENT_LENGTH, &len);
            }
        }
        Ok(one)
    }
}
