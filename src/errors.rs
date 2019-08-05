use failure::Fail;
pub use failure::{err_msg, Error, ResultExt};
use std::result::Result as StdResult;

pub type Result<T = ()> = StdResult<T, Error>;

#[derive(Debug, Fail)]
pub enum ErrorType {
    /// File errors
    #[fail(display = "file does not exist")]
    FileNotExistError,
    #[fail(display = "An error occurred opening file: {}.", _0)]
    FileOpenError(String),
    #[fail(display = "unable to read file")]
    FileReadError,
    #[fail(display = "unable to write file")]
    FileWriteError,
    /// Parsing errors
    #[fail(display = "unable to parse")]
    ParseError,
    #[fail(display = "unable to convert to type")]
    TypeConversionError,
    #[fail(display = "Option value is None: {}.", _0)]
    NoneError(String),
}
