use buffer_plz::{Cursor, Event};
use bytes::BytesMut;
use error::OneOneRWError;
use header_plz::{
    body_headers::parse::ParseBodyHeaders, header_struct::HeaderStruct, info_line::InfoLine,
};
use protocol_traits_plz::Step;
use tokio::io::AsyncReadExt;
mod error;

use crate::{oneone::OneOne, state::State};

pub async fn read_http<T, U>(reader: &mut T, buf: &mut BytesMut) -> Result<OneOne<U>, OneOneRWError>
where
    T: AsyncReadExt + Unpin,
    U: InfoLine,
    HeaderStruct<U>: ParseBodyHeaders,
{
    let mut frame_state = State::<U>::new();
    let mut cbuf = Cursor::new(buf);
    loop {
        let event = fill_buffer(reader, &mut cbuf)
            .await
            .map_err(OneOneRWError::Read)?;
        frame_state = frame_state.next(event)?;
        if frame_state.is_ended() {
            return Ok(frame_state.into_frame()?);
        }
    }
}

pub async fn fill_buffer<'a, 'b, T>(
    stream: &mut T,
    buf: &'a mut Cursor<'b>,
) -> Result<Event<'a, 'b>, std::io::Error>
where
    T: AsyncReadExt + Unpin,
{
    let size = stream.read_buf(buf.as_mut()).await?;
    if size == 0 {
        Ok(Event::End(buf))
    } else {
        Ok(Event::Read(buf))
    }
}
