use std::ops::Deref;

use gateway_basic::util::case_sense_map::CaseSenseMap;
use gateway_basic::util::small_case_string::SmallCaseString;
use http::Uri;
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
        value: impl SmallCaseString
    ) {

    }


}