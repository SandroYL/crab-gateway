
use bytes::{BufMut, Bytes, BytesMut};
use gateway_error::{Error as Error, ErrorType, Result};
use http::{HeaderName, HeaderValue, Version};
use gateway_error::error_trait::OrErr;
use crate::{connections::response::ResponseHeader, http::common::*, util_code::util_code::get_version_str};

use http::request::Parts as ReqHeader;

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
fn validate_connect_response(resp: Box<ResponseHeader>) -> Result<ProxyDigest> {
    if !resp.status.is_success() {
        return Error::generate_error_with_root(ErrorType::ConnectProxyError, "None 2xx code",
    ConnectProxyError::boxed_new(resp));
    }
    Ok(ProxyDigest::new(resp))
}

pub fn generate_connect_header<'a, H, S> (
    host: &str,
    port: u16,
    headers: H,
) -> Result<Box<ReqHeader>>
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
    
    for (k, v) in headers {
        let header_name = HeaderName::from_bytes(k.as_ref())
            .or_err(ErrorType::InvalidHttpHeader, "Invalid connect request")?;
        let header_value = HeaderValue::from_bytes(v.as_slice())
            .or_err(ErrorType::InvalidHttpHeader, "Invalid connect request")?;
        req.headers.insert(header_name, header_value);
    }
    Ok(Box::new(req))
}

#[inline]
fn from_request_head_to_bytes (req: &RequestHeader) -> BytesMut {
    let mut buf = BytesMut::with_capacity(512);
    let method = req.method.as_str().as_bytes();
    buf.put_slice(method);
    buf.put_u8(b' ');
    if let Some(path) = req.uri.authority() {
        buf.put_slice(path.as_str().as_bytes());
    }
    buf.put_u8(b' ');

    buf.put_slice(get_version_str(&req.version).as_bytes());
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