use std::fmt::Display;

use body_plz::{
    reader::{chunked_reader::ChunkReaderState, content_length_reader::read_content_length},
    variants::Body,
};
use buffer_plz::Event;
use bytes::BytesMut;
use header_plz::{
    body_headers::{parse::ParseBodyHeaders, transfer_types::TransferType},
    message_head::{MessageHead, info_line::InfoLine},
};
use protocol_traits_plz::Step;

use crate::{
    error::HttpReadError,
    oneone::{OneOne, impl_try_from::FrameError},
};

#[cfg_attr(test, derive(PartialEq, Eq))]
#[derive(Debug)]
pub enum State<T>
where
    T: InfoLine,
{
    ReadMessageHead,
    ReadBodyContentLength(OneOne<T>, usize),
    ReadBodyContentLengthExtra(OneOne<T>),
    ReadBodyChunked(OneOne<T>, ChunkReaderState),
    ReadBodyChunkedExtra(OneOne<T>),
    ReadBodyClose(OneOne<T>),
    End(OneOne<T>),
    ReadBodyContentLengthExtraEnd(OneOne<T>, BytesMut),
    ReadBodyChunkedExtraEnd(OneOne<T>, BytesMut),
}

impl<T> State<T>
where
    T: InfoLine,
    MessageHead<T>: ParseBodyHeaders,
{
    #[allow(clippy::new_without_default)]
    pub fn new() -> State<T> {
        State::<T>::ReadMessageHead
    }

    /* Steps:
     *      1. Build OneOne from headers
     *
     *      2. Match body_headers
     *          a. None => End
     *          b. Some, match transfer_type
     *              1. ContentLength,
     *                  a. size == 0    => End
     *                  b. size != 0    => ReadBodyContentLength(size)
     *              2. Chunked          => ReadBodyChunked
     *              3. Close            => ReadBodyClose
     *              3. Unknown          => End
     *
     *      3. Default => End
     */

    #[allow(clippy::result_large_err)]
    fn build_oneone(headers: BytesMut) -> Result<Self, HttpReadError<T>> {
        let mut one = OneOne::try_from(headers)?;
        let next_state = match one.body_headers() {
            None => Self::End(one),
            Some(body_headers) => match body_headers.transfer_type {
                Some(tt) => match tt {
                    TransferType::ContentLength(size) => {
                        if size == 0 {
                            Self::End(one)
                        } else {
                            Self::ReadBodyContentLength(one, size)
                        }
                    }
                    TransferType::Chunked => {
                        one.set_body(Body::Chunked(Vec::new()));
                        Self::ReadBodyChunked(one, ChunkReaderState::ReadSize)
                    }
                    TransferType::Close => Self::ReadBodyClose(one),
                },
                None => Self::End(one),
            },
        };
        Ok(next_state)
    }
}

