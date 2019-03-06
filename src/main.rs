#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(dead_code)]
#[macro_use]
extern crate lazy_static;

use ansi_term::Color::Fixed;
use log::{debug, info, log_enabled, trace};
use regex::{Captures, Regex};
use std::{
    env, fs,
    io::{prelude::BufRead, BufReader},
    path::PathBuf,
    process::exit,
};
// use stderrlog;
use loggerv;
use structopt::StructOpt;

/// Constants
const LIME: u8 = 154;
const LIGHTORANGE: u8 = 215;

/// REGEXES
lazy_static! {
    static ref RE_DATE_ISO: Regex = Regex::new(r"(?P<y>\d{4})-(?P<m>\d{2})-(?P<d>\d{2})").unwrap();
}
lazy_static! {
    static ref RE_PROJECT: Regex = Regex::new(r"(\+\w+)").unwrap();
}
lazy_static! {
    static ref RE_CONTEXT: Regex = Regex::new(r"(.*)(@\S+)(.*)").unwrap();
}

type AnyResult<T> = Result<T, Box<std::error::Error>>;

fn format_line_output(line: String) -> Result<String, regex::Error> {
    let line = RE_PROJECT.replace_all(&line, |c: &Captures| {
        format!("{}", Fixed(LIME).paint(&c[0]))
    });
    let line = RE_CONTEXT.replace_all(&line, |caps: &Captures| {
        format!(
            "{}{}{}",
            &caps[1],
            Fixed(LIGHTORANGE).paint(&caps[2]),
            &caps[3]
        )
    });
    Ok(line.to_string())
}

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
    #[structopt(
        short = "v",
        long = "verbose",
        parse(from_occurrences),
        // default_value = "true true true true"
    )]
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
    #[structopt(name = "list", visible_alias = "ls")]
    /// List todos
    List,

    #[structopt(name = "listall", visible_alias = "lsa")]
    /// List all todos
    Listall,
}

fn main() -> Result<(), Box<std::error::Error>> {
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
        .line_numbers(true)
        .level(true)
        .verbosity(verbosity)
        .init()?;
    debug!("{:#?}", opts);

    let home = dirs::home_dir().ok_or("error getting home directory")?;
    let mut path = PathBuf::from(home);
    path.push("Dropbox");
    path.push("todo");
    path.push("todo.txt");
    trace!("path to read: {:?}", &path);

    let todo_file = fs::read_to_string(path)?;
    let formatted = format_line_output(todo_file)?;
    let lines = formatted.lines();
    let mut ctr = 0;
    for line in lines {
        println!("{:02} {}", ctr + 1, line);
        ctr += 1;
    }
    println!("--\nTODO: {} of {} tasks shown", ctr, ctr,);
    Ok(())
}
