/** Main program logic */
use crate::{
    args::{self, Command},
    util::{
        error::{err, AppError},
        logger,
    },
};
use log::{debug, info, trace};
use regex::{self, Regex};
use std::{
    fs,
    io::{self, Write},
    path::PathBuf,
    str::FromStr,
};
use structopt::StructOpt;
use termcolor::{BufferWriter, Color, ColorChoice, ColorSpec, WriteColor};

/// Store constants of ANSI 256-color codes
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
fn get_priority_color(c: char) -> Result<ColorSpec, io::Error> {
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
        _ => err!("color for priority `{}` not found!", &c),
    };
    Ok(color)
}

/// Use regex to add color to priorities, projects and contexts
fn format_buffer(s: &[String], bufwtr: BufferWriter, opts: &args::Opt) -> Result<(), AppError> {
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
                    .ok_or("error getting priority")?,
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
fn get_todo_file_path() -> Result<PathBuf, AppError> {
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

/// Gets toml config file path based on default location
fn get_def_cfg_file_path() -> Result<PathBuf, AppError> {
    let mut path = PathBuf::new();
    if let Some(home) = dirs::home_dir() {
        path.push(home);
    } else {
        path.push("~");
    }
    path.push("Dropbox");
    path.push("todo");
    path.push("todo.toml");
    Ok(path)
}

/// Source todo.cfg using bash
fn source_cfg_file(cfg_file_path: &str) -> Result<String, AppError> {
    let child = std::process::Command::new("/bin/bash")
        .arg("-c")
        .arg(format!("source {}; env", cfg_file_path))
        .output()?;
    String::from_utf8(child.stdout).map_err(AppError::from)
}

/// Hold key value pairs for env vars
#[derive(Debug)]
struct EnvVar<'a> {
    name: &'a str,
    value: &'a str,
}

fn read_config(file_path: &PathBuf) -> Result<(), AppError> {
    use std::io::prelude::*;
    let mut config_toml = String::new();
    let mut file = std::fs::File::open(file_path)?;
    file.read_to_string(&mut config_toml)?;
    info!("{}", config_toml);
    Ok(())
}

/// Process strings into EnvVars
fn process_cfg(cfg_item: &str) -> Result<EnvVar, AppError> {
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
        .ok_or_else(|| AppError {
            kind: String::from("parse"),
            message: String::from("unable to parse cfg item"),
        })
}

/// Entry point for main program logic
pub fn run(args: &[String]) -> Result<(), AppError> {
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

    if let Some(ref cfg_file) = opts.config_file {
        info!("Found cfg file path: {:?}", cfg_file);
        if let Ok(env) = source_cfg_file(cfg_file) {
            let lines = env.split_whitespace();
            for line in lines {
                debug!("{:?}", process_cfg(line)?);
            }
        };
    };

    let toml_file_path = get_def_cfg_file_path()?;
    read_config(&toml_file_path)?;

    // read_config()
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

#[cfg(test)]
mod test {
    use std::str::FromStr;

    #[test]
    fn str_to_task() {
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
