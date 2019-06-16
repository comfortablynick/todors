/** Main program logic */
use crate::{
    args::{self, Command},
    util::logger,
};
use failure::{err_msg, Error};
use log::{debug, info, trace};
use regex::{self, Regex};
use serde::Deserialize;
use std::collections::HashMap;
use std::{fs, io::Write, path::PathBuf, str::FromStr};
use structopt::StructOpt;
use termcolor::{BufferWriter, Color, ColorChoice, ColorSpec, WriteColor};

// Color/formatting {{{1
// struct Ansi {{{2
#[derive(Debug)]
/// Store constants of ANSI 256-color code
struct Ansi;

#[allow(dead_code)]
impl Ansi {
    const HOTPINK: u8 = 198;
    const LIME: u8 = 154;
    const LIGHTORANGE: u8 = 215;
    const GREEN: u8 = 2;
    const BLUE: u8 = 4;
    const TURQUOISE: u8 = 37;
    const TAN: u8 = 179;
    const GREY: u8 = 246;
    const SKYBLUE: u8 = 111;
    const OLIVE: u8 = 113;
}

/// Get color for a given priority
fn get_priority_color(c: char) -> Result<ColorSpec, Error> { //{{{2
    let mut color = ColorSpec::new();
    match c {
        'A' => color.set_fg(Some(Color::Ansi256(Ansi::HOTPINK))),
        'B' => color.set_fg(Some(Color::Ansi256(Ansi::GREEN))),
        'C' => color
            .set_fg(Some(Color::Ansi256(Ansi::BLUE)))
            .set_bold(true),
        'D' => color
            .set_fg(Some(Color::Ansi256(Ansi::TURQUOISE)))
            .set_bold(true),
        'E'...'Z' => color.set_fg(Some(Color::Ansi256(Ansi::TAN))),
        _ => unreachable!("priority `{}` not found!", &c),
    };
    Ok(color)
}

/// Use regex to add color to priorities, projects and contexts
fn format_buffer(s: &[String], bufwtr: BufferWriter, opts: &args::Opt) -> Result<(), Error> { //{{{2
    lazy_static! {
        static ref RE_PRIORITY: Regex = Regex::new(r"(?m)\(([A-Z])\).*$").unwrap();
    }
    let mut buf = bufwtr.buffer();
    let mut color = ColorSpec::new();
    for ln in s {
        let line = ln;
        if let Some(caps) = RE_PRIORITY.captures(&line) {
            let color = get_priority_color(
                caps.get(1)
                    .map_or("", |c| c.as_str())
                    .chars()
                    .next()
                    .expect("error getting priority"),
            )?;
            buf.set_color(&color)?;
        }
        for word in line.split_whitespace() {
            let first_char = word.chars().next();
            let prev_color = color.fg().cloned();
            match first_char {
                Some('+') => {
                    if opts.hide_project % 2 == 0 {
                        color.set_fg(Some(Color::Ansi256(Ansi::LIME)));
                        buf.set_color(&color)?;
                        write!(&mut buf, "{} ", word)?;
                        color.set_fg(prev_color);
                        buf.set_color(&color)?;
                    }
                }
                Some('@') => {
                    if opts.hide_context % 2 == 0 {
                        color.set_fg(Some(Color::Ansi256(Ansi::LIGHTORANGE)));
                        buf.set_color(&color)?;
                        write!(&mut buf, "{} ", word)?;
                        color.set_fg(prev_color);
                        buf.set_color(&color)?;
                    }
                }
                _ => {
                    write!(&mut buf, "{} ", word)?;
                }
            }
        }
        buf.reset()?;
        writeln!(&mut buf)?;
    }
    bufwtr.print(&buf)?;
    Ok(())
}

/// Gets path based on default location
fn get_todo_file_path() -> Result<PathBuf, Error> {
    let mut path = PathBuf::new();
    if let Some(home) = dirs::home_dir() {
        path.push(home);
    } else {
        path.push("~");
    }
    path.push("Dropbox");
    path.push("todo");
    path.push("todo.txt");
    Ok(path)
 }

// Config file todo.cfg {{{1
#[allow(dead_code)]
/// Source todo.cfg using bash
fn source_cfg_file(cfg_file_path: &str) -> Result<String, Error> { //{{{2
    let child = std::process::Command::new("/bin/bash")
        .arg("-c")
        .arg(format!("source {}; env", cfg_file_path))
        .output()?;
    String::from_utf8(child.stdout).map_err(Error::from)
}

/// Hold key value pairs for env vars
#[allow(dead_code)]
#[derive(Debug)]
struct EnvVar<'a> { //{{{2
    name: &'a str,
    value: &'a str,
}

#[allow(dead_code)]
/// Process strings into EnvVars
fn process_cfg(cfg_item: &str) -> Result<EnvVar, Error> { //{{{2
    let mut split = cfg_item.split('=').map(str::trim);
    split
        .next()
        .and_then(|name| {
            split
                .next()
                .and_then(|v| {
                    if split.next().is_some() {
                        None
                    } else {
                        Some(v)
                    }
                })
                .map(|value| EnvVar { name, value })
        })
        .ok_or_else(|| err_msg("unable to parse cfg item"))
}

