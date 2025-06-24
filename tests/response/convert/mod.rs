use super::*;

mod content_length;
mod decompress;

#[test]
fn test_response_convert_no_cl() {
    let input = "HTTP/1.1 200 OK\r\n\
               Host: reqbin.com\r\n\
               Content-Type: text/plain\r\n\r\n\
               MozillaDeveloperNetwork";
    let verify = "HTTP/1.1 200 OK\r\n\
                  Host: reqbin.com\r\n\
                  Content-Type: text/plain\r\n\
                  Content-Length: 23\r\n\r\n\
                  MozillaDeveloperNetwork";

    let mut buf: BytesMut = input.into();
    let mut cbuf = Cursor::new(&mut buf);
    let mut state = poll_first::<Response>(&mut cbuf);
    let event = Event::End(&mut cbuf);
    state = state.try_next(event).unwrap();
    match state {
        State::End(_) => {
            let data = state.try_into_frame().unwrap().into_bytes();
            assert_eq!(data, verify);
        }
        _ => {
            panic!()
        }
    }
}
