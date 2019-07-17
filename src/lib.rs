#!/usr/bin/env bash
use args::{Command, Opt};
use errors::{Error, Result};
use failure::ResultExt;
use log::{debug, info, trace};
use regex::Regex;
use serde::Deserialize;
use std::{
    cmp::Ordering,
    fmt::{self, Display},
    fs::OpenOptions,
    io::{self, Read, Write},
    path::{Path, PathBuf},
};
use structopt::StructOpt;
use termcolor::{Color, ColorSpec, WriteColor};

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

impl Task {
    /// Create new task from string and ID
    fn new(id: usize, raw_text: &str) -> Self {
        Task {
            id,
            parsed: todo_txt::parser::task(raw_text)
                .unwrap_or_else(|_| panic!("couldn't parse into todo: '{}'", raw_text)),
            raw: raw_text.to_string(),
        }
    }

    /// Turn into blank task with same id
    fn clear(&self) -> Self {
        Task::new(self.id, "")
    }

    /// Returns true if the task is a blank line
    fn is_blank(&self) -> bool {
        self.raw == ""
    }

    /// Normalize whitespace (condense >1 space to 1) and reparse
    fn normalize_whitespace(&self) -> Self {
        Task::new(
            self.id,
            &self.raw.split_whitespace().collect::<Vec<&str>>().join(" "),
        )
    }

    /// Turn into plain string with properly padded line number
    #[allow(dead_code)]
    fn stringify(&self, total_task_ct: usize) -> impl Display {
        format!(
            "{:0ct$} {}",
            self.id,
            self.raw,
            ct = total_task_ct.to_string().len(),
        )
    }
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

impl Display for Task {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", self.id, self.raw,)
    }
}

#[derive(Debug)]
/// Store constants of ANSI 256-color code
struct Ansi;

#[allow(dead_code)]
impl Ansi {
    const BLUE: u8 = 4;
    const GREEN: u8 = 2;
    const GREY: u8 = 246;
    const HOTPINK: u8 = 198;
    const LIGHTORANGE: u8 = 215;
    const LIME: u8 = 154;
    const OLIVE: u8 = 113;
    const SKYBLUE: u8 = 111;
    const TAN: u8 = 179;
    const TURQUOISE: u8 = 37;
}

