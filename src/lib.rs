use args::Command;
use errors::{Error, Result};
use failure::{err_msg, ResultExt};
use log::{debug, info, trace};
use serde::Deserialize;
use std::{
    cmp::Ordering,
    fs::{self, OpenOptions},
    io::Write,
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
    /// Returns true if the task is a blank line
    fn is_blank(&self) -> bool {
        self.raw == ""
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
        let color = get_colors_from_style(&pri, ctx)?;
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
        if task.parsed.priority < 26 {
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

/// Gets path based on default location
fn get_todo_file_path() -> Result<PathBuf> {
    let mut path =
        dirs::home_dir().ok_or_else(|| err_msg("get_todo_file_path(): cannot find home dir"))?;
    path.push("Dropbox");
    path.push("todo");
    path.push("todo.txt");
    Ok(path)
}

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
    opts: args::Opt,
    settings: Settings,
    styles: Vec<Style>,
}

/// General app settings
#[derive(Debug, Deserialize)]
struct Settings {
    todo_file: Option<String>,
    done_file: Option<String>,
    report_file: Option<String>,
    date_on_add: Option<bool>,
    default_action: Option<String>,
}

/// All configuration settings from toml
#[derive(Debug, Deserialize)]
struct Config {
    general: Settings,
    styles: Vec<Style>,
}

/// Gets toml config file in same directory as src
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

/// Add task to todo.txt file
fn add(task: &str) -> Result {
    info!("Adding {:?}", task);
    let mut file = OpenOptions::new()
        .append(true)
        .open(get_todo_file_path()?)?;
    writeln!(file, "{}", task)?;
    Ok(())
}

/// Add multiple tasks to todo.txt file
fn addm(tasks: &[String]) -> Result {
    info!("Adding multiple: {:?}", tasks);
    for task in tasks.iter() {
        add(task)?;
    }
    Ok(())
}

/// Delete task by line number, or delete word from task
fn delete(tasks: &mut Vec<Task>, item: usize, term: &Option<String>) -> Result<bool> {
    if let Some(t) = term {
        info!("Removing {:?} from item# {}", t, item);
    } else {
        for i in 0..tasks.len() {
            let t = &tasks[i];
            if t.id == item {
                info!("Removing item# {} '{}' at index {}", t.id, t.raw, i);
                if util::ask_user_yes_no(&format!("Delete '{}'?  (y/n)\n", t.raw,))? {
                    let msg = format!("{} {}\nTODO: {} deleted.", &t.id, &t.raw, &t.id);
                    tasks.remove(i);
                    println!("{}", msg);
                    return Ok(true);
                }
                println!("TODO: No tasks were deleted.");
                return Ok(true);
            }
        }
    }
    println!("TODO: No task {}.", item);
    Ok(false)
}

/// Load todo.txt file and parse into Task objects
fn get_tasks<P>(todo_file_path: P) -> Result<Vec<Task>>
where
    P: AsRef<Path>,
    P: std::fmt::Debug,
{
    let todo_file =
        fs::read_to_string(&todo_file_path).context(format!("file: {:?}", todo_file_path))?;
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
            field: SortByField::Id,
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

/// Entry point for main program logic
pub fn run(args: &[String], buf: &mut termcolor::Buffer) -> Result {
    let opts = args::Opt::from_iter(args);

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
    let ctx = Context {
        opts,
        settings: cfg.general,
        styles: cfg.styles,
    };
    debug!("{:#?}", ctx);
    let mut tasks = get_tasks(
        ctx.settings
            .todo_file
            .as_ref()
            .and_then(|s| shellexpand::env(s).ok())
            .expect("error expanding env vars in todo.txt path")
            .as_ref(),
    )?;

    match &ctx.opts.cmd {
        Some(command) => match command {
            Command::Add { task } => add(task)?,
            Command::Addm { tasks } => addm(tasks)?,
            Command::Delete { item, term } => {
                if delete(&mut tasks, *item, term)? {
                    // TODO: write tasks to file
                    return Ok(());
                }
                std::process::exit(1)
            }
            Command::List { terms } => {
                list(&mut tasks, terms, buf, &ctx)?;
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
    /** Defines all arguments and commands */
    use structopt::StructOpt;

    /// Command line options
    #[derive(Debug, StructOpt)]
    #[structopt(
    about = "Command line interface for todo.txt files",
    // Don't collapse all positionals into [ARGS]
    raw(setting = "structopt::clap::AppSettings::DontCollapseArgsInUsage"),
    // Don't print versions for each subcommand
    raw(setting = "structopt::clap::AppSettings::VersionlessSubcommands")
)]
    pub struct Opt {
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

        /// Hide priority labels in list output.
        ///
        /// Use twice to show priority labels (default).
        #[structopt(short = "P", parse(from_occurrences))]
        pub hide_priority: u8,

        /// Plain mode turns off colors
        #[structopt(short = "p")]
        pub plain: bool,

        /// Increase log verbosity (can be passed multiple times)
        ///
        /// The default verbosity is ERROR. With this flag, it is set to:{n}
        /// -v = WARN, -vv = INFO, -vvv = DEBUG, -vvvv = TRACE
        #[structopt(short = "v", parse(from_occurrences))]
        pub verbosity: u8,

        /// Quiet debug messages
        #[structopt(short = "q")]
        pub quiet: bool,

        /// Use a config file to set preferences
        #[structopt(
            short = "d",
            name = "CONFIG_FILE",
            env = "TODORS_CFG_FILE",
            hide_env_values = false
        )]
        pub config_file: Option<std::path::PathBuf>,

        /// List contents of todo.txt file
        #[structopt(subcommand)]
        pub cmd: Option<Command>,
    }

    #[derive(StructOpt, Debug)]
    pub enum Command {
        /// Add line to todo.txt file
        #[structopt(name = "add", visible_alias = "a")]
        Add {
            #[structopt(name = "TASK")]
            /// Todo item
            ///
            /// "THING I NEED TO DO +project @context"
            task: String,
        },

        /// Add multiple lines to todo.txt file
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

        /// Add line of text to any file in the todo.txt directory
        #[structopt(name = "addto")]
        Addto,

        /// Add text to end of the item
        #[structopt(name = "append", visible_alias = "app")]
        Append {
            /// Append text to end of this line number
            #[structopt(name = "ITEM")]
            item: usize,

            /// Text to append (quotes optional)
            #[structopt(name = "TEXT")]
            text: String,
        },

        /// Deletes the task on line ITEM of todo.txt
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

        /// Displays all tasks (optionally filtered by terms)
        #[structopt(name = "list", visible_alias = "ls")]
        List {
            /// Term to search for
            #[structopt(name = "TERM")]
            terms: Vec<String>,
        },

        /// List all todos
        #[structopt(name = "listall", visible_alias = "lsa")]
        Listall {
            /// Term to search for
            #[structopt(name = "TERM")]
            terms: Vec<String>,
        },

        /// List all tasks with priorities (optionally filtered)
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
