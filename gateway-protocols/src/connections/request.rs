use http::{request::Parts, HeaderMap};


type ReqParts = Parts;
pub struct RequestHeader {
    base: ReqParts,
    header_name_map: Option<HeaderMap<CaseHeaderName>>
}