use bytes::BytesMut;
use header_plz::{
    body_headers::parse::ParseBodyHeaders,
    const_headers::{CLOSE, WS_EXT},
    message_head::{MessageHead, info_line::InfoLine},
};
use thiserror::Error;

use crate::{convert::convert_body, oneone::OneOne, state::State};

#[derive(Debug, Error)]
pub enum MessageFramingError {
    #[error("incorrect state| {0}")]
    IncorrectState(String),
}

impl<T> TryFrom<(State<T>, &mut BytesMut)> for OneOne<T>
where
    T: InfoLine,
    MessageHead<T>: ParseBodyHeaders,
{
    type Error = MessageFramingError;

    fn try_from((state, buf): (State<T>, &mut BytesMut)) -> Result<Self, Self::Error> {
        let result = match state {
            State::End(mut one) => {
                if one.body().is_some() {
                    match convert_body(&mut one, None, buf) {
                        Ok(_) => Ok(one),
                        Err(e) => Err((one, e)),
                    }
                } else {
                    Ok(one)
                }
            }
            State::ReadBodyContentLengthExtraEnd(mut one, extra)
            | State::ReadBodyChunkedExtraEnd(mut one, extra) => {
                match convert_body(&mut one, Some(extra), buf) {
                    Ok(_) => Ok(one),
                    Err(e) => Err((one, e)),
                }
            }
            _ => return Err(MessageFramingError::IncorrectState(state.to_string())),
        };

        let mut one = match result {
            Ok(one) => one,
            Err((one, e)) => {
                eprintln!("{e}");
                one
            }
        };

        if let Some(pos) = one.has_connection_keep_alive() {
            one.update_header_value_on_position(pos, CLOSE);
        }
        if let Some(pos) = one.has_proxy_connection() {
            one.remove_header_on_position(pos);
        }
        one.remove_header_on_key(WS_EXT);
        Ok(one)
    }
}
