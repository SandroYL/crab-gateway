use bytes::BufMut;
use http::HeaderMap;

pub mod row_connection;
pub mod response;
pub mod request;
pub mod stream;

pub enum Opt {
    INSERT,
    REMOVE,
    APPEND,
    MODIFY
}
#[inline]
fn header_to_h1_wire(value_map: &HeaderMap, buf: &mut impl BufMut) {
    const CRLF: &[u8; 2] = b"\r\n";
    const HEADER_KV_DELIMITER: &[u8; 2] = b": ";
    
    for (header, value) in value_map {
        let header_b = 
            header.as_str().as_bytes();
        buf.put_slice(header_b);
        buf.put_slice(HEADER_KV_DELIMITER);
        buf.put_slice(value.as_bytes());
        buf.put_slice(CRLF);
    }
}