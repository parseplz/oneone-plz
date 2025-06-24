use super::*;

#[test]
fn test_response_state_chunked_partial() {
    let input = "HTTP/1.1 200 OK\r\n\
                 Transfer-Encoding: chunked\r\n\r\n\
                 6\r\n\
                 hello \r\n\
                 5";

    let result = poll_state_result_with_end::<Response>(input.as_bytes());
    if let Err(e) = result {
        matches!(e, HttpReadError::ChunkReaderPartial(_, _));
        assert_eq!(BytesMut::from(input.as_bytes()), BytesMut::from(e));
    } else {
        panic!()
    }
}
