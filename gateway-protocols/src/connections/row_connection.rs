use bytes::{BufMut, BytesMut};
use gateway_error::{Error as Error, ErrorType, Result};
use http::Version;
use gateway_error::error_trait::OrErr;
use crate::{connections::response::ResponseHeader, http::common::*};

use super::request::RequestHeader;

pub struct ProxyDigest {
    pub response: Box<ResponseHeader>
}

impl ProxyDigest {
    pub fn new(response: Box<ResponseHeader>) -> Self {
        ProxyDigest { response }
    }
}

#[derive(Debug)]
pub struct ConnectProxyError {
    pub response: Box<ResponseHeader>,
}

impl ConnectProxyError {
    pub fn boxed_new(response: Box<ResponseHeader>) -> Box<Self> {
        Box::new(ConnectProxyError { response })
    }
}

impl std::error::Error for ConnectProxyError {}

impl std::fmt::Display for ConnectProxyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const PROXY_STATUS: &str = "proxy-status";
        let reason = self
            .response
            .headers
            .get(PROXY_STATUS)
            .and_then(|s| s.to_str().ok())
            .unwrap_or("missing proxy-status header value");
        write!(
            f,
            "Failed to Connect Response: status {}, proxy-status {reason}",
            &self.response.status
        )
    }
}

#[inline]
fn http_req_header_to_wire_auth_form(req: &RequestHeader) -> BytesMut {
    let mut buf = BytesMut::with_capacity(512);
    let method = req.method.as_str().as_bytes();
    buf.put_slice(method);
    buf.put_u8(b' ');
    if let Some(path) = req.uri.authority() {
        buf.put_slice(path.as_str().as_bytes());
    }
    buf.put_u8(b' ');

    let version = match req.version {
        Version::HTTP_09 => "HTTP/0.9",
        Version::HTTP_10 => "HTTP/1.0",
        Version::HTTP_11 => "HTTP/1.1",
        Version::HTTP_2 => "HTTP/2.0",
        _ => "HTTP/0.9",
    };
    buf.put_slice(version.as_bytes());
    buf.put_slice(CRLF);

    let headers = &req.headers;
    for (key, value) in headers.iter() {
        buf.put_slice(key.as_ref());
        buf.put_slice(HEADER_KV_DELIMITER);
        buf.put_slice(value.as_ref());
        buf.put_slice(CRLF);
    }
    buf.put_slice(CRLF);
    buf
}

#[inline]
fn validate_connect_response(resp: Box<ResponseHeader>) -> Result<ProxyDigest> {
    if !resp.status.is_success() {
        return Error::generate_error_with_root(ErrorType::ConnectProxyError, "None 2xx code",
    ConnectProxyError::boxed_new(resp));
    }
    Ok(ProxyDigest::new(resp))
}

pub fn generate_connect_header<H, S, 'a> (
    host: &str,
    port: u16,
    headers: H,
) -> Result<Box<RequestHeader>>
where
    S: AsRef<[u8]>,
    H: Iterator<Item = (S, &'a Vec<u8>)>
{
    let authority = if host.parse::<std::net::Ipv6Addr>().is_ok() {
        format!("[{host}]:{port}")
    } else {
        format!("{host}:{port}")
    };

    let req = 
        http::request::Builder::new()
            .version(http::Version::HTTP_11)
            .method(http::method::Method::CONNECT)
            .uri(format!("http://{authority}/"))
            .header(http::header::HOST, &authority);

    let (mut req, _) = match req.body(()) {
        Ok(r) => r.into_parts(),
        Err(e) => {
            return Err(e).or_err(ErrorType::InvalidHttpHeader, "Invalid CONNECT request");
        }
    };
    

}