use bytes::Bytes;

use crate::{http::common::Stream, util_code::buf_ref::BufRef};


/// HTTP 1.x client Session
pub struct HttpSession {
    buf: Bytes,
    pub(crate) underlying_stream: Stream,
    raw_header: Option<BufRef>,
    preread_body: Option<BufRef>,
    body_reader: BodyReader,
    body_writer: BodyWriter,
}