#![allow(dead_code)]
#[macro_use]
extern crate lazy_static;

use ansi_term::Color::Fixed;
use env_logger;
use log::info;
use regex::{Captures, Regex};
use std::io::prelude::BufRead;
use std::{env, fs, io, path::PathBuf};

const LIME: u8 = 154;
const LIGHTORANGE: u8 = 215;

// REGEXES
// let re_date = Regex::new(r"(?P<y>\d{4})-(?P<m>\d{2})-(?P<d>\d{2})").unwrap();
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

fn main() -> Result<(), Box<std::error::Error>> {
    env::set_var("RUST_LOG", "warning");
    env_logger::init();

    let home = dirs::home_dir().ok_or("error getting home directory")?;
    let mut path = PathBuf::from(home);
    path.push("Dropbox");
    path.push("todo");
    path.push("todo.txt");
    info!("path to read: {:?}", &path);

    let todo_file = io::BufReader::new(fs::File::open(path)?);
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
