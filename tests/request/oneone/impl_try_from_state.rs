use body_plz::reader::chunked_reader::ChunkReaderState;
use oneone_plz::error::IncorrectState;

use super::*;

#[test]
fn test_request_try_from_state_incorrect_state() {
    let input = "GET / HTTP/1.1\r\n\r\n";
    let state: State<OneRequestLine> = State::ReadMessageHead;
    let _err = IncorrectState::default();
    assert!(matches!(state.try_into_frame(), Err(_err)));

    let request =
        OneOne::<OneRequestLine>::try_from(BytesMut::from(input)).unwrap();
    let state: State<OneRequestLine> =
        State::ReadBodyContentLength(request, 10);
    assert!(matches!(state.try_into_frame(), Err(_err)));

    let request =
        OneOne::<OneRequestLine>::try_from(BytesMut::from(input)).unwrap();
    let state: State<OneRequestLine> =
        State::ReadBodyContentLengthExtra(request);
    assert!(matches!(state.try_into_frame(), Err(_err)));

    let request =
        OneOne::<OneRequestLine>::try_from(BytesMut::from(input)).unwrap();
    let state: State<OneRequestLine> =
        State::ReadBodyChunked(request, ChunkReaderState::ReadSize);
    assert!(matches!(state.try_into_frame(), Err(_err)));

    let request =
        OneOne::<OneRequestLine>::try_from(BytesMut::from(input)).unwrap();
    let state: State<OneRequestLine> = State::ReadBodyChunkedExtra(request);
    assert!(matches!(state.try_into_frame(), Err(_err)));

    let request =
        OneOne::<OneRequestLine>::try_from(BytesMut::from(input)).unwrap();
    let state: State<OneRequestLine> = State::ReadBodyClose(request);
    assert!(matches!(state.try_into_frame(), Err(_err)));
}

#[test]
fn test_request_try_from_state_full() {
    let input = "GET / HTTP/1.1\r\n\
                 Host: example.com\r\n\
                 Proxy-Connection: keep-alive\r\n\r\n";
    let result = poll_oneone_only_read::<OneRequestLine>(input.as_bytes());
    let verify = "GET / HTTP/1.1\r\n\
                  Host: example.com\r\n\
                  Proxy-Connection: keep-alive\r\n\r\n";
    assert_eq!(result.into_bytes(), verify);
}

#[test]
fn test_request_try_from_state_full_2() {
    let input = "GET / HTTP/1.1\r\n\
                 Host: example.com\r\n\
                 Connection: keep-alive\r\n\r\n";
    let result = poll_oneone_only_read::<OneRequestLine>(input.as_bytes());
    let verify = "GET / HTTP/1.1\r\n\
                  Host: example.com\r\n\
                  Connection: keep-alive\r\n\r\n";
    assert_eq!(result.into_bytes(), verify);
}
