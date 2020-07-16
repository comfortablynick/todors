use crate::prelude::*;
use log::{self, debug};
use std::{
    io::{stdin, stdout, Write},
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
