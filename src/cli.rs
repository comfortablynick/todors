use crate::{args, logger, util::AnyError};
use ansi_term::Color::Fixed;
use log::{debug, error, info, log_enabled, trace, warn, Level, LevelFilter};
use regex::{Captures, Regex};
use std::{fs, io::Write, path::PathBuf};
use structopt::StructOpt;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

/// Colors
const HOTPINK: u8 = 198;
const GREY: u8 = 246;
const SKYBLUE: u8 = 111;
const OLIVE: u8 = 113;
const LIME: u8 = 154;
const LIGHTORANGE: u8 = 215;
const GREEN: u8 = 112;
// lazy_static! {
//     static ref RE_PROJECT: Regex = Regex::new(r"(\+\w+)").unwrap();
//     static ref RE_CONTEXT: Regex = Regex::new(r"(@\w+)").unwrap();
//     static ref RE_PRIORITY: Regex = Regex::new(r"(?m)^\((.)\)").unwrap();
//     static ref RE_DATE_ISO: Regex = Regex::new(r"(?P<y>\d{4})-(?P<m>\d{2})-(?P<d>\d{2})").unwrap();
// }

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

fn format_colors(s: String) -> String {
    lazy_static! {
        static ref RE_PROJECT: Regex = Regex::new(r"(\+\w+)").unwrap();
        static ref RE_CONTEXT: Regex = Regex::new(r"(@\w+)").unwrap();
    }
    let s = RE_PROJECT.replace_all(&s, |c: &Captures| format!("{}", Fixed(LIME).paint(&c[0])));
    let s = RE_CONTEXT.replace_all(&s, |caps: &Captures| {
        format!("{}", Fixed(LIGHTORANGE).paint(&caps[0]),)
    });
    s.to_string()
}

// fn match_pri(s: &str) {
//     lazy_static! {
//         static ref RE_PRIORITY: Regex = Regex::new(r"(?m)^\((.)\).*$").unwrap();
//     }
//     for cap in RE_PRIORITY.captures_iter(s) {
//         let ch: u32 = cap[1].chars().next().expect("error getting priority") as u32;
//         debug!("Priority: {}/{} | Todo: {}", &cap[1], ch - 64, &cap[0]);
//     }
// }

fn format_priority(s: String) -> String {
    lazy_static! {
        static ref RE_PRIORITY: Regex = Regex::new(r"(?m)^\((.)\).*$").unwrap();
    }
    let s = RE_PRIORITY.replace_all(&s, |c: &Captures| {
        format!("{}", Fixed(HOTPINK).paint(&c[0]))
    });
    s.to_string()
}

fn print_todos(s: String) {
    let lines = s.lines();
    let mut ctr = 0;
    for line in lines {
        if line != "" {
            println!("{:02} {}", ctr + 1, line);
            ctr += 1;
        }
    }
    println!("--\nTODO: {} of {} tasks shown", ctr, ctr,);
}

fn test_termcolor(s: &str) {
    let mut buf = StandardStream::stderr(ColorChoice::Always);
    buf.set_color(ColorSpec::new().set_fg(Some(Color::Ansi256(LIGHTORANGE))))
        .expect("error writing color to stdout");
    writeln!(&mut buf, "{}", s).expect("error writing to buffer");
}

pub fn run(args: Vec<String>) -> Result<(), AnyError> {
    let opts = args::Opt::from_iter(&args);

    if !opts.quiet {
        // init logger
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
    let formatted = format_priority(lines);
    let formatted = format_colors(formatted);
    print_todos(formatted);
    test_termcolor("test orange text on stderr!");
    Ok(())
}
