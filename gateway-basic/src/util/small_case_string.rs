
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
        self.0.clone()
    }
}

impl SmallCaseString for &str {
    fn into_small_case_header(self) -> SmallCaseHeader {
        self.into()
    }

    fn to_string(&self) -> String {
        String::from(*self)
    }
}

impl SmallCaseString for String {
    fn into_small_case_header(self) -> SmallCaseHeader {
        self.into()
    }

    fn to_string(&self) -> String {
        self.clone()
    }
}

impl SmallCaseString for http::header::HeaderName {
    fn into_small_case_header(self) -> SmallCaseHeader {
        SmallCaseString::to_string(&self).into_small_case_header()
    }

    fn to_string(&self) -> String {
        ToString::to_string(&self)
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
        self.0.clone()
    }
}

impl Into<SmallCaseHeader> for &str {
    fn into(self) -> SmallCaseHeader {
        SmallCaseHeader(self.to_lowercase())
    }
}