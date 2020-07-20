//! Re-export of common types and traits used in crate
pub use anyhow::{bail, format_err, Context, Error};

pub type Result<T = ()> = anyhow::Result<T>;
