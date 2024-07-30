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
        let mut raw_parts = ReqBuilder::new().body(())
    }

    pub fn build(
        method: impl TryInto<Method>,
        path: &[u8],
    ) -> Result<Self> {

    }
}