impl<T> Step<OneOne<T>> for State<T>
where
    T: InfoLine,
    MessageHead<T>: ParseBodyHeaders,
{
    type StateError = HttpReadError<T>;
    type FrameError = FrameError;

    fn try_next(mut self, event: Event) -> Result<Self, Self::StateError> {
        match (self, event) {
            (State::ReadMessageHead, Event::Read(buf)) => match MessageHead::is_complete(buf) {
                true => {
                    let raw_headers = buf.split_at_current_pos();
                    self = State::build_oneone(raw_headers)?;
                    if buf.len() > 0 {
                        self = self.try_next(Event::Read(buf))?;
                    }
                    Ok(self)
                }
                false => Ok(Self::ReadMessageHead),
            },
            (State::ReadMessageHead, Event::End(buf)) => {
                Err(HttpReadError::Unparsed(buf.into_inner()))?
            }
            (State::ReadBodyContentLength(mut oneone, mut size), mut event) => match event {
                Event::Read(ref mut buf) | Event::End(ref mut buf) => {
                    match read_content_length(buf, &mut size) {
                        true => {
                            oneone.set_body(Body::Raw(buf.split_at_current_pos()));
                            if buf.len() > 0 {
                                let next_state = State::ReadBodyContentLengthExtra(oneone);
                                match event {
                                    Event::Read(_) => Ok(next_state),
                                    Event::End(_) => next_state.try_next(event),
                                }
                            } else {
                                Ok(State::End(oneone))
                            }
                        }
                        false => match event {
                            Event::Read(_) => Ok(State::ReadBodyContentLength(oneone, size)),
                            Event::End(buf) => Err(HttpReadError::ContentLengthPartial(
                                oneone,
                                buf.split_at_current_pos(),
                            ))?,
                        },
                    }
                }
            },
            (State::ReadBodyContentLengthExtra(oneone), Event::Read(_)) => {
                Ok(State::ReadBodyContentLengthExtra(oneone))
            }
            (State::ReadBodyContentLengthExtra(oneone), Event::End(buf)) => {
                let extra_body = buf.into_inner();
                Ok(State::ReadBodyContentLengthExtraEnd(oneone, extra_body))
            }
            (State::ReadBodyChunked(mut oneone, mut chunk_state), Event::Read(buf)) => loop {
                match chunk_state.next(buf) {
                    Some(chunk_to_add) => {
                        oneone.body_as_mut().unwrap().push_chunk(chunk_to_add);
                        match chunk_state {
                            ChunkReaderState::LastChunk => {
                                chunk_state = if oneone.has_trailers() {
                                    ChunkReaderState::ReadTrailers
                                } else {
                                    ChunkReaderState::EndCRLF
                                };
                                continue;
                            }
                            ChunkReaderState::End => {
                                return if buf.len() > 0 {
                                    Ok(State::ReadBodyChunkedExtra(oneone))
                                } else {
                                    Ok(State::End(oneone))
                                };
                            }
                            ChunkReaderState::Failed(e) => return Err(e.into()),
                            _ => continue,
                        }
                    }
                    None => {
                        return Ok(State::ReadBodyChunked(oneone, chunk_state));
                    }
                }
            },
            (State::ReadBodyChunked(oneone, _), Event::End(buf)) => {
                Err(HttpReadError::ChunkReaderPartial(oneone, buf.into_inner()))
            }
            (State::ReadBodyChunkedExtra(oneone), Event::Read(_)) => {
                Ok(State::ReadBodyChunkedExtra(oneone))
            }
            (State::ReadBodyChunkedExtra(oneone), Event::End(buf)) => {
                let extra_body = buf.into_inner();
                Ok(State::ReadBodyChunkedExtraEnd(oneone, extra_body))
            }
            (State::ReadBodyClose(oneone), Event::Read(_)) => Ok(Self::ReadBodyClose(oneone)),
            (State::ReadBodyClose(mut oneone), Event::End(buf)) => {
                oneone.set_body(Body::Raw(buf.into_inner()));
                Ok(State::End(oneone))
            }
            (State::End(oneone), event) => {
                self = match oneone.body() {
                    None => State::ReadBodyClose(oneone),
                    Some(Body::Raw(_)) => State::ReadBodyContentLengthExtra(oneone),
                    Some(Body::Chunked(_)) => State::ReadBodyChunkedExtra(oneone),
                };
                match event {
                    Event::Read(_) => Ok(self),
                    Event::End(_) => self.try_next(event),
                }
            }
            (State::ReadBodyContentLengthExtraEnd(..), _)
            | (State::ReadBodyChunkedExtraEnd(..), _) => unreachable!("not possible"),
        }
    }

    fn is_ended(&self) -> bool {
        matches!(self, Self::End(_))
            | matches!(self, State::ReadBodyContentLengthExtraEnd(..))
            | matches!(self, State::ReadBodyChunkedExtraEnd(..))
    }

    fn try_into_frame(self) -> Result<OneOne<T>, FrameError> {
        let mut buf = BytesMut::with_capacity(65535);
        OneOne::<T>::try_from((self, &mut buf))
    }
}

impl<T> Display for State<T>
where
    T: InfoLine,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            State::ReadMessageHead => write!(f, "ReadMessageHead"),
            State::ReadBodyContentLength(_, _) => write!(f, "ReadBodyContentLength"),
            State::ReadBodyContentLengthExtra(_) => write!(f, "ReadBodyContentLengthExtra"),
            State::ReadBodyChunked(_, _) => write!(f, "ReadBodyChunked"),
            State::ReadBodyChunkedExtra(_) => write!(f, "ReadBodyChunkedExtra"),
            State::ReadBodyClose(_) => write!(f, "ReadBodyClose"),
            State::End(_) => write!(f, "End"),
            State::ReadBodyContentLengthExtraEnd(_, _) => {
                write!(f, "ReadBodyContentLengthExtraEnd")
            }
            State::ReadBodyChunkedExtraEnd(_, _) => write!(f, "ReadBodyChunkedExtraEnd"),
        }
    }
}
