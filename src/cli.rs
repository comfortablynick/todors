//! clap-based cli
//! Methods adapted from ripgrep

#![allow(dead_code)]
use crate::{
    app::{ArgExt, *},
    errors::Result,
    long,
};
use clap::{value_t, values_t, AppSettings, SubCommand};
use log::{debug, log_enabled, trace};
use std::{convert::TryInto, path::PathBuf};

/// Command line arguments
#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct Opt {
    pub long_help:             bool,
    pub hide_context:          u8,
    pub hide_project:          u8,
    pub remove_blank_lines:    bool,
    pub preserve_line_numbers: bool,
    pub hide_priority:         u8,
    pub plain:                 bool,
    pub verbosity:             u8,
    pub quiet:                 bool,
    pub date_on_add:           bool,
    pub no_date_on_add:        bool,
    pub total_task_ct:         usize,
    pub config_file:           Option<PathBuf>,
    pub cmd:                   Option<Command>,
}

/// Subcommands
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Command {
    Add { task: String },
    Addm { tasks: Vec<String> },
    Addto,
    Append { item: usize, text: String },
    Delete { item: usize, term: Option<String> },
    List { terms: Vec<String> },
    Listall { terms: Vec<String> },
    Listpri { priorities: Vec<String> },
}

fn tf(args: &mut Vec<Arg>) {
    let arg = Arg::flag("test", "t", None, "FILE")
        .hidden(false)
        .overrides_with("itself");
    args.push(arg);
}

fn flag_verbosity(args: &mut Vec<CliArg>) {
    const SHORT: &str = "Increase log verbosity printed to console.";
    const LONG: &str = long!(
        "\
Increase log verbosity printed to console. Log verbosity increases
each time the flag is found.

For example: -v, -vv, -vvv

The quiet flag -q will override this setting and will silence log output."
    );

    let arg = CliArg::switch("verbosity", "v", None)
        .help(SHORT)
        .long_help(LONG)
        .multiple();
    args.push(arg);
}

fn flag_quiet(args: &mut Vec<CliArg>) {
    const SHORT: &str = "Quiet debug messages.";
    const LONG: &str = long!(
        "\
Quiet debug messages on console. Overrides verbosity (-v) setting.

The arguments -vvvq will produce no console debug output."
    );

    let arg = CliArg::switch("quiet", "q", None)
        .help(SHORT)
        .long_help(LONG)
        .overrides("verbosity");
    args.push(arg);
}

fn flag_plain(args: &mut Vec<CliArg>) {
    const SHORT: &str = "Plain mode to turn off colors.";
    const LONG: &str = long!(
        "\
Plain mode turns off colors and overrides environment settings
that control terminal colors. Color settings in config will
have no effect."
    );

    let arg = CliArg::switch("plain", "p", None)
        .help(SHORT)
        .long_help(LONG);
    args.push(arg);
}

fn flag_preserve_line_numbers(args: &mut Vec<CliArg>) {
    const SHORT: &str = "Preserve line (task) numbers.";
    const LONG: &str = long!(
        "\
Preserve line (task) numbers. This allows consistent access of the
tasks by the same id each time. When a task is deleted, it will
remain blank.
        "
    );

    let arg = CliArg::switch("preserve_line_numbers", "N", None)
        .help(SHORT)
        .long_help(LONG)
        .overrides("remove_blank_lines");
    args.push(arg);
}

fn flag_remove_blank_lines(args: &mut Vec<CliArg>) {
    const SHORT: &str = "Don't preserve line (task) numbers";
    const LONG: &str = long!(
        "\
Don't preserve line (task) numbers. Opposite of -N. When a task is
deleted, the following tasks will be moved up one line."
    );

    let arg = CliArg::switch("remove_blank_lines", "n", None)
        .help(SHORT)
        .long_help(LONG);
    args.push(arg);
}

fn flag_hide_context(args: &mut Vec<CliArg>) {
    const SHORT: &str = "Hide task contexts from output.";
    const LONG: &str = long!(
        "\
Hide task contexts from output. Use twice to show contexts, which
is the default."
    );

    let arg = CliArg::switch("hide_context", "@", None)
        .help(SHORT)
        .long_help(LONG);
    args.push(arg);
}

fn flag_hide_project(args: &mut Vec<CliArg>) {
    const SHORT: &str = "Hide task projects from output.";
    const LONG: &str = long!(
        "\
Hide task projects from output. Use twice to show projects, which
is the default."
    );

    let arg = CliArg::switch("hide_project", "+", None)
        .help(SHORT)
        .long_help(LONG);
    args.push(arg);
}

fn flag_hide_priority(args: &mut Vec<CliArg>) {
    const SHORT: &str = "Hide task priorities from output.";
    const LONG: &str = long!(
        "\
Hide task priorities from output. Use twice to show priorities, which
is the default."
    );

    let arg = CliArg::switch("hide_priority", "P", None)
        .help(SHORT)
        .long_help(LONG);
    args.push(arg);
}

fn flag_date_on_add(args: &mut Vec<CliArg>) {
    const SHORT: &str = "Prepend current date to new task";
    const LONG: &str = long!("Prepend current date to new task");
    let arg = CliArg::switch("date_on_add", "t", None)
        .help(SHORT)
        .long_help(LONG)
        .overrides("no_date_on_add");
    args.push(arg);
}

fn flag_no_date_on_add(args: &mut Vec<CliArg>) {
    const SHORT: &str = "Don't prepend current date to new task";
    const LONG: &str = long!("Don't prepend current date to new task");
    let arg = CliArg::switch("no_date_on_add", "T", None)
        .help(SHORT)
        .long_help(LONG)
        .overrides("date_on_add");
    args.push(arg);
}

