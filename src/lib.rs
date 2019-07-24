#[macro_use]
extern crate lazy_static;

// mod args;
// use structopt::StructOpt;
mod app;
mod cli;
mod errors;
mod logger;
mod util;
use crate::{
    cli::{Command, Opt},
    errors::{Error, Result},
};
use chrono::Utc;
use failure::{err_msg, ResultExt};
use log::{debug, info, trace};
use regex::Regex;
use serde::Deserialize;
use std::{
    cmp::Ordering,
    fmt::{self, Display},
    fs::OpenOptions,
    io::{self, Read, Write},
    path::{Path, PathBuf},
    process::{exit, Command as ExtCommand, Output},
};
use termcolor::{Color, ColorSpec, WriteColor};

#[derive(Debug, Default, Eq, PartialEq, Clone)]
struct Tasks(Vec<Task>);

impl Display for Tasks {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for t in &self.0 {
            writeln!(f, "{}", t)?;
        }
        Ok(())
    }
}

impl IntoIterator for Tasks {
    type IntoIter = ::std::vec::IntoIter<Self::Item>;
    type Item = Task;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[allow(dead_code)]
impl Tasks {
    /// Add a new Task to Tasks collection
    pub fn add(mut self, new_task: Task) -> Self {
        self.0.push(new_task);
        self
    }

    /// Remove Task by id
    pub fn remove_by_id(&mut self, id: usize) -> &Self {
        for i in 0..self.0.len() {
            if self.0[i].id == id {
                self.0[i] = Task::new(id, "");
            }
        }
        self
    }

    /// Returns the number of elements in the slice
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Retain based on closure
    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(&Task) -> bool,
    {
        self.0.retain(|x| f(x));
    }

    /// Filter tasks list against terms
    fn filter_terms(&mut self, terms: &[String]) {
        self.0.retain(|t| {
            for term in terms.iter() {
                if !t.raw.contains(term) {
                    return false;
                }
            }
            true
        });
    }

    /// Sort task list by slice of TaskSort objects
    fn sort(&mut self, sorts: &[SortBy]) {
        self.0.sort_by(|a, b| {
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
                    SortByField::ThresholdDate => {
                        a.parsed.threshold_date.cmp(&b.parsed.threshold_date)
                    }
                };
                cmp = if sort.reverse { cmp.reverse() } else { cmp };
            }
            cmp
        })
    }
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

