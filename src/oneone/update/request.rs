use header_plz::{info_line::request::Request, methods::METHODS_WITH_BODY};

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

impl UpdateHttp for OneOne<Request> {
    fn update(buf: BytesMut) -> Result<Self, UpdateFrameError> {
        let mut req = update_one_one::<Request>(buf)?;
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn update_request_post_no_body() {
        let buf = BytesMut::from("POST / HTTP/1.1\r\n\r\n");
        let req = OneOne::<Request>::update(buf).unwrap();
        let verify = "POST / HTTP/1.1\r\nContent-Length: 0\r\n\r\n";
        assert_eq!(req.into_bytes(), verify);
    }

    #[test]
    fn update_request_post_with_body() {
        let buf = BytesMut::from("POST / HTTP/1.1\r\nContent-Length: 10\r\n\r\na");
        let req = OneOne::<Request>::update(buf).unwrap();
        let verify = "POST / HTTP/1.1\r\nContent-Length: 1\r\n\r\na";
        assert_eq!(req.into_bytes(), verify);
    }

    #[test]
    fn update_request_no_cl() {
        let buf = BytesMut::from("POST / HTTP/1.1\r\n\r\n");
        let req = OneOne::<Request>::update(buf).unwrap();
        let verify = "POST / HTTP/1.1\r\nContent-Length: 0\r\n\r\n";
        assert_eq!(req.into_bytes(), verify);
    }
}
