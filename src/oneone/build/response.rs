use header_plz::Response;

use super::*;

impl BuildFrame for OneOne<Response> {
    fn build(buf: BytesMut) -> Result<Self, BuildMessageError> {
        OneOne::<Response>::try_from(buf)
    }
}
