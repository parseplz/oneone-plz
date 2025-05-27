use buffer_plz::{Cursor, Event};
use bytes::BytesMut;
use header_plz::{
    body_headers::parse::ParseBodyHeaders, info_line::InfoLine, message_head::MessageHead,
};
use oneone_plz::{oneone::OneOne, state::State};
use protocol_traits_plz::Step;

mod response;

pub fn parse_full_single<T>(input: &[u8]) -> OneOne<T>
where
    T: InfoLine + std::fmt::Debug,
    MessageHead<T>: ParseBodyHeaders,
{
    let mut buf = BytesMut::from(input);
    let mut cbuf = Cursor::new(&mut buf);
    let state = poll_first(&mut cbuf);
    assert!(matches!(state, State::End(_)));
    state.into_frame().unwrap()
}

pub fn poll_first<T>(buf: &mut Cursor<'_>) -> State<T>
where
    T: InfoLine + std::fmt::Debug,
    MessageHead<T>: ParseBodyHeaders,
{
    let state: State<T> = State::new();
    state.next(Event::Read(buf)).unwrap()
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
        state = state.next(Event::Read(&mut cbuf)).unwrap();
    }
    state = state.next(Event::End(&mut cbuf)).unwrap();
    assert!(matches!(state, State::End(_)));
    state.into_frame().unwrap()
}
