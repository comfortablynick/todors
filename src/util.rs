use crate::{
    cli::Output,
    errors::{Error, Result, ResultExt},
};
use chrono::Local;
use env_logger::{fmt::Color, Env};
use log::{self, debug, Level};
use std::{
    io::{stdin, stdout, Write},
    process::Command as ExtCommand,
};

/// Get user response to question as 'y' or 'n'
pub fn ask_user_yes_no(prompt_ln: &str) -> Result<bool> {
    let mut cin = String::new();
    stdout().write_all(prompt_ln.as_bytes())?;
    stdout().flush()?;
    stdin().read_line(&mut cin)?;
    if let Some(c) = cin.to_lowercase().chars().next() {
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
    ExtCommand::new("todo.sh")
        .args(argv.unwrap_or_default())
        .env("TODOTXT_SORT_COMMAND", sort_cmd)
        .output()
        .context("get_todo_sh_output(): error getting command output")
        .map_err(Error::from)
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

/// Initialize customized instance of env_logger
pub fn init_env_logger(verbose: u8) {
    env_logger::Builder::from_env(Env::new().default_filter_or(match verbose {
        0 => "warn",
        1 => "info",
        2 => "debug",
        _ => "trace",
    }))
    .format(|buf, record| {
        let mut level_style = buf.style();
        match record.level() {
            Level::Trace => level_style.set_color(Color::Ansi256(142)), // dim yellow
            Level::Debug => level_style.set_color(Color::Ansi256(37)),  // dim cyan
            Level::Info => level_style.set_color(Color::Ansi256(34)),   // dim green
            Level::Warn => level_style.set_color(Color::Ansi256(130)),  // dim orange
            Level::Error => level_style.set_color(Color::Red).set_bold(true),
        };

        let level = level_style.value(format!("{:5}", record.level()));
        let tm_fmt = "%F %H:%M:%S%.3f";
        let time = Local::now().format(tm_fmt);

        let mut subtle_style = buf.style();
        subtle_style.set_color(Color::Black).set_intense(true);

        let mut gray_style = buf.style();
        gray_style.set_color(Color::Ansi256(250));

        writeln!(
            buf,
            "\
             {lbracket}\
             {time}\
             {rbracket}\
             {level}\
             {lbracket}\
             {file}\
             {colon}\
             {line_no}\
             {rbracket} \
             {record_args}\
             ",
            lbracket = subtle_style.value("["),
            rbracket = subtle_style.value("]"),
            colon = subtle_style.value(":"),
            file = gray_style.value(record.file().unwrap_or_default()),
            time = gray_style.value(time),
            level = level,
            line_no = gray_style.value(record.line().unwrap_or_default()),
            record_args = &record.args(),
        )
    })
    .init();
}
