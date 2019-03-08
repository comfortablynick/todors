#![allow(unused_imports)]
#[macro_use]
extern crate lazy_static;

use ansi_term::Color::Fixed;
use env_logger::fmt::{Color, Style};
use log::{debug, error, info, log_enabled, trace, warn, Level, LevelFilter};
use regex::{Captures, Regex};
use std::io::Write;
use std::{fs, path::PathBuf};
use structopt::StructOpt;

/// Colors
pub const GREY: u8 = 246;
pub const SKYBLUE: u8 = 111;
pub const OLIVE: u8 = 113;
pub const LIME: u8 = 154;
pub const LIGHTORANGE: u8 = 215;

/// Regexes
lazy_static! {
    static ref RE_DATE_ISO: Regex = Regex::new(r"(?P<y>\d{4})-(?P<m>\d{2})-(?P<d>\d{2})").unwrap();
    static ref RE_PROJECT: Regex = Regex::new(r"(\+\w+)").unwrap();
    static ref RE_CONTEXT: Regex = Regex::new(r"(.*)(@\S+)(.*)").unwrap();
}

pub type AnyError = Box<dyn std::error::Error + 'static>;

/// Command line options
#[derive(Debug, StructOpt)]
#[structopt(
    name = "todors",
    about = "View and edit a file in todo.txt format",
    raw(setting = "structopt::clap::AppSettings::ColoredHelp"),
    raw(setting = "structopt::clap::AppSettings::DontCollapseArgsInUsage")
)]
pub struct Opt {
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[structopt(short = "v", long = "verbose", parse(from_occurrences))]
    verbose: u8,

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
pub enum Command {
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

pub struct TodoFile {
    pub path: PathBuf,
    pub contents: String,
}

impl TodoFile {
    pub fn new() -> TodoFile {
        let file_path = TodoFile::get_path().unwrap();
        TodoFile {
            path: file_path.clone(),
            contents: fs::read_to_string(file_path).unwrap(),
        }
    }

    fn get_path() -> Result<PathBuf, AnyError> {
        let home = dirs::home_dir().ok_or("error getting home directory")?;
        let mut path = PathBuf::from(home);
        path.push("Dropbox");
        path.push("todo");
        path.push("todo.txt");
        Ok(path)
    }
}

pub fn format_colors(line: String) -> Result<String, regex::Error> {
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

#[allow(dead_code)]
fn init_loggerv(verbosity: u8) {
    loggerv::Logger::new()
        .output(&Level::Error, loggerv::Output::Stderr)
        .output(&Level::Warn, loggerv::Output::Stderr)
        .output(&Level::Info, loggerv::Output::Stderr)
        .output(&Level::Debug, loggerv::Output::Stderr)
        .output(&Level::Trace, loggerv::Output::Stderr)
        .color(&Level::Trace, Fixed(GREY))
        .color(&Level::Debug, Fixed(SKYBLUE))
        .color(&Level::Info, Fixed(OLIVE))
        .line_numbers(true)
        .level(true)
        .verbosity(verbosity as u64)
        .init()
        .expect("Initialize loggerv");
}

#[allow(dead_code)]
fn init_env_logger(verbosity: u8) {
    let mut log_builder = env_logger::Builder::new();
    log_builder
        .format(|buf, record| {
            let target = record.target();
            let mut level_style = buf.style();
            let level = record.level();
            match level {
                Level::Trace => level_style.set_color(Color::Magenta).set_bold(false),
                Level::Debug => level_style.set_color(Color::Blue).set_bold(true),
                Level::Info => level_style.set_color(Color::Green).set_bold(true),
                Level::Warn => level_style.set_color(Color::Yellow).set_bold(true),
                Level::Error => level_style.set_color(Color::Red).set_bold(true),
            };
            let mut style = buf.style();
            let target =
                style
                    .set_bold(true)
                    .value(format!("{: <width$}", target, width = target.len()));
            writeln!(
                buf,
                " {} {} > {}",
                level_style.value(level),
                target,
                record.args(),
            )
        })
        .filter(
            Some("todors"),
            match verbosity {
                0 => LevelFilter::Warn,
                1 => LevelFilter::Info,
                2 => LevelFilter::Debug,
                _ => LevelFilter::Trace,
            },
        )
        .init();
}

pub fn run(args: Vec<String>) -> Result<(), AnyError> {
    let opts = Opt::from_iter(args);

    // init logger if no -q
    if !opts.quiet {
        // init_env_logger(opts.verbose);
        init_loggerv(opts.verbose);
    }

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
