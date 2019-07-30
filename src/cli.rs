//! Define cli and methods used globally
//! Some methods adapted from ripgrep and cargo

use crate::actions::{add, delete, list};
pub use crate::{
    app::{AppExt, ArgExt},
    config::Context,
    long,
};
pub use clap::{
    app_from_crate, crate_authors, crate_description, crate_name, crate_version, value_t, values_t,
    AppSettings, ArgSettings,
};
pub use failure::{err_msg, Error, ResultExt};
pub use log::{debug, info, log_enabled, trace};
use std::result::Result as StdResult;
pub use std::{
    convert::TryInto,
    path::{Path, PathBuf},
    process::{exit, Command as ExtCommand, Output},
};
pub type Result<T = ()> = StdResult<T, Error>;

pub type Arg = clap::Arg<'static>;
pub type App = clap::App<'static>;

/// Command line arguments
#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct Opt {
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
    pub config_file:           Option<PathBuf>,
    pub cmd:                   Option<Command>,
}

/// Subcommands
#[allow(dead_code)]
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

fn flag_verbose(args: &mut Vec<Arg>) {
    const SHORT: &str = "Increase log verbosity printed to console.";
    const LONG: &str = long!(
        "\
Increase log verbosity printed to console. Log verbosity increases
each time the flag is found.

For example: -v, -vv, -vvv

The quiet flag -q will override this setting and will silence log output."
    );
    args.push(
        Arg::flag("verbose", 'v')
            .setting(ArgSettings::MultipleOccurrences)
            .help(SHORT)
            .long_help(LONG),
    );
}

fn flag_quiet(args: &mut Vec<Arg>) {
    const SHORT: &str = "Quiet debug messages.";
    const LONG: &str = long!(
        "\
Quiet debug messages on console. Overrides verbosity (-v) setting.

The arguments -vvvq will produce no console debug output."
    );
    args.push(
        Arg::flag("quiet", 'q')
            .help(SHORT)
            .long_help(LONG)
            .overrides_with("verbosity"),
    );
}

fn flag_plain(args: &mut Vec<Arg>) {
    const SHORT: &str = "Plain mode to turn off colors.";
    const LONG: &str = long!(
        "\
Plain mode turns off colors and overrides environment settings
that control terminal colors. Color settings in config will
have no effect."
    );
    args.push(Arg::flag("plain", 'p').help(SHORT).long_help(LONG));
}

fn flag_preserve_line_numbers(args: &mut Vec<Arg>) {
    const SHORT: &str = "Preserve line (task) numbers.";
    const LONG: &str = long!(
        "\
Preserve line (task) numbers. This allows consistent access of the
tasks by the same id each time. When a task is deleted, it will
remain blank.
        "
    );
    args.push(
        Arg::flag("preserve-line-numbers", 'N')
            .help(SHORT)
            .long_help(LONG)
            .overrides_with("remove-blank-lines"),
    );
}

fn flag_remove_blank_lines(args: &mut Vec<Arg>) {
    const SHORT: &str = "Don't preserve line (task) numbers";
    const LONG: &str = long!(
        "\
Don't preserve line (task) numbers. Opposite of -N. When a task is
deleted, the following tasks will be moved up one line."
    );
    args.push(
        Arg::flag("remove-blank-lines", 'n')
            .help(SHORT)
            .long_help(LONG),
    );
}

fn flag_hide_context(args: &mut Vec<Arg>) {
    const SHORT: &str = "Hide task contexts from output.";
    const LONG: &str = long!(
        "\
Hide task contexts from output. Use twice to unhide contexts, which
returns to the default behavior of showing contexts."
    );
    args.push(Arg::flag("hide-context", '@').help(SHORT).long_help(LONG));
}

fn flag_hide_project(args: &mut Vec<Arg>) {
    const SHORT: &str = "Hide task projects from output.";
    const LONG: &str = long!(
        "\
Hide task projects from output. Use twice to unhide projects, which
returns to the default behavior of showing projects."
    );
    args.push(Arg::flag("hide-project", '+').help(SHORT).long_help(LONG));
}

fn flag_hide_priority(args: &mut Vec<Arg>) {
    const SHORT: &str = "Hide task priorities from output.";
    const LONG: &str = long!(
        "\
Hide task priorities from output. Use twice to show priorities, which
returns to the default behavior of showing priorities."
    );

    args.push(Arg::flag("hide-priority", 'P').help(SHORT).long_help(LONG));
}

fn flag_date_on_add(args: &mut Vec<Arg>) {
    const SHORT: &str = "Prepend current date to new task";
    const LONG: &str = long!("Prepend current date to new task");
    args.push(
        Arg::flag("date-on-add", 't')
            .help(SHORT)
            .long_help(LONG)
            .overrides_with("no-date-on-add"),
    );
}

fn flag_no_date_on_add(args: &mut Vec<Arg>) {
    const SHORT: &str = "Don't prepend current date to new task";
    const LONG: &str = long!("Don't prepend current date to new task");
    args.push(
        Arg::flag("no-date-on-add", 'T')
            .help(SHORT)
            .long_help(LONG)
            .overrides_with("date-on-add"),
    );
}

