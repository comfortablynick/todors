use crate::prelude::*;
use log::debug;
use std::{
    fs::OpenOptions,
    io::{stdin, stdout, Read, Write},
    path::Path,
    process::{Command, Output},
};

/// Get user response to question as 'y' or 'n'
pub fn ask_user_yes_no(prompt_ln: &str) -> Result<bool> {
    let mut input = String::new();
    let stdout = stdout();
    let mut lock = stdout.lock();
    lock.write_all(prompt_ln.as_bytes())?;
    stdin().read_line(&mut input)?;
    if let Some(c) = input.to_lowercase().chars().next() {
        debug!("User input: '{}'", c);
        if c == 'y' {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Get output of todo.sh `list` command
pub fn get_todo_sh_output(argv: Option<&[&str]>, sort_cmd: Option<&str>) -> Result<Output> {
    let sort_cmd = sort_cmd.unwrap_or("sort -f -k 2");
    debug!("TODOTXT_SORT_COMMAND={}", sort_cmd);
    Command::new("todo.sh")
        .args(argv.unwrap_or_default())
        .env("TODOTXT_SORT_COMMAND", sort_cmd)
        .output()
        .context("get_todo_sh_output() failed")
}

/// Get string priority name in the form of `pri_x`
pub fn get_pri_name(pri: u8) -> Option<String> {
    match pri {
        0..=25 => {
            let mut s = String::from("pri_");
            s.push((pri + 97).into());
            Some(s)
        }
        _ => None,
    }
}

/// Read file to string
pub fn read_file_to_string<P>(file_path: P) -> Result<String>
where
    P: AsRef<Path> + std::fmt::Debug,
{
    OpenOptions::new()
        .read(true)
        .open(&file_path)
        .and_then(|mut file| {
            let mut buf = String::new();
            file.read_to_string(&mut buf).map(|_| buf)
        })
        .with_context(|| format!("reading file {:?} to string", file_path))
}
