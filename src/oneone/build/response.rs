use header_plz::Response;

use super::*;

impl BuildFrame for OneOne<Response> {
    fn build(buf: BytesMut) -> Result<Self, BuildFrameError> {
        OneOne::<Response>::try_from(buf)
    }
}
