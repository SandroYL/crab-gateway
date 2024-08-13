
use crate::{BErr, ErrorType};


pub trait ErrTrans<T, E> {
    fn explain_error(self, et: ErrorType) -> Result<T, BErr>;
    fn to_b_err(self, et: ErrorType, s: &str) -> Result<T, BErr>;
}