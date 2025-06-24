mod request;
mod response;

use buffer_plz::{Cursor, Event};
use bytes::BytesMut;
use header_plz::{InfoLine, body_headers::parse::ParseBodyHeaders, message_head::MessageHead};
use oneone_plz::error::HttpReadError;
use oneone_plz::{oneone::OneOne, state::State};
use protocol_traits_plz::Step;

pub fn poll_first<T>(buf: &mut Cursor<'_>) -> State<T>
where
    T: InfoLine + std::fmt::Debug,
    MessageHead<T>: ParseBodyHeaders,
{
    let state: State<T> = State::new();
    state.try_next(Event::Read(buf)).unwrap()
}

pub fn parse_full_single_state<T>(input: &[u8]) -> State<T>
where
    T: InfoLine + std::fmt::Debug,
    MessageHead<T>: ParseBodyHeaders,
{
    let mut buf = BytesMut::from(input);
    let mut cbuf = Cursor::new(&mut buf);
    let state = poll_first(&mut cbuf);
    assert!(matches!(state, State::End(_)));
    state
}

pub fn parse_full_single<T>(input: &[u8]) -> OneOne<T>
where
    T: InfoLine + std::fmt::Debug,
    MessageHead<T>: ParseBodyHeaders,
{
    let state = parse_full_single_state(input);
    state.try_into_frame().unwrap()
}

#[allow(clippy::result_large_err)]
pub fn poll_state<T>(input: &[u8]) -> Result<State<T>, HttpReadError<T>>
where
    T: InfoLine + std::fmt::Debug,
    MessageHead<T>: ParseBodyHeaders,
{
    let mut buf = BytesMut::from(input);
    let mut cbuf = Cursor::new(&mut buf);
    let mut state: State<T> = State::new();
    state = state.try_next(Event::Read(&mut cbuf))?;
    state.try_next(Event::End(&mut cbuf))
}

pub fn parse_full_multiple<T>(input: &[&[u8]]) -> OneOne<T>
where
    T: InfoLine + std::fmt::Debug,
    MessageHead<T>: ParseBodyHeaders,
{
    let mut buf = BytesMut::from(input[0]);
    let mut cbuf = Cursor::new(&mut buf);
    let mut state = poll_first(&mut cbuf);

    for &chunk in &input[1..] {
        cbuf.as_mut().extend_from_slice(chunk);
        state = state.try_next(Event::Read(&mut cbuf)).unwrap();
    }
    state = state.try_next(Event::End(&mut cbuf)).unwrap();
    assert!(state.is_ended());
    state.try_into_frame().unwrap()
}