impl Task {
    /// Create new task from string and ID
    fn new<T>(id: usize, raw_text: T) -> Self
    where
        T: Into<String>,
        T: Copy,
    {
        Task {
            id,
            parsed: todo_txt::parser::task(&raw_text.into())
                .unwrap_or_else(|_| panic!("couldn't parse into todo: '{}'", raw_text.into())),
            raw: raw_text.into(),
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
    fn stringify(&self, task_ct: usize) -> impl Display {
        format!(
            "{:0ct$} {}",
            self.id,
            self.raw,
            ct = task_ct.to_string().len(),
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
fn tasks_to_string(ctx: &mut Context) -> Result<String> {
    if ctx.opts.remove_blank_lines {
        ctx.tasks.retain(|t| !t.is_blank());
    }
    Ok(ctx.tasks.to_string())
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
fn format_buffer(buf: &mut termcolor::Buffer, ctx: &Context) -> Result {
    for task in &ctx.tasks.0 {
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
            ct = ctx.task_ct.to_string().len()
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

#[derive(Debug, Default)]
/// Wrapper that holds all current settings, args, and data
/// that needs to be passed around to various functions. It takes
/// the place of "global" variables.
struct Context {
    opts:        Opt,
    settings:    Settings,
    styles:      Vec<Style>,
    tasks:       Tasks,
    task_ct:     usize,
    todo_file:   PathBuf,
    done_file:   PathBuf,
    report_file: PathBuf,
}

/// General app settings
#[derive(Debug, Deserialize, Default)]
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

#[allow(clippy::needless_range_loop)]
/// Delete task by line number, or delete word from task
fn delete(item: usize, term: &Option<String>, ctx: &mut Context) -> Result<bool> {
    if let Some(t) = term {
        let re = Regex::new(t).unwrap();

        for i in 0..ctx.tasks.len() {
            let task = &ctx.tasks.0[i];
            if task.id == item {
                info!("Removing {:?} from {}", t, task);
                println!("{} {}", task.id, task.raw);
                if !re.is_match(&task.raw) {
                    info!("'{}' not found in task.", t);
                    println!("TODO: '{}' not found; no removal done.", t);
                    return Ok(false);
                }
                let result = re.replace_all(&task.raw, "");
                let new = Task::new(task.id, &result.into_owned()).normalize_whitespace();
                info!("Task after editing: {}", new.raw);
                println!("TODO: Removed '{}' from task.", t);
                println!("{}", new);
                ctx.tasks.0[i] = new;
            }
        }
        return Ok(true);
    }
    for i in 0..ctx.tasks.len() {
        let t = &ctx.tasks.0[i];
        if t.id == item {
            info!("Removing '{}' at index {}", t, i);
            if util::ask_user_yes_no(&format!("Delete '{}'?  (y/n)\n", t.raw,))? {
                let msg = format!("{}\nTODO: {} deleted.", t, t.id);
                ctx.tasks.0[i] = t.clear();
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
fn write_buf_to_file<T>(buf: T, ctx: &Context, append: bool) -> Result
where
    T: Into<String>,
{
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(!append)
        .append(append)
        .open(&ctx.todo_file)?;
    write!(file, "{}", buf.into())?;
    if append {
        writeln!(file)?; // Add newline at end
    }
    let action = if append { "Appended" } else { "Wrote" };
    info!("{} tasks to file {:?}", action, ctx.todo_file);
    Ok(())
}

/// Load todo.txt file and parse into Task objects.
/// If the file doesn't exist, create it.
fn get_tasks(ctx: &mut Context) -> Result {
    let mut todo_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&ctx.todo_file)
        .context(format!("file: {:?}", ctx.todo_file))?;
    // create string buffer and read file into it
    let mut buf = String::new();
    todo_file.read_to_string(&mut buf)?;
    let mut task_ct = 0;
    ctx.tasks = Tasks(
        buf.lines()
            .map(|l| {
                task_ct += 1;
                Task::new(task_ct, l)
            })
            .collect(),
    );
    ctx.task_ct = task_ct;
    Ok(())
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

/// List tasks from todo.txt file
fn list(terms: &[String], buf: &mut termcolor::Buffer, ctx: &mut Context) -> Result {
    ctx.tasks.sort(&[SortBy {
        field:   SortByField::Id,
        reverse: false,
    }]);
    // remove blank rows
    ctx.tasks.retain(|t| !t.is_blank());
    // use for 'n of m tasks shown' message (not including blanks)
    let prefilter_len = ctx.tasks.len();
    // filter based on terms
    if !terms.is_empty() {
        info!("Listing with terms: {:?}", terms);
        // apply_filter(terms, ctx)?;
        ctx.tasks.filter_terms(terms);
    } else {
        info!("Listing without filter");
    }
    // fill buffer with formatted (colored) output
    format_buffer(buf, &ctx)?;
    // write footer
    write!(
        buf,
        "--\nTODO: {} of {} tasks shown\n",
        ctx.tasks.len(),
        prefilter_len,
    )?;
    Ok(())
}

/// Create task from raw input. Print confirmation and return to caller.
fn add(task: String, ctx: &mut Context) -> Result<Task> {
    let mut task = task;
    if task == "" {
        io::stdout().write_all(b"Add: ").unwrap();
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut task).unwrap();
    }
    ctx.task_ct += 1;
    if ctx.opts.date_on_add {
        let dt = Utc::today().format("%Y-%m-%d");
        task = format!("{} {}", dt, task);
    }
    let new = Task::new(ctx.task_ct, &task);
    println!("{}", new);
    println!("TODO: {} added.", new.id);
    Ok(new)
}

/// Expand shell variables in paths and write to
/// top-level variables in Context
fn expand_paths(ctx: &mut Context) -> Result {
    ctx.todo_file = ctx
        .settings
        .todo_file
        .as_ref()
        .and_then(|s| shellexpand::env(s).ok())
        .map(|s| PathBuf::from(s.into_owned()))
        .ok_or_else(|| err_msg("could not get todo file"))?;
    ctx.done_file = ctx
        .settings
        .done_file
        .as_ref()
        .and_then(|s| shellexpand::env(s).ok())
        .map(|s| PathBuf::from(s.into_owned()))
        .ok_or_else(|| err_msg("could not get todo file"))?;
    ctx.report_file = ctx
        .settings
        .report_file
        .as_ref()
        .and_then(|s| shellexpand::env(s).ok())
        .map(|s| PathBuf::from(s.into_owned()))
        .ok_or_else(|| err_msg("could not get todo file"))?;
    Ok(())
}

/// Direct the execution of the program based on the Command in the
/// Context object
fn handle_command(ctx: &mut Context, buf: &mut termcolor::Buffer) -> Result {
    expand_paths(ctx)?;
    get_tasks(ctx)?;

    // Debug print of all settings
    debug!("{:#?}", ctx.opts);
    debug!("{:#?}", ctx.settings);
    debug!("Todo file: {:?}", ctx.todo_file);
    debug!("Done file: {:?}", ctx.done_file);
    debug!("Rept file: {:?}", ctx.report_file);
    trace!("{:#?}", ctx.styles);
    trace!("{:#?}", ctx.tasks);

    match ctx.opts.cmd.clone() {
        Some(command) => match command {
            Command::Add { task } => {
                let new = add(task, ctx)?;
                write_buf_to_file(new.raw, ctx, true)?;
            }
            Command::Addm { tasks } => {
                for task in tasks {
                    let new = add(task, ctx)?;
                    write_buf_to_file(new.raw, ctx, true)?;
                }
            }
            Command::Delete { item, term } => {
                if delete(item, &term, ctx)? {
                    write_buf_to_file(tasks_to_string(ctx)?, ctx, false)?;
                    return Ok(());
                }
                exit(1)
            }
            Command::List { terms } => {
                list(&terms, buf, ctx)?;
            }
            Command::Listall { terms } => info!("Listing all {:?}", terms),
            Command::Listpri { priorities } => info!("Listing priorities {:?}", priorities),
            Command::Addto => info!("Adding to..."),
            Command::Append { item, text } => info!("Appending: {:?} to task {}", text, item),
        },
        None => match &ctx.settings.default_action {
            Some(cmd) => match cmd.as_str() {
                "ls" | "list" => list(&[], buf, ctx)?,
                _ => panic!("Unknown command: {:?}", cmd),
            },
            None => {
                info!("No command supplied; defaulting to List");
                list(&[], buf, ctx)?;
            }
        },
    }
    Ok(())
}

/// Entry point for main program logic
pub fn run(args: &[String], buf: &mut termcolor::Buffer) -> Result {
    // let opts = Opt::from_iter(args);
    let opts = cli::parse(args)?;

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
        ..Default::default()
    };
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