fn opt_config_file(args: &mut Vec<Arg>) {
    const SHORT: &str = "Location of config toml file.";
    const LONG: &str = long!(
        "\
Location of toml config file. Various options can be set, including 
colors and styles."
    );
    args.push(
        Arg::with_name("config-file")
            .short('d')
            .help(SHORT)
            .long_help(LONG)
            .takes_value(true)
            .value_name("CONFIG_FILE")
            .env("TODORS_CFG_FILE"),
    );
}

fn base_args() -> Vec<Arg> {
    let mut args = vec![];
    flag_verbose(&mut args);
    flag_quiet(&mut args);
    flag_plain(&mut args);
    flag_preserve_line_numbers(&mut args);
    flag_remove_blank_lines(&mut args);
    flag_hide_context(&mut args);
    flag_hide_project(&mut args);
    flag_hide_priority(&mut args);
    flag_date_on_add(&mut args);
    flag_no_date_on_add(&mut args);
    opt_config_file(&mut args);
    args
}

fn commands() -> Vec<App> {
    let mut cmds = vec![];
    add::command_add(&mut cmds);
    add::command_addm(&mut cmds);
    list::command_list(&mut cmds);
    list::command_listall(&mut cmds);
    delete::command_del(&mut cmds);
    cmds
}

fn build_app() -> App {
    const TEMPLATE: &str = "\
{bin} {version}
{author}
{about}

USAGE: {usage}

OPTIONS:
{unified}

ACTIONS:
{subcommands}";

    // clap is expecting static strings, so we need to trick it with lazy_static
    lazy_static! {
        static ref USAGE: String = format!("{} [OPTIONS] [ACTIONS]", env!("CARGO_PKG_NAME"));
    }

    let mut app = app_from_crate!() // use Cargo.toml fields
        // settings
        .setting(AppSettings::DontCollapseArgsInUsage)
        .setting(AppSettings::VersionlessSubcommands)
        .setting(AppSettings::DeriveDisplayOrder)
        .setting(AppSettings::AllArgsOverrideSelf)
        .setting(AppSettings::UnifiedHelpMessage)
        .override_usage(USAGE.as_str())
        .help_template(TEMPLATE)
        .max_term_width(100);

    for arg in base_args() {
        app = app.arg(arg);
    }
    app = app.subcommands(commands());
    app
}

/// Parse the clap matches into Command.
/// Will return an error if required arguments are missing or invalid.
fn handle_subcommand(cmd: (&str, Option<&clap::ArgMatches>), opt: &mut Opt) -> Result {
    match cmd {
        ("add", Some(arg)) => {
            opt.cmd = Some(Command::Add {
                task: value_t!(arg.value_of("task"), String)?,
            });
        }
        ("addm", Some(args)) => {
            opt.cmd = Some(Command::Addm {
                tasks: values_t!(args.values_of("tasks"), String)?,
            });
        }
        ("del", Some(args)) => {
            opt.cmd = Some(Command::Delete {
                item: value_t!(args.value_of("item"), usize)?,
                term: value_t!(args.value_of("term"), String).ok(),
            });
        }
        ("list", Some(args)) => {
            opt.cmd = Some(Command::List {
                terms: values_t!(args.values_of("terms"), String).unwrap_or_default(),
            });
        }
        ("listall", Some(args)) => {
            opt.cmd = Some(Command::Listall {
                terms: values_t!(args.values_of("terms"), String).unwrap_or_default(),
            });
        }
        ("", None) => (),
        _ => unreachable!(),
    }
    Ok(())
}

/// Parse clap matches into Opt object.
/// The result will now be decoupled from clap, so it isn't needed elsewhere.
pub fn parse<I>(arg_iter: I) -> Result<Opt>
where
    I: IntoIterator,
    I::Item: Into<std::ffi::OsString> + Clone,
{
    let cli = build_app().get_matches_from(arg_iter);
    let mut opt = Opt::default();
    // set fields
    opt.hide_context = value_t!(cli, "hide-context", u8).unwrap_or(0);
    opt.hide_project = value_t!(cli, "hide-project", u8).unwrap_or(0);
    opt.hide_priority = value_t!(cli, "hide-priority", u8).unwrap_or(0);
    opt.remove_blank_lines = cli.is_present("remove-blank-lines");
    opt.preserve_line_numbers = cli.is_present("preserve-line-numbers");
    opt.plain = cli.is_present("plain");
    opt.quiet = cli.is_present("quiet");
    opt.verbosity = cli.occurrences_of("verbose").try_into().unwrap();
    opt.date_on_add = cli.is_present("date-on-add");
    opt.no_date_on_add = cli.is_present("no-date-on-add");
    opt.config_file = value_t!(cli, "config-file", std::path::PathBuf).ok();

    handle_subcommand(cli.subcommand(), &mut opt)?;

    debug!("{:#?}", opt);

    if log_enabled!(log::Level::Trace) {
        for m in cli.args {
            trace!("{:#?}", m);
        }
    }
    Ok(opt)
}
