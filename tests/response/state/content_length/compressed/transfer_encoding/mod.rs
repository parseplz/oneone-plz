use super::*;

#[test]

fn test_response_state_cl_decompress_te_complete_gzip() {
    let chunks: &[&[u8]] = &[b"HTTP/1.1 200 OK\r\nTransfer-Encoding: gzip\r\n\r\n",
    b"\x1f\x8b\x08\x00\x1f\x30\xa0\x65\x00\xff\x05\x40\xc1\x09\x00\x40\x08\x5a\xc5\xe1\xce\x28\xb0\x82",
    b"\xfb\xb5\xbd\x24\xa5\x45\x1f\xe2\x17\xe7\x19\xd3\x90\xd8\x52\x0f\x00\x00\x00"];

    let response = poll_oneone_multiple::<Response>(chunks);
    assert_eq!(response.status_code(), "200");

    let expected = "HTTP/1.1 200 OK\r\n\
                    Content-Length: 15\r\n\r\n\
                    hello my friend";
    assert_eq!(response.into_bytes(), expected);
}

#[test]
fn test_response_state_cl_decompress_ce_complete_unknown_te() {
    let input = "HTTP/1.1 200 OK\r\n\
                 Content-Length: 15\r\n\
                 Transfer-Encoding: rot13\r\n\r\n\
                 ZLRAPBQRQFGEVAT";
    let result = poll_oneone_only_read::<Response>(input.as_bytes());
    assert_eq!(result.into_bytes(), input);
}
