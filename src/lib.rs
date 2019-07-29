#![allow(unused_imports)]

#[macro_use]
extern crate lazy_static;

// mod args;
// use structopt::StructOpt;
pub mod actions;
pub mod app;
pub mod cli;
pub mod config;
pub mod file;
pub mod logger;
pub mod style;
pub mod task;
pub mod util;
use crate::{actions::list, cli::*, config::Context, style::*};

use chrono::Utc;
use failure::{err_msg, ResultExt};
use log::{debug, info, trace};
use regex::Regex;
use std::{
    cmp::Ordering,
    fmt::{self, Display},
    fs::OpenOptions,
    io::{self, Read, Write},
    ops::AddAssign,
    path::{Path, PathBuf},
    process::{exit, Command as ExtCommand, Output},
};
use termcolor::{Color, ColorSpec, WriteColor};

/// Entry point for main program logic
pub fn run(args: &[String], buf: &mut termcolor::Buffer) -> Result {
    // let opts = Opt::from_iter(args);
    let opts = cli::parse(args)?;

    if !opts.quiet {
        logger::init_logger(opts.verbosity);
    }
    if opts.plain {
        std::env::set_var("TERM", "dumb");
    }
    info!("Running with args: {:?}", args);
    let cfg_file = opts
        .config_file
        .clone()
        .expect("could not find valid cfg file path");
    let cfg = config::read_config(cfg_file)?;
    let mut ctx = Context {
        opts,
        settings: cfg.general,
        styles: cfg.styles,
        ..Default::default()
    };
    actions::handle_command(&mut ctx, buf)?;
    // trace!(
    //     "todo.sh output:\n{:?}",
    //     std::str::from_utf8(&get_todo_sh_output(None, Some("sort"))?.stdout)?
    // );
    // if !buf.is_empty() {
    //     trace!(
    //         "Buffer contents:\n{:?}",
    //         std::str::from_utf8(buf.as_slice())?
    //     );
    // }
    Ok(())
}
