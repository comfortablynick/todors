#[macro_use]
extern crate lazy_static;

use ansi_term::Color::Fixed;
use regex::{Captures, Regex};
use std::{fs, path::PathBuf};

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
