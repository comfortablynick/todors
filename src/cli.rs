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
use std::{fs, io::Write, path::PathBuf};
use structopt::StructOpt;
use termcolor::{BufferWriter, Color, ColorChoice, ColorSpec, WriteColor};

#[derive(Debug)]
/// Contains parsed task data and original raw string
struct Task {
    /// Line number in todo.txt file
    id: usize,
    /// Task data parsed by todo_txt crate
    parsed: Option<todo_txt::Task>,
    /// Original unmodified text
    raw: String,
}

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
fn get_priority_color(c: char) -> Result<ColorSpec, Error> {
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
fn format_buffer(
    tasks: &[Task],
    buf: &mut termcolor::Buffer,
    opts: &args::Opt,
) -> Result<(), Error> {
    lazy_static! {
        static ref RE_PRIORITY: Regex = Regex::new(r"(?m)\(([A-Z])\).*$").unwrap();
    }
    // let mut buf = bufwtr.buffer();
    let mut color = ColorSpec::new();
    for task in tasks {
        let line = &task.raw;
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
        // write line number (id)
        write!(
            buf,
            "{:0ct$} ",
            &task.id,
            ct = tasks.len().to_string().len()
        )?;
        for word in line.split_whitespace() {
            let first_char = word.chars().next();
            let prev_color = color.fg().cloned();
            match first_char {
                Some('+') => {
                    if opts.hide_project % 2 == 0 {
                        color.set_fg(Some(Color::Ansi256(Ansi::LIME)));
                        buf.set_color(&color)?;
                        write!(buf, "{} ", word)?;
                        color.set_fg(prev_color);
                        buf.set_color(&color)?;
                    }
                }
                Some('@') => {
                    if opts.hide_context % 2 == 0 {
                        color.set_fg(Some(Color::Ansi256(Ansi::LIGHTORANGE)));
                        buf.set_color(&color)?;
                        write!(buf, "{} ", word)?;
                        color.set_fg(prev_color);
                        buf.set_color(&color)?;
                    }
                }
                _ => {
                    write!(buf, "{} ", word)?;
                }
            }
        }
        buf.reset()?;
        writeln!(buf)?;
    }
    // bufwtr.print(&buf)?;
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

// todo.cfg {{{
#[allow(dead_code)]
/// Source todo.cfg using bash
fn source_cfg_file(cfg_file_path: &str) -> Result<String, Error> {
    let child = std::process::Command::new("/bin/bash")
        .arg("-c")
        .arg(format!("source {}; env", cfg_file_path))
        .output()?;
    String::from_utf8(child.stdout).map_err(Error::from)
}

/// Hold key value pairs for env vars
#[allow(dead_code)]
#[derive(Debug)]
struct EnvVar<'a> {
    name: &'a str,
    value: &'a str,
}

#[allow(dead_code)]
/// Process strings into EnvVars
fn process_cfg(cfg_item: &str) -> Result<EnvVar, Error> {
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
} //}}}
  // todo.toml {{{
#[derive(Debug, Deserialize)]
/// Color settings for terminal output
struct Colors {
    context: Option<u8>,
    project: Option<u8>,
    done: Option<u8>,
    priority: HashMap<char, u8>,
}

/// General app settings
#[derive(Debug, Deserialize)]
struct Settings {
    date_on_add: Option<bool>,
    default_action: Option<String>,
}

/// All configuration settings
#[derive(Debug, Deserialize)]
struct Config {
    colors: Colors,
    general: Settings,
}

/// Gets toml config file path based on default location
fn get_def_cfg_file_path() -> Result<PathBuf, Error> {
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
fn read_config(file_path: &PathBuf) -> Result<Config, Error> {
    use std::io::prelude::*;
    let mut config_toml = String::new();
    let mut file = std::fs::File::open(file_path)?;
    info!("Found config file at {:?}", file_path);
    file.read_to_string(&mut config_toml)?;
    let cfg: Result<Config, Error> = toml::from_str(&config_toml).map_err(Error::from);
    cfg
}
//}}}
/// Add task to todo.txt file
fn add(task: &str) -> Result<(), Error> {
    info!("Adding {:?}", task);
    Ok(())
}

/// Add multiple tasks to todo.txt file
fn addm(tasks: &[String]) -> Result<(), Error> {
    info!("Adding multiple: {:?}", tasks);
    for task in tasks.iter() {
        add(task)?;
    }
    Ok(())
}

/// List tasks from todo.txt file
fn list(terms: Option<&[String]>, opts: &args::Opt) -> Result<(), Error> {
    // TODO: remove after structopt update to allow Option<Vec<T>>
    if let Some(t) = terms {
        // TODO: handle filter by TERMS
        info!("Listing with terms: {:?}", t);
    } else {
        info!("Listing without filter");
    }
    // Open todo.txt file
    let todo_file = fs::read_to_string(get_todo_file_path()?)?;
    let mut ctr = 0;
    // Get non-empty lines from file
    let tasks: Vec<Task> = todo_file
        .lines()
        .filter(|l| *l != "")
        .map(|l| {
            ctr += 1;
            Task {
                id: ctr,
                parsed: todo_txt::parser::task(l).ok(),
                raw: l.to_string(),
            }
        })
        .collect();

    trace!("Parsed tasks:\n{:#?}", tasks);

    let bufwtr = BufferWriter::stdout(ColorChoice::Auto);
    let mut buf = bufwtr.buffer();
    // fill buffer with formatted (colored) output
    format_buffer(&tasks, &mut buf, &opts)?;
    write!(buf, "--\nTODO: {} of {} tasks shown\n", ctr, tasks.len())?;
    // print buffer
    bufwtr.print(&buf)?;
    Ok(())
}

/// Entry point for main program logic
pub fn run(args: &[String]) -> Result<(), Error> {
    let opts = args::Opt::from_iter(args);

    if !opts.quiet {
        logger::init_logger(opts.verbose);
        info!("Running with args: {:?}", args);
        info!("Parsed options:\n{:#?}", opts);
    }
    if opts.plain {
        std::env::set_var("TERM", "dumb");
    }

    // Load toml configuration file and deserialize
    let toml_file_path = get_def_cfg_file_path()?;
    let cfg: Config = read_config(&toml_file_path)?;
    debug!("{:#?}", cfg);

    match &opts.cmd {
        Some(command) => match command {
            Command::Add { task } => add(task)?,
            Command::Addm { tasks } => addm(tasks)?,
            // TODO: see if structopt is handling Option<Vec>
            Command::List { terms } => {
                if terms.is_empty() {
                    list(None, &opts)?;
                } else {
                    list(Some(terms), &opts)?;
                }
            }
            Command::Listall => info!("Listing all..."),
            Command::Addto => info!("Adding to..."),
            Command::Append { item, text } => info!("Appending: {:?} to task {}", text, item),
        },
        None => {
            info!("No command supplied; defaulting to List");
            list(None, &opts)?;
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
    Ok(())
}

// Tests {{{
#[cfg(test)]
mod test {
    //{{{
    use std::str::FromStr;

    #[test]
    fn str_to_task() {
        //{{{
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
