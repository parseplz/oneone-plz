use bytes::BytesMut;
use header_plz::{
    body_headers::parse::ParseBodyHeaders,
    const_headers::{CLOSE, WS_EXT},
    info_line::InfoLine,
    message_head::MessageHead,
};

use crate::{
    convert::{chunked::chunked_to_raw, convert_body, decompress::error::DecompressError},
    oneone::OneOne,
};

use super::State;

/*
impl<T> TryFrom<(State<T>, &mut BytesMut)> for OneOne<T>
where
    T: InfoLine,
    MessageHead<T>: ParseBodyHeaders,
{
    type Error = DecompressError;

    fn try_from((state, buf): (State<T>, &mut BytesMut)) -> Result<Self, Self::Error> {
        let mut one = match state {
            State::End(mut one) => {
                if one.body().is_some() {
                    convert_body(&mut one, None, buf)?;
                }
                one
            }
            State::ReadBodyContentLengthExtraEnd(mut one, extra) => {
                convert_body(&mut one, Some(extra), buf)?;
                one
            }
            State::ReadBodyChunkedExtraEnd(mut one, extra) => {
                chunked_to_raw(&mut one, buf);
                convert_body(&mut one, Some(extra), buf)?;
                one
            }
            _ => unreachable!(),
        };
        if let Some(pos) = one.has_connection_keep_alive() {
            one.header_map_as_mut()
                .change_header_value_on_pos(pos, CLOSE);
        }
        if let Some(pos) = one.has_proxy_connection() {
            one.header_map_as_mut().remove_header_on_pos(pos);
        }
        one.header_map_as_mut().remove_header_on_key(WS_EXT);
        Ok(one)
    }
}
*/

impl<T> From<(State<T>, &mut BytesMut)> for OneOne<T>
where
    T: InfoLine,
    MessageHead<T>: ParseBodyHeaders,
{
    fn from((state, buf): (State<T>, &mut BytesMut)) -> Self {
        let result = match state {
            State::End(mut one) => {
                if one.body().is_some() {
                    convert_body(one, None, buf)
                } else {
                    Ok(one)
                }
            }
            State::ReadBodyContentLengthExtraEnd(mut one, extra) => {
                convert_body(one, Some(extra), buf)
            }
            State::ReadBodyChunkedExtraEnd(mut one, extra) => {
                chunked_to_raw(&mut one, buf);
                convert_body(one, Some(extra), buf)
            }
            _ => unreachable!(),
        };
        let mut one = match result {
            Ok(one) => one,
            Err((one, e)) => {
                eprintln!("{}", e);
                one
            }
        };

        if let Some(pos) = one.has_connection_keep_alive() {
            one.header_map_as_mut()
                .change_header_value_on_pos(pos, CLOSE);
        }
        if let Some(pos) = one.has_proxy_connection() {
            one.header_map_as_mut().remove_header_on_pos(pos);
        }
        one.header_map_as_mut().remove_header_on_key(WS_EXT);
        one
    }
}
