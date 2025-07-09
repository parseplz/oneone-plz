use header_plz::{Request, methods::METHODS_WITH_BODY};

use super::*;

/* Steps:
 *      1. Call update_one_one() with buf
 *      2. If method is in METHODS_WITH_BODY and no content length header is
 *         present, add Content-Length of zero.
 *
 * Note:
 *      https://github.com/curl/curl/issues/13380
 *      Adding "Content-Length: 0" is not mandatory.
 */

impl BuildFrame for OneOne<Request> {
    fn build(buf: BytesMut) -> Result<Self, UpdateFrameError> {
        let mut req = OneOne::<Request>::try_from(buf)?;
        if METHODS_WITH_BODY.contains(&req.method_as_enum()) {
            // If No content length header is present
            if req.has_header_key(CONTENT_LENGTH).is_none() {
                // Add Content-Length of zero
                req.add_header(CONTENT_LENGTH, "0");
            }
        }
        Ok(req)
    }
}
