mod test_mod;
pub mod error_trait;

use std::fmt::{self};
use std::error::Error as ErrorTrait;
use std::result::Result as StdResult;

pub use error_trait::ErrTrans;
use error_trait::OrErr;

#[derive(Debug)]
pub struct Error {
    error_type: ErrorType,
    error_source: ErrorSource,
    error_retry: RetryType,
    error_cause: Option<Box<(dyn ErrorTrait + Send + Sync)>>,
    error_description: Option<String>,
}

type BErr = Box<Error>;
pub type Result<T, E = BErr> = StdResult<T, E>;
#[derive(Debug)]
pub enum ErrorType {
    /*----------Reading Request/Response------------ */
    ReadError,
    ConnectionClosed,
    /*----------Connect Problem------------*/
    ConnectTimeout,
    ConnectRefused,
    InternalError,
    /*----------Connect Problem------------*/
    BindError,
    SocketError,
    HttpCode(u16),
    InvalidHttpHeader,
    /*----------Response Problem------------*/
    ConnectProxyError,
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
            error_description: None 
        }
    }
}

impl ErrorTrait for Error {}

impl Error {
    fn chain_display(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}: context: {};", self.error_type,self.error_description.as_ref().unwrap_or(&"non description.".to_string()))?;
        if let Some(case_error) = self.error_cause.as_ref() {
            write!(f, " Error is transport To ->\n")?;
            case_error.downcast_ref::<Error>().unwrap().chain_display(f)
        } else {
            Ok(())
        }
    }

    ///generate error with cause.
    /// 
    ///[RetryType] not always worked, if error_cause cant retry, then [RetryType] is false.
    fn generate_error(error_type: ErrorType, error_source: ErrorSource, 
        error_retry: RetryType, error_cause: Option<Box<(dyn std::error::Error + Send + Sync + 'static)>>, error_description: Option<&str>,) -> Self {
        
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
            error_description: error_description.as_deref().map(|strs| strs.to_string()),
        }
    }

    /// descripe error.
    /// 
    /// maybe i should make it in step create?
    fn descripe_error(&mut self, description: &str) {
        self.error_description.replace(description.to_string());
    }

    /// new an Error.
    fn new(error_type: ErrorType) -> BErr {
        Box::new(Error::generate_error(error_type, ErrorSource::DownStream, false.into(), None, None))
    }

    fn new_with_reason(error_type: ErrorType, error_cause: &str) -> BErr {
        let mut nself =  Self::new(error_type);
        nself.set_context(error_cause.to_string());
        nself
    }

    pub fn generate_error_with_root<T> (
        error_type: ErrorType, 
        error_description: &str,
        error_cause: Option<Box<(dyn ErrorTrait + Send + Sync)>>
    ) -> Result<T> {
        let mut be = Self::new_with_reason(error_type, error_description);
        if let Some(e) = error_cause {
            be.because(e);
        }
        Err(be)
    }

    pub fn generate_error_with_root_raw (
        error_type: ErrorType, 
        error_description: &str,
        error_cause: Option<Box<(dyn ErrorTrait + Send + Sync)>>
    ) -> BErr {
        let mut be = Self::new_with_reason(error_type, error_description);
        if let Some(e) = error_cause {
            be.because(e);
        }
        be
    }

    fn because(&mut self, cause: Box<(dyn ErrorTrait + Send + Sync)>) {
        self.error_cause.replace(cause);
    }

    fn set_context(&mut self, description: String) {
        self.error_description = Some(description);
    }

    fn change_error_type(mut self, etype: ErrorType) -> BErr {
        self.error_type = etype;
        Box::new(self)
    }

}

impl<T, E> ErrTrans<T, E> for Result<T, E> {
    
    fn explain_error(self, et: ErrorType) -> Result<T, BErr> {
        self.map_err(|_| Error::new(et))
    }
    
    fn to_b_err(self, et: ErrorType, s: &str) -> Result<T, BErr> {
        self.map_err(|_| Error::new_with_reason(et, s))
    }

    
}

impl ErrorType {
    pub fn new_custom_with_code(error_type: &'static str, error_code: u16) -> Self {
        Self::CustomCode(error_type, error_code)
    }    
    pub fn new_custom(error_type: &'static str) -> Self {
        Self::Custom(error_type)
    }
}

impl<T, E> OrErr<T, E> for StdResult<T, E> {
    fn or_err(self, et: ErrorType,
        context: &'static str) -> Result<T, BErr>
    where 
        E: Into<Box<dyn ErrorTrait + Send + Sync>> {
        self.map_err(|e| Error::generate_error_with_root_raw(et, &context, Some(e.into())))
    }

    fn or_fail(self) -> StdResult<T, BErr>
    where
        E: Into<Box<dyn ErrorTrait + Send + Sync>> {
        self.map_err(|e| Error::generate_error_with_root_raw(ErrorType::InternalError, "", Some(e.into())))
    }
}