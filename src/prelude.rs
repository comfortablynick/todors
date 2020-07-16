//! Re-export of common types and traits used in crate
pub use crate::cli::{App, AppExt, Arg, ArgExt};
pub use anyhow::{format_err, Context, Error};

pub type Result<T = ()> = anyhow::Result<T>;
