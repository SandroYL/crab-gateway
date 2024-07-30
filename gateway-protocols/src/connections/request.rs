use std::ops::Deref;

use gateway_basic::util::case_sense_map::CaseSenseMap;
use http::{request::Parts, Method};
use gateway_error::Result as Result;
use http::request::Builder as ReqBuilder;

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

    pub fn build(
        method: impl TryInto<Method>,
        path: &[u8],
    ) -> Self {
        let mut raw_req = Self::new();
        raw_req.base.method = method.try_into()
            .expect_err()        
    }
}