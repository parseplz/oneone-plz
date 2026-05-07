use decompression_plz::DecompressTrait;
use std::fmt::Display;

use body_plz::{
    reader::{
        chunked_reader::ChunkReaderState,
        content_length_reader::read_content_length,
    },
    variants::Body,
};
use buffer_plz::{Cursor, Event};
use bytes::BytesMut;
use header_plz::{
    OneHeader, OneInfoLine,
    body_headers::{parse::ParseBodyHeaders, transfer_types::TransferType},
    message_head::MessageHead,
};
use http_plz::OneOne as Msg;

use crate::error::{Error, IncorrectState};

#[derive(Debug, PartialEq)]
pub enum State<T>
where
    T: OneInfoLine + std::fmt::Debug,
{
    ReadMessageHead,
    ReadBodyContentLength(Msg<T>, usize),
    ReadBodyContentLengthExtra(Msg<T>),
    ReadBodyChunked(Msg<T>, ChunkReaderState),
    ReadBodyChunkedExtra(Msg<T>),
    ReadBodyClose(Msg<T>),
    End(Msg<T>),
    ReadBodyContentLengthExtraEnd(Msg<T>, BytesMut),
    ReadBodyChunkedExtraEnd(Msg<T>, BytesMut),
}

impl<T> State<T>
where
    T: OneInfoLine + std::fmt::Debug,
    MessageHead<T, OneHeader>: ParseBodyHeaders,
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

    fn try_build_msg(
        headers: BytesMut,
        buf: &mut Cursor,
    ) -> Result<Self, Error<T>> {
        let mut msg = Msg::try_from_message_head_buf(headers)?;
        let next_state = match msg.body_headers() {
            None => Self::End(msg),
            Some(body_headers) => match body_headers.transfer_type {
                Some(tt) => match tt {
                    TransferType::ContentLength(0) => Self::End(msg),
                    TransferType::ContentLength(size) => {
                        buf.as_mut().reserve(size);
                        Self::ReadBodyContentLength(msg, size)
                    }
                    TransferType::Chunked => {
                        msg.set_body(Body::Chunked(Vec::new()));
                        Self::ReadBodyChunked(msg, ChunkReaderState::ReadSize)
                    }
                    TransferType::Close => Self::ReadBodyClose(msg),
                },
                None => Self::End(msg),
            },
        };
        Ok(next_state)
    }
}