// Config file toml {{{1
#[derive(Debug, Deserialize)]
/// Color settings for terminal output
struct Colors { //{{{2
    context: Option<u8>,
    project: Option<u8>,
    done: Option<u8>,
    priority: HashMap<char, u8>,
}

/// General app settings
#[derive(Debug, Deserialize)]
struct Settings { //{{{2
    date_on_add: Option<bool>,
    default_action: Option<String>,
}

/// All configuration settings
#[derive(Debug, Deserialize)]
struct Config { //{{{2
    colors: Colors,
    general: Settings,
}

/// Gets toml config file path based on default location
fn get_def_cfg_file_path() -> Result<PathBuf, Error> { //{{{2
    let mut path = PathBuf::new();
    if let Some(home) = dirs::home_dir() {
        path.push(home);
    } else {
        path.push("~");
    }
    // path.push("Dropbox");
    // path.push("todo");
    path.push("git");
    path.push("todors");
    path.push("todo.toml");
    Ok(path)
}

#[allow(dead_code)]
/// Read and process cfg from toml into Config object
fn read_config(file_path: &PathBuf) -> Result<Config, Error> { //{{{2
    use std::io::prelude::*;
    let mut config_toml = String::new();
    let mut file = std::fs::File::open(file_path)?;
    info!("Found config file at {:?}", file_path);
    file.read_to_string(&mut config_toml)?;
    let cfg: Result<Config, Error> = toml::from_str(&config_toml).map_err(Error::from);
    cfg
}

// Main {{{1
/// Entry point for main program logic
pub fn run(args: &[String]) -> Result<(), Error> { //{{{2
    let opts = args::Opt::from_iter(args);

    if !opts.quiet {
        logger::init_logger(opts.verbose);
        info!("Running with args: {:?}", args);
        info!("Parsed options:\n{:#?}", opts);
    }
    if opts.plain {
        std::env::set_var("TERM", "dumb");
    }

    match &opts.cmd {
        Some(command) => match command {
            Command::Add { todo } => info!("Adding: {:?}", todo),
            Command::Addm { todo } => info!("Adding multiple: {:?}", todo),
            Command::List { terms } => info!("Listing with terms: {:?}", terms),
            Command::Listall => info!("Listing all..."),
            Command::Addto => info!("Adding to..."),
            Command::Append { item, text } => info!("Appending: {:?} to task {}", text, item),
        },
        None => {
            info!("No command supplied; defaulting to List");
        }
    }

    // Load shell config file {{{
    // if let Some(ref cfg_file) = opts.config_file {
    //     info!("Found cfg file path: {:?}", cfg_file);
    //     if let Ok(env) = source_cfg_file(cfg_file) {
    //         let lines = env.split_whitespace();
    //         for line in lines {
    //             debug!("{:?}", process_cfg(line)?);
    //         }
    //     };
    // };
    // }}}

    // Load toml configuration file and deserialize
    let toml_file_path = get_def_cfg_file_path()?;
    let cfg: Config = read_config(&toml_file_path)?;
    debug!("{:#?}", cfg);

    let todo_file = fs::read_to_string(get_todo_file_path()?)?;
    let mut tasks: Vec<todo_txt::Task> = Vec::with_capacity(50);

    let mut file_line_ct = 0;
    for line in todo_file.lines() {
        file_line_ct += 1;
        tasks.push(todo_txt::Task::from_str(line).expect("couldn't parse string as text"));
    }
    trace!("Parsed tasks:\n{:#?}", tasks);

    let mut ctr = 0;
    let lines = todo_file
        .lines()
        .map(|ln| {
            ctr += 1;
            format!("{:0ct$} {}", ctr, ln, ct = file_line_ct.to_string().len())
        })
        .collect::<Vec<String>>();

    let bufwtr = BufferWriter::stdout(ColorChoice::Auto);
    format_buffer(&lines, bufwtr, &opts)?;
    println!("--\nTODO: {} of {} tasks shown", ctr, file_line_ct);
    Ok(())
}

// Tests {{{1
#[cfg(test)] //{{{2
mod test {
    use std::str::FromStr;

    #[test]
    fn str_to_task() { //{{{3
        let line = "x (C) 2019-12-18 Get new +pricing for +item @work due:2019-12-31";
        let task = todo_txt::Task::from_str(line).expect("error parsing task");
        assert_eq!(task.subject, "Get new +pricing for +item @work");
        assert_eq!(task.priority, 2);
        assert_eq!(task.contexts, vec!("work".to_owned()));
        assert_eq!(task.projects, vec!("item".to_owned(), "pricing".to_owned()));
        assert_eq!(task.finish_date, None);
        assert_eq!(task.due_date, Some(todo_txt::Date::from_ymd(2019, 12, 31)));
        assert_eq!(task.threshold_date, None);
    }
}
