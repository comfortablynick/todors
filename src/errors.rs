pub use failure::Error;
use std::result::Result as StdResult;

pub type Result<T = ()> = StdResult<T, Error>;

// #[derive(Debug, Fail)]
// pub enum Error {
//     #[fail(display = "parse error")]
//     ParseError,
//     #[fail(display = "error executing command")]
//     CommandError(#[cause] std::io::Error),
// }
//
// impl From<std::io::Error> for Error {
//
// }
