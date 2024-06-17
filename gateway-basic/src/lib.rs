use std::error::Error as ErrorTrait;

pub struct Error {
    pub error_type: ErrorType,
    pub error_source: ErrorSource,
    pub error_retry: RetryType,
    pub error_cause: Option<Box<(dyn ErrorTrait + Send + Sync)>>,
    pub error_desciption: Option<String>,
}

pub enum ErrorType {
    /*----------Connect Problem------------*/
    ConnectTimeout,
    ConnectRefused,
    /*----------Connect Problem------------*/
    BindError,
    SocketError,
    HttpCode(u16),
}

pub enum ErrorSource {
    UpStream,
    DownStream,
    Internal,
    Undefined,
}

pub enum RetryType {
    Decided(bool),
    ReusedOnly,
}