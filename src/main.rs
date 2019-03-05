#![allow(unused_imports)]
#![allow(dead_code)]
#[macro_use]
extern crate lazy_static;

use ansi_term::Color::Fixed;
// use env_logger;
use getopts;
use log::{debug, info, log_enabled, trace};
use regex::{Captures, Regex};
use std::{
    env, fs,
    io::{prelude::BufRead, BufReader},
    path::PathBuf,
    process::exit,
};
use stderrlog;

/// Constants
const VERSION: &'static str = "0.0.1";
const AUTHOR: &'static str = "Nick Murphy <comfortablynick@gmail.com>";
const LIME: u8 = 154;
const LIGHTORANGE: u8 = 215;

/// REGEXES
// lazy_static! {
//     static ref RE_DATE_ISO: Regex = Regex::new(r"(?P<y>\d{4})-(?P<m>\d{2})-(?P<d>\d{2})").unwrap();
// }
lazy_static! {
    static ref RE_PROJECT: Regex = Regex::new(r"(\+\w+)").unwrap();
}
lazy_static! {
    static ref RE_CONTEXT: Regex = Regex::new(r"(.*)(@\S+)(.*)").unwrap();
}

type AnyResult<T> = Result<T, Box<std::error::Error>>;

fn format_line_output(line: &str) -> Result<String, regex::Error> {
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

fn print_version(program: &str) {
    println!("{} {}", &program, &VERSION);
}

fn print_usage(program: &str, opts: getopts::Options) {
    print_version(program);
    println!("{}\n", &AUTHOR);
    let brief = format!("Usage: {} [OPTIONS]", program);
    print!("{}", opts.usage(&brief));
}

fn main() -> Result<(), Box<std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = getopts::Options::new();

    // cli options
    opts.optflag("h", "help", "display this message and exit");
    opts.optflag("V", "version", "display version information and exit");
    opts.optflagmulti(
        "v",
        "verbose",
        "display verbose debug information (ex: '-vv' for INFO)",
    );
    opts.optopt("f", "todo-file", "use this todo.txt file", "NAME");

    let matches = opts.parse(&args[1..])?;
    let verbosity: usize = match matches.opt_count("v") {
        0 => 0,
        1 => 2,
        2 => 3,
        _ => 4,
    };
    stderrlog::new().verbosity(verbosity).quiet(false).init()?;

    if matches.opt_present("h") {
        print_usage(&program, opts);
        exit(1);
    }

    if matches.opt_present("V") {
        println!("{} {}", &program, &VERSION);
        exit(0);
    }

    debug!("{:#?}", &matches);
    let home = dirs::home_dir().ok_or("error getting home directory")?;
    let mut path = PathBuf::from(home);
    path.push("Dropbox");
    path.push("todo");
    path.push("todo.txt");
    trace!("path to read: {:?}", &path);

    let todo_file = BufReader::new(fs::File::open(path)?);
    let lines = todo_file.lines();
    let mut ctr = 0;
    // TODO: read into string and do regex, then iterate to add counts and compare perf
    for line in lines {
        let line = format_line_output(&line?)?;
        println!("{:02} {}", ctr + 1, line);
        ctr += 1;
    }
    println!("--\nTODO: {} of {} tasks shown", ctr, ctr,);
    Ok(())
}
