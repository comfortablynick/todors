#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(dead_code)]

use ansi_term::Color::Fixed;
use std::{
    env, fs,
    io::{prelude::BufRead, BufReader},
    path::PathBuf,
    process::exit,
};
use todors::*;

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
    // turn on ANSI escape support on Windows to use color
    #[cfg(windows)]
    ansi_term::enable_ansi_support().expect("Enable ANSI support on Windows");
    run(args)
}
