use std::error::Error;

use crate::{BErr, ErrorType};

pub trait TransferErr<T, E> {
    fn or_err(self, et: ErrorType, context: &'static str) -> Result<T, BErr>
    where
        E:Into<Box<dyn Error + Send + Sync>>;

    fn or_err_with<C: Into<ImmutStr>>
}