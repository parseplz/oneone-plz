use header_plz::info_line::response::Response;

use super::*;

impl UpdateHttp for OneOne<Response> {
    fn update(buf: BytesMut) -> Result<Self, UpdateFrameError> {
        update_one_one::<Response>(buf)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn update_response() {
        let buf = BytesMut::from("HTTP/1.1 200 OK\r\n\r\nhello");
        let req = OneOne::<Response>::update(buf).unwrap();
        let verify = "HTTP/1.1 200 OK\r\nContent-Length: 5\r\n\r\nhello";
        assert_eq!(req.into_data(), verify);
    }
}
