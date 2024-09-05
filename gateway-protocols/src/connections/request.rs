use std::ops::Deref;

use bytes::BufMut;
use gateway_basic::util::case_sense_map::CaseSenseMap;
use gateway_basic::util::small_case_string::SmallCaseString;
use http::{HeaderMap, HeaderName, HeaderValue, Uri, Version};
use http::{request::Parts, Method};
use gateway_error::ErrorType;
use http::request::Builder as ReqBuilder;
use gateway_error::ErrTrans;
use gateway_error::Result;

use super::{header_to_h1_wire, Opt};

type ReqParts = Parts;
type HeadersMap = CaseSenseMap;
pub struct RequestHeader {
    base: ReqParts,
}



impl AsRef<ReqParts> for RequestHeader {
    fn as_ref(&self) -> &ReqParts {
        &self.base
    }
}

impl Deref for RequestHeader {
    type Target = ReqParts;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl RequestHeader {
    fn new() -> Self {
        let raw_parts = ReqBuilder::new().body(()).unwrap().into_parts().0;
        Self {
            base: raw_parts,
        }
    }

    pub fn build_with_method_path(
        method: impl TryInto<Method>,
        path: &[u8],
    ) -> Result<Self> {
        let mut raw_req = Self::new();
        raw_req.base.method = method.try_into()
            .to_b_err(ErrorType::InvalidHttpHeader, "invalid method")?;
        if let Ok(_) = std::str::from_utf8(path) {
            let uri = Uri::builder()
                .path_and_query(path)
                .build()
                .to_b_err(ErrorType::InvalidHttpHeader, "invalid path")?;
                raw_req.base.uri = uri;
        }
        Ok(raw_req)
    }

    pub fn append_header(
        &mut self,
        name: impl SmallCaseString,
        value: impl TryInto<HeaderValue>
    ) -> Result<()> {
        let (header_name, header_value) = Self::handle_name_value(name, value)?;
        Self::operate_header_value(
            &mut self.base.headers,
            header_name,
            header_value,
            Opt::APPEND
        )
    }

    pub fn remove_header(
        &mut self,
        name: impl SmallCaseString,
        value: impl TryInto<HeaderValue>
    ) -> Result<()> {
        let (header_name, header_value) = Self::handle_name_value(name, value)?;
        Self::operate_header_value(
            &mut self.base.headers,
            header_name,
            header_value,
            Opt::REMOVE
        )
    }

    pub fn insert_header(
        &mut self,
        name: impl SmallCaseString,
        value: impl TryInto<HeaderValue>
    ) -> Result<()> {
        let (header_name, header_value) = Self::handle_name_value(name, value)?;
        Self::operate_header_value(
            &mut self.base.headers,
            header_name,
            header_value,
            Opt::INSERT
        )
    }

    pub fn modify_header(
        &mut self,
        name: impl SmallCaseString,
        value: impl TryInto<HeaderValue>
    ) -> Result<()> {
        let (header_name, header_value) = Self::handle_name_value(name, value)?;
        Self::operate_header_value(
            &mut self.base.headers,
            header_name,
            header_value,
            Opt::MODIFY
        )        
    }

    fn operate_header_value(
        value_map: &mut HeaderMap<HeaderValue>,
        key: HeaderName,   //-0 add -1 modify
        value: HeaderValue,
        opt: Opt
    ) -> Result<()> {
        match opt {
            Opt::INSERT => {
                value_map.insert(key, value);
            },
            Opt::APPEND => {
                value_map.append(key, value);
            },
            Opt::REMOVE => {
                value_map.remove(key);
            },
            Opt::MODIFY => {
                value_map.insert(key, value);
            }
        };
        Ok(())
    }

    fn handle_name_value(
        name: impl SmallCaseString,
        value: impl TryInto<HeaderValue>
    ) -> Result<(HeaderName, HeaderValue)> {
        let header_value = value
            .try_into()
            .to_b_err(ErrorType::InvalidHttpHeader, "invalid http head value")?;
        let header_name = name.into_small_case_header()
            .as_slice()
            .try_into()
            .to_b_err(ErrorType::InvalidHttpHeader, "invalid http head name")?;
        Ok((header_name, header_value))
    }

    pub fn set_version(&mut self, version: Version) {
        self.base.version = version;
    }

    pub fn raw_path(&self) -> &[u8] {
        self.base
            .uri
            .path_and_query()
            .as_ref()
            .unwrap()
            .as_str()
            .as_bytes()
    }

    pub fn header_to_h1_wire(&self, buf: &mut impl BufMut) {
        header_to_h1_wire(&self.base.headers, buf)
    }
}


#[cfg(test)]
mod tests {
    use crate::connections::request::RequestHeader;

    

    #[test]
    fn test_single_header() {
        let mut req = RequestHeader::build_with_method_path("GET", b"/icbc/biom").unwrap();

        req.insert_header("foo", "bar").unwrap();
        req.append_header("foo", "fkv").unwrap();
        let mut buf = vec![];
        req.header_to_h1_wire(&mut buf);
        assert_eq!(buf, b"foo: bar\r\nfoo: fkv\r\n");
    }

    #[test]
    fn test_big_small_header() {
        let mut req = RequestHeader::build_with_method_path("GET", b"/icbc/biom").unwrap();

        req.insert_header("foo", "bar").unwrap();
        req.append_header("Foo", "fkv").unwrap();
        let mut buf = vec![];
        req.header_to_h1_wire(&mut buf);
        assert_eq!(buf, b"foo: bar\r\nfoo: fkv\r\n");
    }

    #[test]
    fn test_modify_header() {
        let mut req = RequestHeader::build_with_method_path("GET", b"/icbc/biom").unwrap();

        req.insert_header("foo", "bar").unwrap();
        req.append_header("Foo", "fkv").unwrap();
        req.modify_header("FoO", "shit").unwrap();
        let mut buf = vec![];
        req.header_to_h1_wire(&mut buf);
        assert_eq!(buf, b"foo: shit\r\n");  
    }

    #[test]
    fn test_remove_header() {
        let mut req = RequestHeader::build_with_method_path("GET", b"/icbc/biom").unwrap();

        req.insert_header("foo", "bar").unwrap();
        req.append_header("Foo", "fkv").unwrap();
        req.append_header("vio", "shit").unwrap();
        req.remove_header("foo", "shit").unwrap();
        let mut buf = vec![];
        req.header_to_h1_wire(&mut buf);
        assert_eq!(buf, b"vio: shit\r\n");  
    }

    #[test]
    fn test_format_header() {
        let mut req = RequestHeader::build_with_method_path("GET", b"/icbc/biom").unwrap();

        req.insert_header("foo", "bar").unwrap();
        req.insert_header(http::header::CONTENT_TYPE, "piece").unwrap();
        req.insert_header(http::header::CONTENT_TYPE, "down").unwrap();
        let mut buf = vec![];
        req.header_to_h1_wire(&mut buf);
        assert_eq!(buf, b"foo: bar\r\ncontent-type: down\r\n");          
    }
}