/// Get item style from preferences (or default)
fn get_colors_from_style(name: &str, ctx: &Context) -> Result<ColorSpec> {
    // TODO: build ColorSpecs for each style in the configuration and iterate once
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

/// Convert a slice of tasks to a newline-delimited string
fn tasks_to_string(tasks: &[Task], ctx: &Context) -> Result<String> {
    Ok(tasks
        .iter()
        .filter(|t| {
            if ctx.opts.remove_blank_lines {
                !t.is_blank()
            } else {
                true
            }
        })
        .map(|t| t.raw.clone())
        .collect::<Vec<String>>()
        .join("\n"))
}

/// Get string priority name in the form of `pri_x`
fn get_pri_name(pri: u8) -> Option<String> {
    match pri {
        0..=25 => {
            let mut s = String::from("pri_");
            s.push((pri + 97).into());
            Some(s)
        }
        _ => None,
    }
}

/// Format output and add color to priorities, projects and contexts
fn format_buffer(
    tasks: &[Task],
    buf: &mut termcolor::Buffer,
    ctx: &Context,
    total_task_ct: usize,
) -> Result {
    for task in tasks {
        let line = &task.raw;
        let pri = get_pri_name(task.parsed.priority).unwrap_or_default();
        let color = if task.parsed.finished {
            get_colors_from_style("done", ctx)?
        } else {
            get_colors_from_style(&pri, ctx)?
        };
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
                        buf.set_color(&get_colors_from_style("project", ctx)?)?;
                        write!(buf, "{}", word)?;
                        buf.reset()?;
                        buf.set_color(&prev_color)?;
                    }
                }
                Some('@') => {
                    if ctx.opts.hide_context % 2 == 0 {
                        buf.set_color(&get_colors_from_style("context", ctx)?)?;
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
        if task.parsed.priority < 26 || task.parsed.finished {
            buf.reset()?;
        }
        writeln!(buf)?;
    }
    Ok(())
}

/// Get output of todo.sh `list` command
pub fn get_todo_sh_output(
    argv: Option<&[&str]>,
    sort_cmd: Option<&str>,
) -> Result<std::process::Output> {
    let sort_cmd = sort_cmd.unwrap_or("sort -f -k 2");
    debug!("TODOTXT_SORT_COMMAND={}", sort_cmd);
    std::process::Command::new("todo.sh")
        .args(argv.unwrap_or_default())
        .env("TODOTXT_SORT_COMMAND", sort_cmd)
        .output()
        .context("get_todo_sh_output(): error getting command output")
        .map_err(Error::from)
}

#[derive(Debug, Deserialize)]
/// Color settings for terminal output
struct Style {
    name:      String,
    color_fg:  Option<u8>,
    color_bg:  Option<u8>,
    bold:      Option<bool>,
    intense:   Option<bool>,
    underline: Option<bool>,
}

impl Style {
    fn default(name: &str) -> Style {
        let mut default = Style {
            name:      name.into(),
            color_fg:  None,
            color_bg:  None,
            bold:      None,
            intense:   None,
            underline: None,
        };
        if name.starts_with("pri") {
            match name {
                "pri_a" => default.color_fg = Some(Ansi::HOTPINK),
                "pri_b" => default.color_fg = Some(Ansi::GREEN),
                "pri_c" => default.color_fg = Some(Ansi::BLUE),
                "pri_d" => default.color_fg = Some(Ansi::TURQUOISE),
                _ => default.color_fg = Some(Ansi::TAN),
            }
            default
        } else {
            match name {
                "project" => default.color_fg = Some(Ansi::LIME),
                "context" => default.color_fg = Some(Ansi::LIGHTORANGE),
                _ => default.color_fg = None,
            }
            default
        }
    }
}

#[derive(Debug)]
/// Wrapper that holds all current settings, args, etc.
struct Context {
    opts:     Opt,
    settings: Settings,
    styles:   Vec<Style>,
}

/// General app settings
#[derive(Debug, Deserialize)]
struct Settings {
    todo_file:      Option<String>,
    done_file:      Option<String>,
    report_file:    Option<String>,
    date_on_add:    Option<bool>,
    default_action: Option<String>,
}

/// All configuration settings from toml
#[derive(Debug, Deserialize)]
struct Config {
    general: Settings,
    styles:  Vec<Style>,
}

/// Gets toml config file in same directory as src
/// TODO: takes from $PWD, not source dir
fn get_def_cfg_file_path() -> Result<PathBuf> {
    let mut path =
        std::env::current_dir().context("get_def_cfg_file_path(): error getting current dir")?;
    path.push("todo.toml");
    Ok(path)
}

/// Read and process cfg from toml into Config object
fn read_config<P>(file_path: P) -> Result<Config>
where
    P: AsRef<Path>,
    P: std::fmt::Debug,
{
    use std::io::prelude::*;
    let mut config_toml = String::new();
    let mut file = std::fs::File::open(&file_path)
        .context(format!("could not open file {:?}", file_path))
        .map_err(Error::from)?;
    info!("Found config file at {:?}", file_path);
    file.read_to_string(&mut config_toml)?;
    toml::from_str(&config_toml)
        .context("could not convert toml config data")
        .map_err(Error::from)
}

/// Filter tasks list against terms
fn apply_filter(tasks: &mut Vec<Task>, terms: &[String]) -> Result {
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

#[allow(clippy::needless_range_loop)]
/// Delete task by line number, or delete word from task
fn delete(tasks: &mut Vec<Task>, item: usize, term: &Option<String>) -> Result<bool> {
    if let Some(t) = term {
        let re = Regex::new(t).unwrap();

        for i in 0..tasks.len() {
            let task = &tasks[i];
            if task.id == item {
                info!("Removing {:?} from {}", t, task);
                println!("{} {}", task.id, task.raw);
                if !re.is_match(&task.raw) {
                    info!("'{}' not found in task.", t);
                    println!("TODO: '{}' not found; no removal done.", t);
                    return Ok(false);
                }
                let result = re.replace_all(&task.raw, "");
                let new = Task::new(task.id, &result).normalize_whitespace();
                info!("Task after editing: {}", new.raw);
                println!("TODO: Removed '{}' from task.", t);
                println!("{}", new);
                tasks[i] = new;
            }
        }
        return Ok(true);
    }
    for i in 0..tasks.len() {
        let t = &tasks[i];
        if t.id == item {
            info!("Removing '{}' at index {}", t, i);
            if util::ask_user_yes_no(&format!("Delete '{}'?  (y/n)\n", t.raw,))? {
                let msg = format!("{}\nTODO: {} deleted.", t, t.id);
                tasks[i] = t.clear();
                println!("{}", msg);
                return Ok(true);
            }
            println!("TODO: No tasks were deleted.");
            return Ok(true);
        }
    }
    println!("TODO: No task {}.", item);
    Ok(false)
}

/// Write tasks to file
fn write_buffer<P: AsRef<Path>>(buf: &str, todo_file_path: P, append: bool) -> Result {
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(!append)
        .append(append)
        .open(&todo_file_path)?;
    write!(file, "{}", buf)?;
    let action = if append { "Appended" } else { "Wrote" };
    info!("{} tasks to file {:?}", action, todo_file_path.as_ref());
    Ok(())
}

/// Load todo.txt file and parse into Task objects.
/// If the file doesn't exist, create it.
fn get_tasks<P: AsRef<Path>>(todo_file_path: P) -> Result<Vec<Task>> {
    let mut todo_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&todo_file_path)
        .context(format!("file: {:?}", todo_file_path.as_ref()))?;
    // create string buffer and read file into it
    let mut buf = String::new();
    todo_file.read_to_string(&mut buf)?;
    let mut task_ct = 0;
    Ok(buf
        .lines()
        .map(|l| {
            task_ct += 1;
            Task::new(task_ct, l)
        })
        .collect())
}

/// Fields of `Task` we can sort by
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SortByField {
    /// Parsed body of the task
    Body,
    /// Complete date of completed task
    CompleteDate,
    /// Whether task is completed or not
    Completed,
    /// The first context
    Context,
    /// Create date if present
    CreateDate,
    /// Due date tag if present
    DueDate,
    /// Line number
    Id,
    /// Priority code (A-Z)
    Priority,
    /// The first project
    Project,
    /// The unparsed line from todo.txt file
    Raw,
    /// Threshold date tag if present
    ThresholdDate,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SortBy {
    /// Sorting criterion
    field: SortByField,
    /// Whether to reverse the sort
    reverse: bool,
}

/// Sort task list by slice of TaskSort objects
fn sort_tasks(tasks: &mut [Task], sorts: &[SortBy]) {
    tasks.sort_by(|a, b| {
        let mut cmp = Ordering::Equal;
        for sort in sorts {
            if cmp != Ordering::Equal {
                break;
            }
            cmp = match sort.field {
                SortByField::CompleteDate => a.parsed.finish_date.cmp(&b.parsed.finish_date),
                SortByField::Completed => a.parsed.finished.cmp(&b.parsed.finished),
                SortByField::Context => a.parsed.contexts.get(0).cmp(&b.parsed.contexts.get(0)),
                SortByField::CreateDate => a.parsed.create_date.cmp(&b.parsed.create_date),
                SortByField::DueDate => a.parsed.due_date.cmp(&b.parsed.due_date),
                SortByField::Id => a.id.cmp(&b.id),
                SortByField::Priority => a.parsed.priority.cmp(&b.parsed.priority),
                SortByField::Project => a.parsed.projects.get(0).cmp(&b.parsed.projects.get(0)),
                SortByField::Body => a.parsed.subject.cmp(&b.parsed.subject),
                SortByField::Raw => a.raw.cmp(&b.raw),
                SortByField::ThresholdDate => a.parsed.threshold_date.cmp(&b.parsed.threshold_date),
            };
            cmp = if sort.reverse { cmp.reverse() } else { cmp };
        }
        cmp
    })
}

/// Process new input into tasks
fn process_input(task: &mut String) -> Result {
    if task == "" {
        io::stdout().write_all(b"Add: ").unwrap();
        io::stdout().flush().unwrap();
        io::stdin().read_line(task).unwrap();
    }
    Ok(())
}

/// List tasks from todo.txt file
fn list(
    tasks: &mut Vec<Task>,
    terms: &[String],
    buf: &mut termcolor::Buffer,
    ctx: &Context,
) -> Result {
    sort_tasks(
        tasks,
        &[SortBy {
            field:   SortByField::Id,
            reverse: false,
        }],
    );
    // use for formatting line# column
    let total_task_ct = tasks.len();
    // remove blank rows
    tasks.retain(|t| !t.is_blank());
    // use for 'n of m tasks shown' message (not including blanks)
    let prefilter_len = tasks.len();
    // filter based on terms
    if !terms.is_empty() {
        info!("Listing with terms: {:?}", terms);
        apply_filter(tasks, terms)?;
    } else {
        info!("Listing without filter");
    }

    trace!("Parsed tasks:\n{:#?}", tasks);

    // fill buffer with formatted (colored) output
    format_buffer(&tasks, buf, &ctx, total_task_ct)?;

    // write footer
    write!(
        buf,
        "--\nTODO: {} of {} tasks shown\n",
        tasks.len(),
        prefilter_len,
    )?;
    Ok(())
}

/// Direct the execution of the program based on the Command in the
/// Context object
fn handle_command(ctx: &mut Context, buf: &mut termcolor::Buffer) -> Result {
    let todo_file_path = ctx.settings.todo_file.as_ref().unwrap();
    let mut tasks = get_tasks(&todo_file_path)?;
    ctx.opts.total_task_ct = tasks.len();
    match ctx.opts.cmd.clone() {
        Some(command) => match command {
            Command::Add { task } => {
                let mut task = task;
                process_input(&mut task)?;
                write_buffer(&task, &todo_file_path, true)?;
            }
            Command::Addm { tasks } => {
                let ts = tasks.join("\n");
                // TODO: join ts to existing tasks! Duh!
                write_buffer(&ts, &todo_file_path, false)?;
            }
            Command::Delete { item, term } => {
                if delete(&mut tasks, item, &term)? {
                    write_buffer(&tasks_to_string(&tasks, &ctx)?, &todo_file_path, false)?;
                    return Ok(());
                }
                std::process::exit(1)
            }
            Command::List { terms } => {
                list(&mut tasks, &terms, buf, &*ctx)?;
            }
            Command::Listall { terms } => info!("Listing all {:?}", terms),
            Command::Listpri { priorities } => info!("Listing priorities {:?}", priorities),
            Command::Addto => info!("Adding to..."),
            Command::Append { item, text } => info!("Appending: {:?} to task {}", text, item),
        },
        None => match &ctx.settings.default_action {
            Some(cmd) => match cmd.as_str() {
                "ls" | "list" => list(&mut tasks, &[], buf, &ctx)?,
                _ => panic!("Unknown command: {:?}", cmd),
            },
            None => {
                info!("No command supplied; defaulting to List");
                list(&mut tasks, &[], buf, &ctx)?;
            }
        },
    }
    Ok(())
}

/// Entry point for main program logic
pub fn run(args: &[String], buf: &mut termcolor::Buffer) -> Result {
    let opts = Opt::from_iter(args);

    if opts.long_help {
        Opt::clap().print_long_help()?;
        println!(); // add line ending
        return Ok(());
    }
    if !opts.quiet {
        logger::init_logger(opts.verbosity);
    }
    if opts.plain {
        std::env::set_var("TERM", "dumb");
    }
    info!("Running with args: {:?}", args);
    let cfg_file = opts
        .config_file
        .clone()
        .or_else(|| get_def_cfg_file_path().ok())
        .expect("could not find valid cfg file path");
    let cfg = read_config(cfg_file)?;
    let mut ctx = Context {
        opts,
        settings: cfg.general,
        styles: cfg.styles,
    };
    debug!("{:#?}", ctx);
    let todo_file_path = &ctx
        .settings
        .todo_file
        .as_ref()
        .and_then(|s| shellexpand::env(s).ok())
        .expect("couldn't get todo file path")
        .into_owned();
    ctx.settings.todo_file = Some(todo_file_path.clone());
    handle_command(&mut ctx, buf)?;
    // trace!(
    //     "todo.sh output:\n{:?}",
    //     std::str::from_utf8(&get_todo_sh_output(None, Some("sort"))?.stdout)?
    // );
    // if !buf.is_empty() {
    //     trace!(
    //         "Buffer contents:\n{:?}",
    //         std::str::from_utf8(buf.as_slice())?
    //     );
    // }
    Ok(())
}

// util :: helper functions, etc {{{1
pub mod util {
    use crate::errors::Result;
    use log::debug;
    use std::io::{stdin, stdout, Write};

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
} /* util */

// args :: Build CLI Arguments {{{1
pub mod args {
    use structopt::StructOpt;

    /// Command line options
    #[derive(Debug, StructOpt)]
    #[structopt(
        name = env!("CARGO_PKG_NAME"),
        about = env!("CARGO_PKG_DESCRIPTION"),
        setting = structopt::clap::AppSettings::DontCollapseArgsInUsage,
        setting = structopt::clap::AppSettings::VersionlessSubcommands,
    )]
    pub struct Opt {
        /// Holds count of tasks for use later in context object
        #[structopt(skip)]
        pub total_task_ct: usize,

        /// Hide context names in list output.
        ///
        /// Use twice to show context names (default).
        #[structopt(short = "@", parse(from_occurrences))]
        pub hide_context: u8,

        /// Hide project names in list output.
        ///
        /// Use twice to show project names (default).
        #[structopt(short = "+", parse(from_occurrences))]
        pub hide_project: u8,

        /// Print long help message and exit (same as --help).
        ///
        /// Shorter help message is printed with -h or `help` subcommand.
        #[structopt(short = "H")]
        pub long_help: bool,

        /// Don't preserve line numbers.
        ///
        /// Automatically remove blank lines during processing.
        #[structopt(short = "n")]
        pub remove_blank_lines: bool,

        /// Preserve line numbers when deleting tasks.
        ///
        /// Don't remove blank lines on task deletion (default).
        #[structopt(short = "N")]
        pub preserve_line_numbers: bool,

        /// Hide priority labels in list output.
        ///
        /// Use twice to show priority labels (default).
        #[structopt(short = "P", parse(from_occurrences))]
        pub hide_priority: u8,

        /// Plain mode turns off colors on the terminal.
        ///
        /// This overrides any color settings in the configuration file.
        #[structopt(short = "p")]
        pub plain: bool,

        /// Increase log verbosity (can be passed multiple times).
        ///
        /// The default verbosity is ERROR. With this flag, it is set to:
        /// {n}-v = WARN, -vv = INFO, -vvv = DEBUG, -vvvv = TRACE
        #[structopt(short = "v", parse(from_occurrences))]
        pub verbosity: u8,

        /// Quiet debug messages on the console.
        ///
        /// This overrides any verbosity (-v) settings and prevents debug
        /// messages from being shown.
        #[structopt(short = "q")]
        pub quiet: bool,

        /// Use a non-default config file to set preferences.
        ///
        /// This file is toml file and will override the default if
        /// specified on the command line. Otherwise the env var is
        /// used.
        #[structopt(
            short = "d",
            name = "CONFIG_FILE",
            env = "TODORS_CFG_FILE",
            hide_env_values = false
        )]
        pub config_file: Option<std::path::PathBuf>,

        #[structopt(subcommand)]
        pub cmd: Option<Command>,
    }

    #[derive(StructOpt, Debug, Clone)]
    pub enum Command {
        /// Add line to todo.txt file.
        #[structopt(name = "add", visible_alias = "a")]
        Add {
            #[structopt(name = "TASK")]
            /// Todo item
            ///
            /// "THING I NEED TO DO +project @context"
            task: String,
        },

        /// Add multiple lines to todo.txt file.
        #[structopt(name = "addm")]
        Addm {
            /// Todo item(s)
            ///
            /// "FIRST THING I NEED TO DO +project1 @context{n}
            /// SECOND THING I NEED TO DO +project2 @context"{n}{n}
            /// Adds FIRST THING I NEED TO DO to your todo.txt on its own line and{n}
            /// Adds SECOND THING I NEED TO DO to your todo.txt on its own line.{n}
            /// Project and context notation optional.
            #[structopt(name = "TASKS", value_delimiter = "\n")]
            tasks: Vec<String>,
        },

        /// Add line of text to any file in the todo.txt directory.
        #[structopt(name = "addto")]
        Addto,

        /// Add text to end of the item.
        #[structopt(name = "append", visible_alias = "app")]
        Append {
            /// Append text to end of this line number
            #[structopt(name = "ITEM")]
            item: usize,

            /// Text to append (quotes optional)
            #[structopt(name = "TEXT")]
            text: String,
        },

        /// Deletes the task on line ITEM of todo.txt.
        ///
        /// If TERM specified, deletes only TERM from the task
        #[structopt(name = "del", visible_alias = "rm")]
        Delete {
            /// Line number of task to delete
            #[structopt(name = "ITEM")]
            item: usize,

            /// Optional term to remove from item
            #[structopt(name = "TERM")]
            term: Option<String>,
        },

        /// Displays all tasks that contain TERM(s) sorted by priority with line
        ///
        /// Each task must match all TERM(s) (logical AND); to display
        /// tasks that contain any TERM (logical OR), use
        /// "TERM1\|TERM2\|..." (with quotes), or TERM1\\|TERM2 (unquoted).
        /// {n}Hides all tasks that contain TERM(s) preceded by a
        /// minus sign (i.e. -TERM). If no TERM specified, lists entire todo.txt.
        #[structopt(name = "list", visible_alias = "ls")]
        List {
            /// Term to search for
            #[structopt(name = "TERM")]
            terms: Vec<String>,
        },

        /// List all todos.
        #[structopt(name = "listall", visible_alias = "lsa")]
        Listall {
            /// Term to search for
            #[structopt(name = "TERM")]
            terms: Vec<String>,
        },

        /// List all tasks with priorities (optionally filtered).
        #[structopt(name = "listpri", visible_alias = "lsp")]
        Listpri {
            /// Priorities to search for
            #[structopt(name = "PRIORITY")]
            priorities: Vec<String>,
        },
    }
} /* args */
// logger :: format output of env_logger {{{1
pub mod logger {
    use chrono::Local;
    use env_logger::{fmt::Color, Env};
    use log::{self, Level};
    use std::io::Write;

    /// Initialize customized instance of env_logger
    pub fn init_logger(verbose: u8) {
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
                file = gray_style.value(record.file().unwrap_or("<unnamed>")),
                time = gray_style.value(time),
                level = level,
                line_no = gray_style.value(record.line().unwrap_or(0)),
                record_args = &record.args(),
            )
        })
        .init();
    }
} /* logger */

// errors :: custom error definitions {{{1
mod errors {
    pub use failure::Error;
    use std::result::Result as StdResult;

    pub type Result<T = ()> = StdResult<T, Error>;

    // #[derive(Debug, Fail)]
    // pub enum Error {
    //     #[fail(display = "parse error")]
    //     ParseError,
    //     #[fail(display = "error executing command")]
    //     CommandError(#[cause] std::io::Error),
    // }
    //
    // impl From<std::io::Error> for Error {
    //
    // }
} /* errors */
