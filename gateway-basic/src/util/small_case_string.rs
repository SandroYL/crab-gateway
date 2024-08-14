use std::clone;

pub struct SmallCaseHeader(String);

pub trait SmallCaseString {
    fn into_small_case_header(self) -> SmallCaseHeader;

    fn to_string(&self) -> String;
}

impl SmallCaseString for SmallCaseHeader {
    fn into_small_case_header(self) -> SmallCaseHeader {
        self.into()
    }
    
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl Into<SmallCaseHeader> for String {
    fn into(self) -> SmallCaseHeader {
        SmallCaseHeader(self.to_lowercase())
    }
}

impl SmallCaseHeader {
    pub fn as_slice(&self) -> &[u8] {
        self.0.as_bytes()
    }

    pub fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl Into<SmallCaseHeader> for &str {
    fn into(self) -> SmallCaseHeader {
        SmallCaseHeader(self.to_lowercase())
    }
}