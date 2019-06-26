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
use std::{cmp::Ordering, fs, io::Write, path::PathBuf};
use structopt::StructOpt;
use termcolor::{Color, ColorSpec, WriteColor};

#[derive(Debug)]
/// Wrapper that holds all current settings
struct Context {
    opts: args::Opt,
    settings: Settings,
    styles: Vec<Style>,
}

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

/// Get item style from preferences (or default)
fn get_style(name: &str, ctx: &Context) -> Result<ColorSpec, Error> {
    let default_style = Style::default(&name);
    let style = ctx
        .styles
        .iter()
        .find(|i| i.name.to_ascii_lowercase() == name)
        .unwrap_or(&default_style);
    let mut color = ColorSpec::new();
    color.set_no_reset(true);
    if let Some(fg) = style.color_fg {
        color.set_fg(Some(Color::Ansi256(fg)));
    }
    if let Some(bg) = style.color_bg {
        color.set_bg(Some(Color::Ansi256(bg)));
    }
    color.set_bold(style.bold.unwrap_or(false));
    color.set_intense(style.intense.unwrap_or(false));
    color.set_underline(style.underline.unwrap_or(false));
    Ok(color)
}

/// Use regex to add color to priorities, projects and contexts
fn format_buffer(
    tasks: &[Task],
    buf: &mut termcolor::Buffer,
    ctx: &Context,
    total_task_ct: usize,
) -> Result<(), Error> {
    for task in tasks {
        let line = &task.raw;
        let mut pri_name = String::from("pri_");
        pri_name.push((task.parsed.priority + 97) as char); // -> 0 == 'a'
        let color = get_style(&pri_name, ctx)?;
        buf.set_color(&color)?;
        // write line number (id)
        write!(
            buf,
            "{:0ct$} ",
            &task.id,
            ct = total_task_ct.to_string().len()
        )?;
        let mut words = line.split_whitespace().peekable();
        while let Some(word) = words.next() {
            let first_char = word.chars().next();
            let prev_color = color.clone();
            match first_char {
                Some('+') => {
                    if ctx.opts.hide_project % 2 == 0 {
                        buf.set_color(&get_style("project", ctx)?)?;
                        write!(buf, "{}", word)?;
                        buf.reset()?;
                        buf.set_color(&prev_color)?;
                    }
                }
                Some('@') => {
                    if ctx.opts.hide_context % 2 == 0 {
                        buf.set_color(&get_style("context", ctx)?)?;
                        write!(buf, "{}", word)?;
                        buf.reset()?;
                        buf.set_color(&prev_color)?;
                    }
                }
                _ => {
                    write!(buf, "{}", word)?;
                }
            }
            if words.peek().is_some() {
                write!(buf, " ")?;
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
struct Style {
    name: String,
    color_fg: Option<u8>,
    color_bg: Option<u8>,
    bold: Option<bool>,
    intense: Option<bool>,
    underline: Option<bool>,
}

impl Style {
    fn default(name: &str) -> Style {
        let mut default = Style {
            name: name.into(),
            color_fg: None,
            color_bg: None,
            bold: None,
            intense: None,
            underline: None,
        };
        match name {
            "project" => default.color_fg = Some(Ansi::LIME),
            "context" => default.color_fg = Some(Ansi::LIGHTORANGE),
            "pri_a" => default.color_fg = Some(Ansi::HOTPINK),
            "pri_b" => default.color_fg = Some(Ansi::GREEN),
            "pri_c" => default.color_fg = Some(Ansi::BLUE),
            "pri_d" => default.color_fg = Some(Ansi::TURQUOISE),
            "pri_x" => default.color_fg = Some(Ansi::TAN),
            _ => default.color_fg = None,
        }
        default
    }
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
    general: Settings,
    styles: Vec<Style>,
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
fn list(terms: &[String], buf: &mut termcolor::Buffer, ctx: &Context) -> Result<(), Error> {
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
    format_buffer(&tasks, buf, &ctx, task_ct)?;

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
pub fn run(args: &[String], buf: &mut termcolor::Buffer) -> Result<(), Error> {
    let opts = args::Opt::from_iter(args);

    if !opts.quiet {
        logger::init_logger(opts.verbosity);
    }
    if opts.plain {
        std::env::set_var("TERM", "dumb");
    }
    info!("Running with args: {:?}", args);
    // TODO: make this an option in case no config exists
    let toml_file_path = get_def_cfg_file_path()?;
    let cfg: Config = read_config(&toml_file_path)?;
    let ctx = Context {
        opts,
        settings: cfg.general,
        styles: cfg.styles,
    };
    debug!("{:#?}", ctx);

    match &ctx.opts.cmd {
        Some(command) => match command {
            Command::Add { task } => add(task)?,
            Command::Addm { tasks } => addm(tasks)?,
            Command::List { terms } => {
                list(terms, buf, &ctx)?;
            }
            Command::Listall { terms } => info!("Listing all {:?}", terms),
            Command::Listpri { priorities } => info!("Listing priorities {:?}", priorities),
            Command::Addto => info!("Adding to..."),
            Command::Append { item, text } => info!("Appending: {:?} to task {}", text, item),
        },
        None => {
            info!("No command supplied; defaulting to List");
            list(&[], buf, &ctx)?;
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
    trace!(
        "todo.sh output:\n{:?}",
        std::str::from_utf8(&get_todo_sh_output(None, Some("sort"))?.stdout)?
    );
    if !buf.is_empty() {
        trace!(
            "Buffer contents:\n{:?}",
            std::str::from_utf8(buf.as_slice())?
        );
    }
    Ok(())
}

#[cfg(test)] //{{{
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn str_to_task() {
        use std::str::FromStr;
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

    #[test]
    fn compare_output() {
        use termcolor::{BufferWriter, ColorChoice};
        let bufwtr = BufferWriter::stdout(ColorChoice::Auto);
        let mut buf = bufwtr.buffer();
        run(&["todors".to_string(), "list".to_string()], &mut buf).unwrap();
        let todors = std::str::from_utf8(buf.as_slice()).unwrap();
        let todo_sh_output = get_todo_sh_output(None, Some("sort")).unwrap();
        let todo_sh = std::str::from_utf8(&todo_sh_output.stdout).unwrap();
        assert_eq!(todo_sh, todors);
    }
}
