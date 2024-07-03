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

type BErr = Box<Error>;

#[derive(Debug)]
pub enum ErrorType {
    /*----------Connect Problem------------*/
    ConnectTimeout,
    ConnectRefused,
    /*----------Connect Problem------------*/
    BindError,
    SocketError,
    HttpCode(u16),
    /*----------DIY Problem------------*/
    Custom(&'static str),
    CustomCode(&'static str, u16),
}
#[derive(Debug)]
pub enum ErrorSource {
    UpStream,
    DownStream,
    Internal,
    Undefined,
}
#[derive(Debug, Clone, Copy)]
pub enum RetryType {
    Decided(bool),
    ReusedOnly,
}

impl Into<RetryType> for bool {
    fn into(self) -> RetryType {
        match self {
            true => RetryType::Decided(true),
            false => RetryType::Decided(false),
        }
    }
}


impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        //链式调用fmt
        self.chain_display(f)
    }
}

impl Default for Error {
    fn default() -> Self {
        Self { 
            error_type: ErrorType::Custom("null"), 
            error_source: ErrorSource::DownStream, 
            error_retry: false.into(), 
            error_cause: None, 
            error_desciption: None 
        }
    }
}

impl ErrorTrait for Error {}

impl Error {
    fn chain_display(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "context: {}", self.error_desciption.as_ref().unwrap())?;
        if let Some(case_error) = self.error_cause.as_ref() {
            write!(f, "\nError is caused by {:?}", case_error.downcast_ref::<Box<Error>>())?;
            self.chain_display(f)
        } else {
            Ok(())
        }
    }

    ///generate error with cause.
    /// 
    ///[RetryType] not always worked, if error_cause cant retry, then [RetryType] is false.
    fn generate_error_withcause(error_type: ErrorType, error_source: ErrorSource, 
        error_retry: RetryType, error_cause: Option<Box<(dyn std::error::Error + Send + Sync + 'static)>>) -> Self {
        
        let retry = if let Some(cause) = error_cause.as_ref() {
            if let Some(upper_cause) = cause.downcast_ref::<Error>() {
                upper_cause.error_retry
            } else {
                false.into()
            }
        } else {
            error_retry
        };
        Error {
            error_type,
            error_source,
            error_retry: retry,
            error_cause,
            error_desciption: None,
        }
    }
}