fn flag_config_file(args: &mut Vec<CliArg>) {
    const SHORT: &str = "Location of config toml file.";
    const LONG: &str = long!(
        "\
Location of toml config file. Various options can be set, including 
colors and styles."
    );

    let arg = CliArg::flag("config", "d", None, "CONFIG_FILE")
        .help(SHORT)
        .long_help(LONG)
        .env("TODORS_CFG_FILE");
    args.push(arg);
}

fn command_list(cmds: &mut Vec<App>) {
    let cmd = SubCommand::with_name("list")
        .alias("ls")
        .about("Displays all tasks (optionally filtered by terms)")
        .arg(
            Arg::with_name("terms")
                .help("Term(s) to filter task list by")
                .takes_value(true)
                .value_name("TERM")
                .multiple(true),
        );
    cmds.push(cmd);
}

fn command_add(cmds: &mut Vec<App>) {
    let cmd = SubCommand::with_name("add")
        .alias("a")
        .about("Add task to todo.txt file")
        .arg(
            Arg::with_name("task")
                .help("Todo item")
                .long_help("THING I NEED TO DO +project @context")
                .takes_value(true)
                .value_name("TASK")
                .required(true),
        );
    cmds.push(cmd);
}

fn command_addm(cmds: &mut Vec<App>) {
    let cmd = SubCommand::with_name("addm")
        .about("Add multiple lines to todo.txt file")
        .arg(
            Arg::with_name("tasks")
                .help("Todo items (one on each line)")
                .long_help(
                    "\"FIRST THING I NEED TO DO +project1 @context
SECOND THING I NEED TO DO +project2 @context\"

Adds FIRST THING I NEED TO DO on its own line and SECOND THING I NEED TO DO on its own line.
Project and context notation optional.",
                )
                .takes_value(true)
                .value_name("TASKS")
                .value_delimiter("\n")
                .required(true),
        );
    cmds.push(cmd);
}

fn command_del(cmds: &mut Vec<App>) {
    let cmd = SubCommand::with_name("del")
        .alias("rm")
        .about("Deletes the task on line ITEM of todo.txt")
        .long_about("If TERM specified, deletes only TERM from the task")
        .arg(
            Arg::with_name("item")
                .value_name("ITEM")
                .help("Line number of task to delete")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("term")
                .value_name("TERM")
                .help("Optional term to remove from item")
                .takes_value(true),
        );
    cmds.push(cmd);
}

pub fn base_args() -> Vec<CliArg> {
    let mut args = vec![];
    flag_verbosity(&mut args);
    flag_quiet(&mut args);
    flag_plain(&mut args);
    flag_preserve_line_numbers(&mut args);
    flag_remove_blank_lines(&mut args);
    flag_hide_context(&mut args);
    flag_hide_project(&mut args);
    flag_hide_priority(&mut args);
    flag_date_on_add(&mut args);
    flag_no_date_on_add(&mut args);
    flag_config_file(&mut args);
    args
}

pub fn commands() -> Vec<App> {
    let mut cmds = vec![];
    command_add(&mut cmds);
    command_addm(&mut cmds);
    command_list(&mut cmds);
    command_del(&mut cmds);
    cmds
}

pub fn base_app() -> App {
    let mut app = App::new(env!("CARGO_PKG_NAME"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        //
        // settings
        .setting(AppSettings::DontCollapseArgsInUsage)
        .setting(AppSettings::VersionlessSubcommands)
        .setting(AppSettings::DeriveDisplayOrder)
        .setting(AppSettings::AllArgsOverrideSelf)
        .setting(AppSettings::UnifiedHelpMessage);

    for arg in base_args() {
        app = app.arg(arg.claparg);
    }
    app = app.subcommands(commands());
    app
}

#[allow(clippy::cognitive_complexity)]
pub fn parse() -> Result<Opt> {
    let cli = base_app().get_matches();
    let mut opt = Opt::default();
    // set fields
    opt.hide_context = value_t!(cli, "hide_context", u8).unwrap_or(0);
    opt.hide_project = value_t!(cli, "hide_project", u8).unwrap_or(0);
    opt.hide_priority = value_t!(cli, "hide_priority", u8).unwrap_or(0);
    opt.remove_blank_lines = cli.is_present("remove_blank_lines");
    opt.preserve_line_numbers = cli.is_present("preserve_line_numbers");
    opt.plain = cli.is_present("plain");
    opt.quiet = cli.is_present("quiet");
    opt.verbosity = cli.occurrences_of("verbosity").try_into().unwrap();
    opt.date_on_add = cli.is_present("date_on_add");
    opt.no_date_on_add = cli.is_present("no_date_on_add");
    opt.config_file = value_t!(cli, "config", std::path::PathBuf).ok();

    match cli.subcommand() {
        ("add", Some(arg)) => {
            opt.cmd = Some(Command::Add {
                task: value_t!(arg.value_of("task"), String).unwrap(),
            });
        }
        ("addm", Some(args)) => {
            opt.cmd = Some(Command::Addm {
                tasks: values_t!(args.values_of("tasks"), String).unwrap(),
            });
        }
        ("del", Some(args)) => {
            opt.cmd = Some(Command::Delete {
                item: value_t!(args.value_of("item"), usize).unwrap(),
                term: value_t!(args.value_of("term"), String).ok(),
            });
        }
        ("list", Some(args)) => {
            opt.cmd = Some(Command::List {
                terms: values_t!(args.values_of("terms"), String).unwrap_or_default(),
            });
        }
        ("", None) => (),
        _ => unreachable!(),
    }

    debug!("{:#?}", opt);

    if log_enabled!(log::Level::Trace) {
        for m in cli.args {
            trace!("{:#?}", m);
        }
    }
    Ok(opt)
}
