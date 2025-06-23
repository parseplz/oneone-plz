use bytes::BytesMut;
use header_plz::const_headers::CONTENT_LENGTH;

use super::*;
pub mod error;
use error::*;
mod request;
mod response;

pub trait UpdateHttp {
    fn update(buf: BytesMut) -> Result<Self, UpdateFrameError>
    where
        Self: Sized;
}

/* Description:
 *      Update oneone from BytesMut.
 *      Used when request/response is modified in interceptor. No chunked body,
 *      as chunked is converted to Content-Length by convert_one_dot_one()
 *
 * Steps:
 *      1. Find HEADER_DELIMITER (2 * CRLF) in buf.
 *      2. Split buf at index.
 *      3. Build OneOne.
 *      4. if buf !empty, i.e. body is present.
 *          a. set body.
 *          b. If CL header is present, update Content-Length by calling
 *         update_content_length()
 *          c. Else add, new CL header.
 *
 * Error:
 *      UpdateFrameError::UnableToFindCRLF  [1]
 *      UpdateFrameError::HttpDecodeError   [3]
 */

/*
pub fn update_one_one<T>(mut buf: BytesMut) -> Result<OneOne<T>, UpdateFrameError>
where
    T: InfoLine,
    MessageHead<T>: ParseBodyHeaders,
{
    // 1. Find HEADER_DELIMITER (2 * CRLF) in buf.
    let index = buf
        .windows(4)
        .position(|window| window == HEADER_DELIMITER)
        .ok_or(UpdateFrameError::UnableToFindCRLF)?;
    let message_head = buf.split_to(index + HEADER_DELIMITER.len());
    let mut one: OneOne<T> = OneOne::try_from_message_head_raw(message_head)?;
    // 4. Body is present
    if !buf.is_empty() {
        let len = buf.len().to_string();
        // 4.a. set body
        one.set_body(Body::Raw(buf));
        if !one
            .header_map_as_mut()
            .update_header_value_on_key(CONTENT_LENGTH, len.as_str())
        {
            one.add_header(CONTENT_LENGTH, &len);
        }
    }
    Ok(one)
}
*/
