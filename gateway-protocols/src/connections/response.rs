use http::{Extensions, HeaderMap, HeaderValue, Result, StatusCode, Version};



pub struct Response<T> {
    head: ResponseHeader,
    body: T,
}

pub struct ResponseHeader {
    pub status: StatusCode,
    pub version: Version,
    pub headers: HeaderMap<HeaderValue>,
    pub extensions: Extensions,
    _priv: (), //防止外界直接构建请求
}

pub struct ResponseHeaderBuilder {
    inner: Result<ResponseHeader>
}

impl ResponseHeader {
    pub fn new() -> Self {
        Self {
            status: StatusCode::default(),
            version: Version::default(),
            headers: HeaderMap::default(),
            extensions: Extensions::default(),
            _priv: (),
        }
    }
}

impl<T> Response<T> {
    #[inline]
    pub fn builder() -> ResponseHeaderBuilder {
        ResponseHeaderBuilder::new()
    }

    #[inline]
    pub fn new(body: T) -> Response<T> {
        Response {
            head: ResponseHeader::new(),
            body,
        }
    }
    ///lose response forever
    pub fn divide_response(self) -> (ResponseHeader, T) {
        (self.head, self.body)
    }

    pub fn union_response(headers: ResponseHeader, body: T) -> Self {
        Self {
            head: headers,
            body,
        }
    }

    pub fn get_status(&self) -> StatusCode {
        self.head.status
    }

    pub fn get_version(&self) -> Version {
        self.head.version
    }

    pub fn get_headers(&self) -> &HeaderMap<HeaderValue> {
        &self.head.headers
    }

    pub fn get_headers_mut(&mut self) -> &mut HeaderMap<HeaderValue> {
        &mut self.head.headers
    }
    
    pub fn get_status_mut(&mut self) -> &mut StatusCode {
        &mut self.head.status
    }

    pub fn map<F, U>(self, f: F) -> Response<U>
    where
        F: FnOnce(T) -> U  
    {
        Response {
            body: f(self.body),
            head: self.head
        }
    }
}

impl ResponseHeaderBuilder {
    pub fn new() -> Self {
        ResponseHeaderBuilder::default()
    }
}

impl Default for ResponseHeaderBuilder {
    fn default() -> Self {
        Self {
            inner: Ok(ResponseHeader::new()),
        }
    }
}