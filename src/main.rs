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
use structopt::StructOpt;
use todors::*;

type AnyError = Box<dyn std::error::Error>;

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
    run(args)
}
