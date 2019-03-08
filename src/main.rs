#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(dead_code)]

use ansi_term::Color::Fixed;
use log::{debug, error, info, log_enabled, trace, warn};
use loggerv;
use std::{
    env, fs,
    io::{prelude::BufRead, BufReader},
    path::PathBuf,
    process::exit,
};
use todors::*;

/// Enable ANSI color support if compiled on Windows
#[cfg(target_os = "windows")]
fn enable_windows_ansi() -> Result<(), u32> {
    ansi_term::enable_ansi_support()
}

fn main() -> Result<(), AnyError> {
    // TODO: remove test vector after testing
    let args: Vec<String> = if std::env::args().len() > 1 {
        std::env::args().collect()
    } else {
        vec!["todors", "-vvvv"]
            .iter()
            .map(|s| s.to_string())
            .collect()
    };
    let enabled = enable_windows_ansi();
    run(args)
}
