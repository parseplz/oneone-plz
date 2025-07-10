use super::*;
use crate::response::state::content_length::compressed::compressed_data;

#[test]
fn test_response_state_chunked_single_compressed() {
    let compressed = compressed_data();
    let mut input: Vec<u8> = format!(
        "HTTP/1.1 200 OK\r\n\
        Transfer-Encoding: chunked\r\n\
        Content-Encoding: br, deflate, gzip, zstd\r\n\r\n",
    )
    .into();
    input.extend_from_slice(format!("{:x}\r\n", compressed.len()).as_bytes());
    input.extend_from_slice(&compressed);
    input.extend_from_slice(b"\r\n0\r\n\r\n");

    let verify = "HTTP/1.1 200 OK\r\n\
                  Content-Length: 11\r\n\r\n\
                  hello world";
    let result = poll_oneone_only_read::<Response>(&input);
    assert_eq!(result.into_bytes(), verify);
}
