//! Re-export of common types and traits used in crate
pub use anyhow::{bail, format_err, Context, Error};
pub use log::{debug, info, trace};

pub type Result<T = ()> = anyhow::Result<T>;
