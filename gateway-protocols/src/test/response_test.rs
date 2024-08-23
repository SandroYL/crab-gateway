#[cfg(test)]
mod tests {
    use crate::connections::response::ResponseHeader;


    #[test]
    fn test_single_header() {
        let req = ResponseHeader::build_with_status_code("404");
        assert!(!req.is_err());
        let mut req = req.unwrap();
        req.insert_header("foo", "bar").unwrap();
        req.append_header("foo", "fkv").unwrap();
        let mut buf = vec![];
        req.header_to_h1_wire(&mut buf);
        assert_eq!(buf, b"foo: bar\r\nfoo: fkv\r\n");
    }

    #[test]
    fn test_illegal_response_header() {
        let req = ResponseHeader::build_with_status_code("999");
        assert!(req.is_err());
    }
}