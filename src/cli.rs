// #![allow(dead_code)]
// #![allow(unused_imports)]
//! Main program logic
// use regex::{self, Regex};
use crate::{
    args::{self, Command},
    util::logger,
};
use failure::{err_msg, Error};
use log::{debug, info, trace};
use serde::Deserialize;
use std::{cmp::Ordering, collections::HashMap, fs, io::Write, path::PathBuf};
use structopt::StructOpt;
use termcolor::{BufferWriter, Color, ColorChoice, ColorSpec, WriteColor};

#[derive(Debug, Eq, PartialEq, Clone)]
/// Contains parsed task data and original raw string
struct Task {
    /// Line number in todo.txt file
    id: usize,
    /// Task data parsed by todo_txt crate
    parsed: todo_txt::Task,
    /// Original unmodified text
    raw: String,
}

impl PartialOrd for Task {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Task {
    fn cmp(&self, other: &Self) -> Ordering {
        self.raw
            .to_ascii_lowercase()
            .cmp(&other.raw.to_ascii_lowercase())
    }
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
fn get_priority_color(n: u8) -> Result<ColorSpec, Error> {
    // TODO: get priority colors from config
    let mut color = ColorSpec::new();
    match n {
        0 => color.set_fg(Some(Color::Ansi256(Ansi::HOTPINK))),
        1 => color.set_fg(Some(Color::Ansi256(Ansi::GREEN))),
        2 => color
            .set_fg(Some(Color::Ansi256(Ansi::BLUE)))
            .set_bold(true),
        3 => color
            .set_fg(Some(Color::Ansi256(Ansi::TURQUOISE)))
            .set_bold(true),
        4...25 => color.set_fg(Some(Color::Ansi256(Ansi::TAN))),
        _ => unreachable!("priority `{}` out of range!", &n),
    };
    Ok(color)
}

/// Use regex to add color to priorities, projects and contexts
fn format_buffer(
    tasks: &[Task],
    buf: &mut termcolor::Buffer,
    opts: &args::Opt,
    total_task_ct: usize,
) -> Result<(), Error> {
    let mut color = ColorSpec::new();
    for task in tasks {
        let line = &task.raw;
        if task.parsed.priority < 26 {
            let color = get_priority_color(task.parsed.priority)?;
            buf.set_color(&color)?;
        }
        // write line number (id)
        write!(
            buf,
            "{:0ct$} ",
            &task.id,
            ct = total_task_ct.to_string().len()
        )?;
        // TODO: figure out how to add the proper amount of whitespace back in
        for word in line.split_whitespace() {
            let first_char = word.chars().next();
            let prev_color = color.fg().cloned();
            match first_char {
                Some('+') => {
                    if opts.hide_project % 2 == 0 {
                        color.set_fg(Some(Color::Ansi256(Ansi::LIME)));
                        buf.set_color(&color)?;
                        write!(buf, "{}", word)?;
                        buf.reset()?;
                        write!(buf, " ")?;
                        color.set_fg(prev_color);
                        buf.set_color(&color)?;
                    }
                }
                Some('@') => {
                    if opts.hide_context % 2 == 0 {
                        color.set_fg(Some(Color::Ansi256(Ansi::LIGHTORANGE)));
                        buf.set_color(&color)?;
                        write!(buf, "{}", word)?;
                        buf.reset()?;
                        write!(buf, " ")?;
                        color.set_fg(prev_color);
                        buf.set_color(&color)?;
                    }
                }
                _ => {
                    write!(buf, "{} ", word)?;
                }
            }
        }
        if task.parsed.priority < 26 {
            buf.reset()?;
        }
        writeln!(buf)?;
    }
    Ok(())
}

#[allow(dead_code)]
/// Get output of todo.sh `list` command
fn get_todo_sh_output(
    argv: Option<&[&str]>,
    sort_cmd: Option<&str>,
) -> Result<std::process::Output, Error> {
    let sort_cmd = sort_cmd.unwrap_or("sort -f -k 2");
    debug!("TODOTXT_SORT_COMMAND={}", sort_cmd);
    std::process::Command::new("todo.sh")
        .args(argv.unwrap_or_default())
        .env("TODOTXT_SORT_COMMAND", sort_cmd)
        .output()
        .map_err(Error::from)
}

/// Gets path based on default location
fn get_todo_file_path() -> Result<PathBuf, Error> {
    let mut path = PathBuf::new();
    path.push(dirs::home_dir().ok_or_else(|| err_msg("cannot find home dir"))?);
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

/// Filter tasks list against terms
fn apply_filter(tasks: &mut Vec<Task>, terms: &[String]) -> Result<(), Error> {
    tasks.retain(|t| {
        for term in terms.iter() {
            if !t.raw.contains(term) {
                return false;
            }
        }
        true
    });
    Ok(())
}

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

// Use todo_txt crate ordering (same as raw cmp?) {{{
// impl Ord for Task {
//     fn cmp(&self, other: &Self) -> Ordering {
//         if self.parsed.due_date != other.parsed.due_date {
//             return self.parsed.due_date.cmp(&other.parsed.due_date);
//         }
//
//         if self.parsed.priority != other.parsed.priority {
//             return self.parsed.priority.cmp(&other.parsed.priority).reverse();
//         }
//
//         if self.parsed.subject != other.parsed.subject {
//             return self.parsed.subject.cmp(&other.parsed.subject);
//         }
//         Ordering::Equal
//     }
// }
//}}}

/// Load todo.txt file and parse into Task objects
fn get_tasks(todo_file: PathBuf) -> Result<Vec<Task>, Error> {
    let todo_file = fs::read_to_string(todo_file)?;
    let mut task_ct = 0;
    Ok(todo_file
        .lines()
        .map(|l| {
            task_ct += 1;
            Task {
                id: task_ct,
                parsed: todo_txt::parser::task(l).expect("couldn't parse string as task"),
                raw: l.to_string(),
            }
        })
        // Remove empty lines
        .filter(|l| l.raw != "")
        .collect())
}

/// List tasks from todo.txt file
fn list(terms: &[String], buf: &mut termcolor::Buffer, opts: &args::Opt) -> Result<(), Error> {
    // Open todo.txt file
    let todo_file = get_todo_file_path()?;
    let mut tasks = get_tasks(todo_file)?;
    // tasks.sort();
    tasks.sort_by(|a, b| Ord::cmp(&a.id, &b.id));
    let task_ct = tasks.len();
    if !terms.is_empty() {
        info!("Listing with terms: {:?}", terms);
        apply_filter(&mut tasks, terms)?;
    } else {
        info!("Listing without filter");
    }

    trace!("Parsed tasks:\n{:#?}", tasks);

    // fill buffer with formatted (colored) output
    format_buffer(&tasks, buf, &opts, task_ct)?;

    // write footer
    write!(
        buf,
        "--\nTODO: {} of {} tasks shown\n",
        tasks.len(),
        task_ct
    )?;
    Ok(())
}

/// Entry point for main program logic
pub fn run(args: &[String]) -> Result<(), Error> {
    let opts = args::Opt::from_iter(args);

    if !opts.quiet {
        logger::init_logger(opts.verbosity);
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

    // create color buffer and writer
    let bufwtr = BufferWriter::stdout(ColorChoice::Auto);
    let mut buf = bufwtr.buffer();
    match &opts.cmd {
        Some(command) => match command {
            Command::Add { task } => add(task)?,
            Command::Addm { tasks } => addm(tasks)?,
            Command::List { terms } => {
                list(terms, &mut buf, &opts)?;
            }
            Command::Listall { terms } => info!("Listing all {:?}", terms),
            Command::Listpri { priorities } => info!("Listing priorities {:?}", priorities),
            Command::Addto => info!("Adding to..."),
            Command::Append { item, text } => info!("Appending: {:?} to task {}", text, item),
        },
        None => {
            info!("No command supplied; defaulting to List");
            list(&[], &mut buf, &opts)?;
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
    debug!(
        "todo.sh output:\n{:?}",
        std::str::from_utf8(&get_todo_sh_output(None, Some("sort"))?.stdout)?
    );
    if !buf.is_empty() {
        debug!(
            "Buffer contents:\n{:?}",
            std::str::from_utf8(buf.as_slice())?
        );
        // print buffer
        bufwtr.print(&buf)?;
    }
    Ok(())
}

// Tests
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
