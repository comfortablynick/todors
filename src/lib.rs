//! # `todo.sh` reimagined in Rust
//!
//! Mimics the behavior of todo.sh but with additional features:
//!
//! * Much faster
//! * Doesn't require `bash` (Windows support)
//! * No dependencies on gnu tools
//! * Can be configured using a sane format (toml)
//!
//! For example, executing this command:
//!
//! ```sh
//! $ todors ls
//! ```
//!
//! Will produce output identical to `todo.sh`:
//!
//!```
//! 01 (A) Thank Mom for the meatballs @phone
//! 02 (B) Schedule Goodwill pickup +GarageSale @phone
//! 03 Post signs around the neighborhood +GarageSale
//! 04 @GroceryStore Eskimo pies
//! ```
#![allow(clippy::pedantic)]
pub mod actions;
pub mod app;
pub mod color;
pub mod config;
pub mod file;
pub mod prelude;
pub mod style;
pub mod task;
pub mod util;
