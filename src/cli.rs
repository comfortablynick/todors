// #![allow(unused_imports)]
// #![allow(dead_code)]

use crate::{args, logger, util::AnyError};
use ansi_term::Color::Fixed;
use log::{debug, error, info, log_enabled, trace, warn, Level, LevelFilter};
use regex::{Captures, Regex};
use std::{fs, path::PathBuf};
use structopt::StructOpt;

/// Colors
const HOTPINK: u8 = 198;
const GREY: u8 = 246;
const SKYBLUE: u8 = 111;
const OLIVE: u8 = 113;
const LIME: u8 = 154;
const LIGHTORANGE: u8 = 215;

/// Regexes
lazy_static! {
    static ref RE_DATE_ISO: Regex = Regex::new(r"(?P<y>\d{4})-(?P<m>\d{2})-(?P<d>\d{2})").unwrap();
    static ref RE_PROJECT: Regex = Regex::new(r"(\+\w+)").unwrap();
    static ref RE_CONTEXT: Regex = Regex::new(r"(.*)(@\S+)(.*)").unwrap();
    static ref RE_PRIORITY: Regex = Regex::new(r"(?m)^\((.)\)").unwrap();
}

#[derive(Default)]
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
        let mut path: PathBuf = home;
        path.push("Dropbox");
        path.push("todo");
        path.push("todo.txt");
        Ok(path)
    }
}

fn format_priority(s: String) -> Result<String, AnyError> {
    // TODO: look for better way to 'join' back to lines
    let s = s
        .lines()
        .map(|ln| {
            if RE_PRIORITY.is_match(ln) {
                format!("{}\n", Fixed(HOTPINK).paint(ln))
            } else {
                format!("{}\n", ln)
            }
        })
        .collect();
    Ok(s)
}

fn format_colors(line: String) -> Result<String, regex::Error> {
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

fn print_todos(s: String) {
    let lines = s.lines();
    let mut ctr = 0;
    for line in lines {
        println!("{:02} {}", ctr + 1, line);
        ctr += 1;
    }
    println!("--\nTODO: {} of {} tasks shown", ctr, ctr,);
}

pub fn run(args: Vec<String>) -> Result<(), AnyError> {
    let opts = args::Opt::from_iter(&args);

    // init logger if no -q
    if !opts.quiet {
        // util::init_logger(opts.verbose);
        logger::Logger::init().expect("error initializing logger");
        log::set_max_level(match opts.verbose {
            0 => LevelFilter::Warn,
            1 => LevelFilter::Info,
            2 => LevelFilter::Debug,
            _ => LevelFilter::Trace,
        });
    }

    trace!("Running with args: {:?}", args);
    debug!("Parsed options:\n{:#?}", opts);

    let todo_file = TodoFile::new();
    let lines = todo_file.contents;
    // let lines: Vec<String> = todo_file
    //     .contents
    //     .lines()
    //     .map(|ln| ln.to_string())
    //     .collect();

    let formatted = format_priority(lines)?;
    let formatted = format_colors(formatted)?;
    print_todos(formatted);
    // let formatted = format_priority(lines)?;
    // let formatted = format_colors(formatted)?;
    // print_todos(formatted);
    Ok(())
}
