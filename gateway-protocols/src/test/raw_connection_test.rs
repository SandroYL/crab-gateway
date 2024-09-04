#[cfg(test)]
mod test {
    use std::collections::BTreeMap;
    use crate::connections::row_connection::generate_connect_header;


    #[test]
    fn test_generate_connect_header_v4() {
        let mut headers = BTreeMap::new();
        headers.insert(String::from("test"), b"test_val".to_vec());
        let req = generate_connect_header("baidu.com", 8080,
    headers.iter()).unwrap();

        assert_eq!(req.method, http::method::Method::CONNECT);

        assert_eq!(req.uri.authority().unwrap(), "baidu.com:8080");

        assert_eq!(req.headers.get(http::header::HOST).unwrap(), "baidu.com:8080");

        assert_eq!(req.headers.get("test").unwrap(), "test_val");
    }

    #[test]
    fn test_generate_connect_header_v6() {
        let mut headers = BTreeMap::new();
        headers.insert(String::from("test"), b"test_val".to_vec());
        let req = generate_connect_header("::1", 8080,
    headers.iter()).unwrap();

        assert_eq!(req.method, http::method::Method::CONNECT);

        assert_eq!(req.uri.authority().unwrap(), "[::1]:8080");

        assert_eq!(req.headers.get(http::header::HOST).unwrap(), "[::1]:8080");

        assert_eq!(req.headers.get("test").unwrap(), "test_val");
    }
}