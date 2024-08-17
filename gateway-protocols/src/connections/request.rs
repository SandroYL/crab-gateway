use std::ops::Deref;

use gateway_basic::util::case_sense_map::CaseSenseMap;
use gateway_basic::util::small_case_string::{SmallCaseHeader, SmallCaseString};
use http::{HeaderMap, HeaderName, HeaderValue, Uri};
use http::{request::Parts, Method};
use gateway_error::ErrorType;
use http::request::Builder as ReqBuilder;
use gateway_error::ErrTrans;
use gateway_error::Result;

type ReqParts = Parts;
type HeadersMap = CaseSenseMap;
pub struct RequestHeader {
    base: ReqParts,
    header_name_map:  Option<HeadersMap>,
}

pub enum Opt {
    INSERT,
    REMOVE,
    APPEND,
    MODIFY
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
            header_name_map: Some(CaseSenseMap::new()),
        }
    }

    pub fn build_with_method_path(
        method: impl TryInto<Method>,
        path: &[u8],
    ) -> Result<Self> {
        let mut raw_req = Self::new();
        raw_req.base.method = method.try_into()
            .to_b_err(ErrorType::InvalidHttpHeader, "invalid method")?;
        if let Ok(p) = std::str::from_utf8(path) {
            let uri = Uri::builder()
                .path_and_query(path)
                .build()
                .to_b_err(ErrorType::InvalidHttpHeader, "invalid path")?;
        }
        Ok(raw_req)
    }

    pub fn append_header(
        &mut self,
        name: impl SmallCaseString,
        value: impl TryInto<HeaderValue>
    ) -> Result<()> {
        let haeder_value = value
            .try_into()
            .to_b_err(ErrorType::InvalidHttpHeader, "invalid http head value")?;
        Self::operate_header_value(
            &mut self.base.headers,
            &mut self.header_name_map,
            vec![name],
            vec![haeder_value],
            Opt::APPEND
        )
    }

    pub fn remove_header(
        &mut self,
        name: impl SmallCaseString,
        value: impl TryInto<HeaderValue>
    ) -> Result<()> {
        let haeder_value = value
            .try_into()
            .to_b_err(ErrorType::InvalidHttpHeader, "invalid http head value")?;
        Self::operate_header_value(
            &mut self.base.headers,
            &mut self.header_name_map,
            vec![name],
            vec![haeder_value],
            Opt::REMOVE
        )
    }

    pub fn insert_header(
        &mut self,
        name: impl SmallCaseString,
        value: impl TryInto<HeaderValue>
    ) -> Result<()> {
        let haeder_value = value
            .try_into()
            .to_b_err(ErrorType::InvalidHttpHeader, "invalid http head value")?;
        Self::operate_header_value(
            &mut self.base.headers,
            &mut self.header_name_map,
            vec![name],
            vec![haeder_value],
            Opt::INSERT
        )
    }

    pub fn modify_header(
        &mut self,
        name_new: impl SmallCaseString,
        name_old: impl SmallCaseString,
        value_new: impl TryInto<HeaderValue>,
        value_old: impl TryInto<HeaderValue>
    )

    fn operate_header_value(
        value_map: &mut HeaderMap<HeaderValue>,
        header_name_map: &mut Option<HeadersMap>,
        key: Vec<impl SmallCaseString>,   //-0 add -1 modify
        mut value: Vec<HeaderValue>,
        opt: Opt
    ) -> Result<()> {
        let case_header_key = key.into_iter().map(|x| x.into_small_case_header()).collect::<Vec<SmallCaseHeader>>();
        let mut header_key: Vec<HeaderName> = case_header_key
            .iter()
            .map(|small_case_header| 
                small_case_header
                    .as_slice()
                    .try_into()
                    .to_b_err(ErrorType::InvalidHttpHeader, "invalid header name").unwrap()
            )
            .collect::<Vec<HeaderName>>();
        match opt {
            Opt::INSERT => {
                if let Some(ok_header_name_map) = header_name_map {
                    ok_header_name_map.insert(header_key[0].to_string(), case_header_key[0].to_string());
                }
                value_map.insert(header_key.pop().unwrap(), value.pop().unwrap());
            },
            Opt::APPEND => {
                if let Some(ok_header_name_map) = header_name_map {
                    ok_header_name_map.append(header_key[0].to_string(), case_header_key[0].to_string());
                }
                value_map.append(header_key.pop().unwrap(), value.pop().unwrap());
            },
            Opt::REMOVE => {
                if let Some(ok_header_name_map) = header_name_map {
                    ok_header_name_map.remove_value(header_key[0].to_string(), case_header_key[0].to_string());
                }
                value_map.remove(header_key.pop().unwrap());
            },
            Opt::MODIFY => {
                if let Some(ok_header_name_map) = header_name_map {
                    ok_header_name_map.append(header_key[0].to_string(), case_header_key[0].to_string());
                    ok_header_name_map.remove_value(header_key[1].to_string(), case_header_key[1].to_string());
                }
                header_key.pop();
                value.pop();
                value_map.insert(header_key.pop().unwrap(), value.pop().unwrap());
            }
        };
        Ok(())
    }


}