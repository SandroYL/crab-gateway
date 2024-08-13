pub struct SmallCaseHeader(String);

pub trait SmallCaseString {
    fn into_small_case_header(self) -> SmallCaseHeader;
}

impl SmallCaseString for SmallCaseHeader {
    fn into_small_case_header(self) -> SmallCaseHeader {
        self.into()
    }
}

impl Into<SmallCaseHeader> for String {
    fn into(self) -> SmallCaseHeader {
        SmallCaseHeader(self.to_lowercase())
    }
}

impl Into<SmallCaseHeader> for &str {
    fn into(self) -> SmallCaseHeader {
        SmallCaseHeader(self.to_lowercase())
    }
}