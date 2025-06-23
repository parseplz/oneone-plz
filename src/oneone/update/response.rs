use header_plz::Response;

use super::*;

impl UpdateHttp for OneOne<Response> {
    fn update(buf: BytesMut) -> Result<Self, UpdateFrameError> {
        OneOne::<Response>::try_from(buf)
    }
}
