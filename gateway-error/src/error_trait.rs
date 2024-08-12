use std::error::Error;

use crate::{BErr, ErrorType};


pub trait ErrTrans<T> {
    fn explain_error<F: FnOnce(ErrorType) -> BErr>(&mut self, f: F) -> Result<T, BErr>;

    fn add_context<F: FnOnce(&str)> (&mut self, f: F) -> Result<T, BErr>;
}