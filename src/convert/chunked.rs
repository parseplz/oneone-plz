use crate::oneone::OneOne;
use body_plz::variants::{
    Body,
    chunked::{ChunkType, total_chunk_size},
};
use bytes::BytesMut;

use header_plz::{
    body_headers::parse::ParseBodyHeaders,
    const_headers::TRAILER,
    message_head::{MessageHead, info_line::InfoLine},
};

/* Description:
 *      Convert chunked body to content length.
 *
 * Steps:
 *      1. Combine ChunkType::Chunk into one body.
 *      2. If trailer is present,
 *          a. remove trailer header
 *          b. add trailer to header_map.
 */

pub fn chunked_to_raw<T>(one: &mut OneOne<T>, buf: &mut BytesMut)
where
    T: InfoLine,
    MessageHead<T>: ParseBodyHeaders,
{
    let body = one.get_body().into_chunks();
    buf.reserve(total_chunk_size(&body));
    let mut new_body = buf.split();
    body.into_iter().for_each(|chunk| match chunk {
        // 1. Combine ChunkType::Chunk into one body.
        ChunkType::Chunk(data) => new_body.extend_from_slice(&data[..data.len() - 2]),
        // 2. If trailer is present,
        ChunkType::Trailers(trailer) => {
            // 2.a. Remove trailer header
            one.remove_header_on_key(TRAILER);
            // 2.b. Add trailer to header_map
            let trailer_header = trailer.into_header_vec();
            one.append_headers(trailer_header);
        }
        ChunkType::Extra(data) => new_body.extend_from_slice(&data),
        _ => {}
    });
    one.set_body(Body::Raw(new_body));
}

// Partial chunked body
pub fn partial_chunked_to_raw(vec_body: Vec<ChunkType>) -> Option<BytesMut> {
    let mut iter = vec_body.into_iter().map(|c| c.into_bytes());
    let mut body = iter.next()?;

    for chunk in iter {
        body.unsplit(chunk);
    }

    Some(body)
}
