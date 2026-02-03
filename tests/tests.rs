#![allow(warnings, clippy::unwrap_used)]
mod request;
mod response;

use buffer_plz::{Cursor, Event};
use bytes::BytesMut;
use header_plz::{OneHeader, OneMessageHead};
use header_plz::{
    OneInfoLine, body_headers::parse::ParseBodyHeaders,
    message_head::MessageHead,
};
use header_plz::{OneRequestLine, OneResponseLine};
use http_plz::OneOne;
use oneone_plz::error::HttpStateError;
use oneone_plz::state::State;

pub fn poll_state_once<T>(input: &[u8]) -> (BytesMut, State<T>)
where
    T: OneInfoLine + std::fmt::Debug,
    OneMessageHead<T>: ParseBodyHeaders,
{
    let mut buf = BytesMut::from(input);
    let mut cbuf = Cursor::new(&mut buf);
    let mut state: State<T> = State::new();
    state = state
        .try_next(Event::Read(&mut cbuf))
        .unwrap();
    (buf, state)
}

#[allow(clippy::result_large_err)]
pub fn poll_state_result_with_end<T>(
    input: &[u8],
) -> Result<State<T>, HttpStateError<T>>
where
    T: OneInfoLine + std::fmt::Debug,
    OneMessageHead<T>: ParseBodyHeaders,
{
    let (mut buf, state) = poll_state_once(input);
    let mut cbuf = Cursor::new(&mut buf);
    state.try_next(Event::End(&mut cbuf))
}

pub fn poll_oneone_only_read<T>(input: &[u8]) -> OneOne<T>
where
    T: OneInfoLine + std::fmt::Debug,
    OneMessageHead<T>: ParseBodyHeaders,
{
    let (_, state) = poll_state_once(input);
    assert!(matches!(state, State::End(_)));
    state.try_into_frame().unwrap()
}

pub fn poll_oneone_multiple<T>(input: &[&[u8]]) -> OneOne<T>
where
    T: OneInfoLine + std::fmt::Debug,
    OneMessageHead<T>: ParseBodyHeaders,
{
    let (mut buf, mut state) = poll_state_once(input[0]);
    let mut cbuf = Cursor::new(&mut buf);

    for &chunk in &input[1..] {
        cbuf.as_mut().extend_from_slice(chunk);
        state = state
            .try_next(Event::Read(&mut cbuf))
            .unwrap();
    }
    state = state
        .try_next(Event::End(&mut cbuf))
        .unwrap();
    assert!(state.is_ended());
    state.try_into_frame().unwrap()
}
