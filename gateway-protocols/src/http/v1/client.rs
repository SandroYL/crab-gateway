use std::time::Duration;

use bytes::Bytes;
use http::HeaderMap;

use crate::{connections::{digest::Digest, request::RequestHeader, response::ResponseHeader}, http::common::{is_upgrade_req, KeepaliveStatus, Stream}, util_code::buf_ref::BufRef};

use super::body::{BodyReader, BodyWriter};


/// HTTP 1.x client Session
pub struct HttpSession {
    buf: Bytes,
    pub(crate) underlying_stream: Stream,
    raw_header: Option<BufRef>,
    preread_body: Option<BufRef>,
    body_reader: BodyReader,
    body_writer: BodyWriter,
    pub read_timeout: Option<Duration>,
    pub write_timeout: Option<Duration>,
    keepalive_timeout: KeepaliveStatus,
    pub(crate) digest: Box<Digest>,
    response_header: Option<Box<ResponseHeader>>,
    request_header: Option<Box<RequestHeader>>,
    bytes_sent: usize,
    upgraded: bool,
}

impl HttpSession {
    pub fn new(stream: Stream) -> Self {
        let digest = Box::new(Digest {
            timing_digest: stream.get_timing_digest(),
            proxy_digest: stream.get_proxy_digest(),
        });
        HttpSession {
            underlying_stream: stream,
            buf: Bytes::new(),
            raw_header: None,
            preread_body: None,
            body_reader: BodyReader::new(),
            body_writer: BodyWriter::new(),
            read_timeout: None,
            write_timeout: None,
            keepalive_timeout: KeepaliveStatus::Off,
            response_header: None,
            request_header: None,
            digest,
            bytes_sent: 0,
            upgraded: false,
        }
    }

    pub async fn write_request_header(&mut self, req: Box<RequestHeader>) -> Result<usize> {
        self.init
    }

    fn init_req_body_writer(&mut self, header: &RequestHeader) {
        if is_upgrade_req(header) {
            self.body_writer.init_http10();
        } else {
            self.init_body_writer_comm(&header.headers);
        }
    }

    fn init_body_writer_comm(&mut self, headers: &HeaderMap) {
        let te_value = headers.get(http::header::TRANSFER_ENCODING);
        
    }
}