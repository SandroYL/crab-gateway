use std::fmt::{self};
use std::error::Error as ErrorTrait;

#[derive(Debug)]
pub struct Error {
    pub error_type: ErrorType,
    pub error_source: ErrorSource,
    pub error_retry: RetryType,
    pub error_cause: Option<Box<(dyn ErrorTrait + Send + Sync)>>,
    pub error_desciption: Option<String>,
}
#[derive(Debug)]
pub enum ErrorType {
    /*----------Connect Problem------------*/
    ConnectTimeout,
    ConnectRefused,
    /*----------Connect Problem------------*/
    BindError,
    SocketError,
    HttpCode(u16),
}
#[derive(Debug)]
pub enum ErrorSource {
    UpStream,
    DownStream,
    Internal,
    Undefined,
}
#[derive(Debug)]
pub enum RetryType {
    Decided(bool),
    ReusedOnly,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        //链式调用fmt
        self.chain_display(f)
    }
}

impl ErrorTrait for Error {}

impl Error {
    fn chain_display(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "context: {}", self.error_desciption.as_ref().unwrap())

    }
}

