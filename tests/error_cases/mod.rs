use super::*;
use header_plz::OneRequestLine;

pub fn poll_err(input: &[u8]) -> Error<OneRequestLine> {
    let mut buf = BytesMut::from(input);
    let mut cbuf = Cursor::new(&mut buf);
    let state: State<OneRequestLine> = State::new();
    state
        .try_next(Event::End(&mut cbuf))
        .expect_err("")
}

#[test]
fn test_error_info_line_first_ows() {
    let input = "GET/index.htmlHTTP/1.1\r\n\r\n";
    let err = poll_err(input.as_bytes());
    assert!(matches!(err, Error::InfoLine(_)));
    assert_eq!(err.into_bytes(), input.as_bytes());
}

#[test]
fn test_error_info_line_second_ows() {
    let input = "GET /index.htmlHTTP/1.1\r\n\r\n";
    let err = poll_err(input.as_bytes());
    assert!(matches!(err, Error::InfoLine(_)));
    assert_eq!(err.into_bytes(), input.as_bytes());
}

#[test]
fn test_error_unparsed() {
    let input = "helloworld";
    let err = poll_err(input.as_bytes());
    assert!(matches!(err, Error::Unparsed(_)));
    assert_eq!(err.into_bytes(), input);
}

#[test]
fn test_error_cl_partial() {
    let input = "POST /path HTTP/1.1\r\n\
                 Host: example.com\r\n\
                 Content-Length: 10\r\n\r\n\
                 hello";
    let err = poll_err(input.as_bytes());
    assert!(matches!(err, Error::ContentLengthPartial(_)));
    assert_eq!(err.into_bytes(), input.as_bytes());
}

#[test]
fn test_error_chunked_partial() {
    let input = "POST /path HTTP/1.1\r\n\
                 Host: example.com\r\n\
                 Transfer-Encoding: chunked\r\n\r\n\
                 1\r\n\
                 a\r\n";
    let err = poll_err(input.as_bytes());
    assert!(matches!(err, Error::ChunkReaderPartial(_)));
    assert_eq!(err.into_bytes(), input.as_bytes());
}
