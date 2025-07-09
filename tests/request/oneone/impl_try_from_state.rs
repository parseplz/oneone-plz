use body_plz::reader::chunked_reader::ChunkReaderState;
use oneone_plz::oneone::impl_try_from_state::MessageFramingError;

use super::*;

#[test]
fn test_request_try_from_state_incorrect_state() {
    let input = "GET / HTTP/1.1\r\n\r\n";
    //
    let state: State<Request> = State::ReadMessageHead;
    assert!(matches!(
        state.try_into_frame(),
        Err(MessageFramingError::IncorrectState(_))
    ));

    //
    let request = OneOne::<Request>::try_from(BytesMut::from(input)).unwrap();
    let state: State<Request> = State::ReadBodyContentLength(request, 10);
    assert!(matches!(
        state.try_into_frame(),
        Err(MessageFramingError::IncorrectState(_))
    ));

    //
    let request = OneOne::<Request>::try_from(BytesMut::from(input)).unwrap();
    let state: State<Request> = State::ReadBodyContentLengthExtra(request);
    assert!(matches!(
        state.try_into_frame(),
        Err(MessageFramingError::IncorrectState(_))
    ));

    //
    let request = OneOne::<Request>::try_from(BytesMut::from(input)).unwrap();
    let state: State<Request> = State::ReadBodyChunked(request, ChunkReaderState::ReadSize);
    assert!(matches!(
        state.try_into_frame(),
        Err(MessageFramingError::IncorrectState(_))
    ));

    //
    let request = OneOne::<Request>::try_from(BytesMut::from(input)).unwrap();
    let state: State<Request> = State::ReadBodyChunkedExtra(request);
    assert!(matches!(
        state.try_into_frame(),
        Err(MessageFramingError::IncorrectState(_))
    ));

    //
    let request = OneOne::<Request>::try_from(BytesMut::from(input)).unwrap();
    let state: State<Request> = State::ReadBodyClose(request);
    assert!(matches!(
        state.try_into_frame(),
        Err(MessageFramingError::IncorrectState(_))
    ));
}

#[test]
fn test_request_try_from_state_remove_proxy_header() {
    let input = "GET / HTTP/1.1\r\n\
                 Host: example.com\r\n\
                 Proxy-Connection: keep-alive\r\n\r\n";
    let result = poll_oneone_only_read::<Request>(input.as_bytes());
    let verify = "GET / HTTP/1.1\r\n\
                   Host: example.com\r\n\r\n";
    assert_eq!(result.into_bytes(), verify);
}

#[test]
fn test_request_try_from_state_modify_connection_header() {
    let input = "GET / HTTP/1.1\r\n\
                 Host: example.com\r\n\
                 Connection: keep-alive\r\n\r\n";
    let result = poll_oneone_only_read::<Request>(input.as_bytes());
    let verify = "GET / HTTP/1.1\r\n\
                   Host: example.com\r\n\
                   Connection: close\r\n\r\n";
    assert_eq!(result.into_bytes(), verify);
}
