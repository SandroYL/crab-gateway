use http::{Extensions, HeaderMap, HeaderValue, StatusCode, Version};



pub struct Response<T> {
    head: String,
    body: T,
}

pub struct ResponseHeader {
    pub status: StatusCode,
    pub version: Version,
    pub headers: HeaderMap<HeaderValue>,
    pub extensions: Extensions,
    _priv: (),
}