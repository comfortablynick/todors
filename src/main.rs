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

/// Command line options
#[derive(Debug, StructOpt)]
#[structopt(
    name = "todors",
    about = "View and edit a file in todo.txt format",
    raw(setting = "structopt::clap::AppSettings::ColoredHelp"),
    raw(setting = "structopt::clap::AppSettings::DontCollapseArgsInUsage")
)]
struct Opt {
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[structopt(short = "v", long = "verbose", parse(from_occurrences))]
    verbose: u64,

    /// Quiet debug messages
    #[structopt(short = "q", long = "quiet")]
    quiet: bool,

    /// Usage information
    #[structopt(long = "usage")]
    usage: bool,

    /// Use a config file othe rthan the default ~/.todo/config
    #[structopt(short = "d", name = "CONFIG_FILE", parse(from_os_str))]
    config_file: Option<PathBuf>,

    /// List contents of todo.txt file
    #[structopt(subcommand)]
    cmd: Option<Command>,
}

#[derive(StructOpt, Debug)]
enum Command {
    /// Add line to todo.txt file
    #[structopt(name = "add", visible_alias = "a")]
    Add,

    /// Add multiple lines to todo.txt file
    #[structopt(name = "addm")]
    Addm,

    /// Add line of text to any file in the todo.txt directory
    #[structopt(name = "addto")]
    Addto,

    /// Add text to end of the item
    #[structopt(name = "append", visible_alias = "app")]
    Append {
        /// Append text to end of this line number
        #[structopt(name = "item")]
        item: u32,

        /// Text to append (quotes optional)
        #[structopt(name = "text")]
        text: String,
    },

    /// List todos
    #[structopt(name = "list", visible_alias = "ls")]
    List,

    /// List all todos
    #[structopt(name = "listall", visible_alias = "lsa")]
    Listall,
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
    let opts = Opt::from_iter(args);
    let verbosity = if opts.quiet { 0 } else { opts.verbose };

    loggerv::Logger::new()
        .output(&log::Level::Error, loggerv::Output::Stderr)
        .output(&log::Level::Warn, loggerv::Output::Stderr)
        .output(&log::Level::Info, loggerv::Output::Stderr)
        .output(&log::Level::Debug, loggerv::Output::Stderr)
        .output(&log::Level::Trace, loggerv::Output::Stderr)
        .color(&log::Level::Trace, Fixed(GREY))
        .color(&log::Level::Debug, Fixed(SKYBLUE))
        .color(&log::Level::Info, Fixed(OLIVE))
        .line_numbers(true)
        .level(true)
        .verbosity(verbosity)
        .init()?;
    debug!("{:#?}", opts);

    let todo_file = TodoFile::new();
    let formatted = format_colors(todo_file.contents)?;
    let lines = formatted.lines();
    let mut ctr = 0;
    for line in lines {
        println!("{:02} {}", ctr + 1, line);
        ctr += 1;
    }
    println!("--\nTODO: {} of {} tasks shown", ctr, ctr,);
    Ok(())
}