impl<T> State<T>
where
    T: OneInfoLine + std::fmt::Debug,
    MessageHead<T, OneHeader>: ParseBodyHeaders,
{
    pub fn try_next(mut self, event: Event) -> Result<Self, Error<T>> {
        use State::*;
        match (self, event) {
            (ReadMessageHead, ref mut event) => {
                let buf = event.inner_mut();
                if !MessageHead::is_complete(buf) {
                    return match event {
                        Event::Read(_) => Ok(ReadMessageHead),
                        Event::End(buf) => {
                            Err(Error::Unparsed(buf.into_inner()))?
                        }
                    };
                }
                let raw_headers = buf.split_at_current_pos();
                self = State::try_build_msg(raw_headers, buf)?;
                if buf.is_empty() && matches!(event, Event::Read(_)) {
                    return Ok(self);
                }
                let event = match event {
                    Event::Read(buf) => Event::Read(buf),
                    Event::End(buf) => Event::End(buf),
                };
                self.try_next(event)
            }
            (ReadBodyContentLength(mut msg, mut size), mut event) => {
                let buf = event.inner_mut();
                if !read_content_length(buf, &mut size) {
                    return match event {
                        Event::Read(_) => Ok(ReadBodyContentLength(msg, size)),
                        Event::End(buf) => Err(Error::ContentLengthPartial(
                            (msg, buf.split_at_current_pos()).into(),
                        )),
                    };
                }
                msg.set_body(Body::Raw(buf.split_at_current_pos()));
                if buf.is_empty() {
                    return Ok(End(msg));
                }
                let next_state = ReadBodyContentLengthExtra(msg);
                match event {
                    Event::Read(_) => Ok(next_state),
                    Event::End(_) => next_state.try_next(event),
                }
            }
            (ReadBodyContentLengthExtra(msg), Event::Read(_)) => {
                Ok(ReadBodyContentLengthExtra(msg))
            }
            (ReadBodyContentLengthExtra(msg), Event::End(buf)) => {
                let extra_body = buf.into_inner();
                Ok(ReadBodyContentLengthExtraEnd(msg, extra_body))
            }
            (ReadBodyChunked(mut msg, mut chunk_state), Event::Read(buf)) => {
                loop {
                    let Some(chunk_to_add) = chunk_state.next(buf) else {
                        return Ok(ReadBodyChunked(msg, chunk_state));
                    };
                    msg.body_as_mut()
                        .expect("no chunked body")
                        .push_chunk(chunk_to_add);
                    match chunk_state {
                        ChunkReaderState::LastChunk => {
                            chunk_state = if msg.has_trailers() {
                                ChunkReaderState::ReadTrailers
                            } else {
                                ChunkReaderState::EndCRLF
                            };
                            continue;
                        }
                        ChunkReaderState::End => {
                            return if !buf.is_empty() {
                                Ok(State::ReadBodyChunkedExtra(msg))
                            } else {
                                Ok(End(msg))
                            };
                        }
                        ChunkReaderState::Failed(e) => {
                            return Err(Error::ChunkState(e, Box::new(msg)));
                        }
                        _ => {}
                    }
                }
            }
            (ReadBodyChunked(msg, _), Event::End(buf)) => {
                Err(Error::ChunkReaderPartial((msg, buf.into_inner()).into()))
            }
            (ReadBodyChunkedExtra(msg), Event::Read(_)) => {
                Ok(ReadBodyChunkedExtra(msg))
            }
            (ReadBodyChunkedExtra(msg), Event::End(buf)) => {
                let extra_body = buf.into_inner();
                Ok(ReadBodyChunkedExtraEnd(msg, extra_body))
            }
            (ReadBodyClose(msg), Event::Read(_)) => {
                Ok(Self::ReadBodyClose(msg))
            }
            (ReadBodyClose(mut msg), Event::End(buf)) => {
                msg.set_body(Body::Raw(buf.into_inner()));
                Ok(End(msg))
            }
            (End(mut msg), event) => {
                self = match msg.body() {
                    None => {
                        msg.set_transfer_type_close();
                        ReadBodyClose(msg)
                    }
                    Some(Body::Raw(_)) => ReadBodyContentLengthExtra(msg),
                    Some(Body::Chunked(_)) => ReadBodyChunkedExtra(msg),
                };
                match event {
                    Event::Read(_) => Ok(self),
                    Event::End(_) => self.try_next(event),
                }
            }
            (ReadBodyContentLengthExtraEnd(..), _)
            | (ReadBodyChunkedExtraEnd(..), _) => {
                unreachable!("not possible")
            }
        }
    }

    pub fn is_ended(&self) -> bool {
        matches!(self, Self::End(_))
            | matches!(self, State::ReadBodyContentLengthExtraEnd(..))
            | matches!(self, State::ReadBodyChunkedExtraEnd(..))
    }

    pub fn try_into_frame(self) -> Result<Msg<T>, IncorrectState> {
        let msg = match self {
            State::End(msg) => msg,
            State::ReadBodyContentLengthExtraEnd(mut msg, extra)
            | State::ReadBodyChunkedExtraEnd(mut msg, extra) => {
                msg.set_extra_body(extra);
                msg
            }
            _ => {
                return Err(IncorrectState(self.to_string()));
            }
        };
        Ok(msg)
    }
}

impl<T> Display for State<T>
where
    T: OneInfoLine + std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            State::ReadMessageHead => write!(f, "ReadMessageHead"),
            State::ReadBodyContentLength(_, _) => {
                write!(f, "ReadBodyContentLength")
            }
            State::ReadBodyContentLengthExtra(_) => {
                write!(f, "ReadBodyContentLengthExtra")
            }
            State::ReadBodyChunked(_, _) => write!(f, "ReadBodyChunked"),
            State::ReadBodyChunkedExtra(_) => {
                write!(f, "ReadBodyChunkedExtra")
            }
            State::ReadBodyClose(_) => write!(f, "ReadBodyClose"),
            State::End(_) => write!(f, "End"),
            State::ReadBodyContentLengthExtraEnd(_, _) => {
                write!(f, "ReadBodyContentLengthExtraEnd")
            }
            State::ReadBodyChunkedExtraEnd(_, _) => {
                write!(f, "ReadBodyChunkedExtraEnd")
            }
        }
    }
}
