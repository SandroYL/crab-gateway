#[cfg(test)]
mod test {
    use crate::util::case_sense_map::CaseSenseMap;

    #[test]
    fn init_map() {
        let mut case_map = CaseSenseMap::new();
        case_map.insert("a".to_string(), "b".to_string());
        case_map.append("A".to_string(), "c".to_string());
        assert_eq!(case_map.get("a").unwrap(), "[b,c]".to_string());
    }

    #[test]
    fn duplicate_insert() {
        let mut case_map = CaseSenseMap::new();
        case_map.insert("a".to_string(), "b".to_string());
        case_map.append("A".to_string(), "c".to_string());

        assert_eq!(case_map.get("a").unwrap(), "[b,c]".to_string());

        let out = case_map.insert("a".to_string(), "d".to_string());

        assert_eq!(out.unwrap(), "[b,c]".to_string());
        assert_eq!(case_map.get("a").unwrap(), "[d]".to_string());
    }

    #[test]
    fn value_is_capital() {
        let mut case_map = CaseSenseMap::new();
        case_map.insert("a".to_string(), "b".to_string());
        case_map.append("A".to_string(), "c".to_string());
        case_map.append("A".to_string(), "B".to_string());

        assert_eq!(case_map.get("A").unwrap(), "[b,c,B]".to_string());
    }
}