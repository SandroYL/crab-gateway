
use std::error::Error as ErrorTrait;

use crate::{BErr, ErrorType};
use std::result::Result as StdResult;
pub type Result<T, E = BErr> = StdResult<T, E>;

pub trait ErrTrans<T, E> {
    fn explain_error(self, et: ErrorType) -> Result<T, BErr>;
    fn to_b_err(self, et: ErrorType, s: &str) -> Result<T, BErr>;
}

pub trait OrErr<T, E> {
    fn or_err(self, et: ErrorType,
        context: &'static str) -> Result<T, BErr>
    where 
        E: Into<Box<dyn ErrorTrait + Send + Sync>>;
    
    fn or_fail(self) -> Result<T, BErr>
    where
        E: Into<Box<dyn ErrorTrait + Send + Sync>>;
}