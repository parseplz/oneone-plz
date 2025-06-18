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
        BodyHeader, content_encoding::ContentEncoding, encoding_info::EncodingInfo,
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
                    add_body_and_update_cl(one, body, body_headers);
                    // TODO: remove chunked TE
                    //if let Some((header_index, value_index)) =
                    //    body_headers.as_ref().chunked_te_position()
                    //{
                    //    todo!()
                    //}
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
    use bytes::BytesMut;
    use header_plz::Response;
    use protocol_traits_plz::{Frame, Step};

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
}
