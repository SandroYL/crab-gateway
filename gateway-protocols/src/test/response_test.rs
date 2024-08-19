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
/*
    #[test]
    fn test_modify_header() {
        let mut req = ResponseHeader::build_with_method_path("GET", b"/icbc/biom").unwrap();

        req.insert_header("foo", "bar").unwrap();
        req.append_header("Foo", "fkv").unwrap();
        req.modify_header("FoO", "shit").unwrap();
        let mut buf = vec![];
        req.header_to_h1_wire(&mut buf);
        assert_eq!(buf, b"foo: shit\r\n");  
    }

    #[test]
    fn test_remove_header() {
        let mut req = ResponseHeader::build_with_method_path("GET", b"/icbc/biom").unwrap();

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
        let mut req = ResponseHeader::build_with_method_path("GET", b"/icbc/biom").unwrap();

        req.insert_header("foo", "bar").unwrap();
        req.insert_header(http::header::CONTENT_TYPE, "piece").unwrap();
        req.insert_header(http::header::CONTENT_TYPE, "down").unwrap();
        let mut buf = vec![];
        req.header_to_h1_wire(&mut buf);
        assert_eq!(buf, b"foo: bar\r\ncontent-type: down\r\n");          
    }
*/
}