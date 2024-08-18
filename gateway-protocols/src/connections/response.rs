use std::ops::Deref;

use bytes::BufMut;
use gateway_basic::util::small_case_string::SmallCaseString;
use gateway_error::{ErrTrans, ErrorType};
use http::StatusCode;
use http::{response::Parts, HeaderMap, HeaderName, HeaderValue, Version};
use http::response::Builder as ReqBuilder;
use gateway_error::Result;
use super::{header_to_h1_wire, Opt};


type ReqParts = Parts;
pub struct ResponseHeader {
    base: ReqParts,
    reason_phrase: Option<String>,
}



impl AsRef<ReqParts> for ResponseHeader {
    fn as_ref(&self) -> &ReqParts {
        &self.base
    }
}

impl Deref for ResponseHeader {
    type Target = ReqParts;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl Clone for ResponseHeader {
    fn clone(&self) -> Self {
        Self { 
            base: self.base.clone(),
            reason_phrase: self.reason_phrase.clone()
        }
    }
}

impl ResponseHeader {
    fn new() -> Self {
        let base = ReqBuilder::new().body(()).unwrap().into_parts().0;
        ResponseHeader {
            base,
            reason_phrase: None,
        }
    }

    pub fn build_with_method_path(
        status_code: impl TryInto<StatusCode>,
    ) -> Result<Self> {
        let mut raw_resp = Self::new();
        raw_resp.base.status = status_code
            .try_into()
            .to_b_err(ErrorType::InvalidHttpHeader, "invalid response Status")?;
        Ok(raw_resp)
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