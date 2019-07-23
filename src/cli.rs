//! clap-based cli
//! Methods adapted from ripgrep

use crate::{app::ArgExt, errors::Result, long};
use clap::{
    app_from_crate, crate_authors, crate_description, crate_name, crate_version, value_t, values_t,
    AppSettings, SubCommand,
};
use log::{debug, log_enabled, trace};
use std::{convert::TryInto, path::PathBuf};

type Arg = clap::Arg<'static, 'static>;
type App = clap::App<'static, 'static>;

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
    pub total_task_ct:         usize,
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
        Arg::flag("verbose", "v")
            .help(SHORT)
            .long_help(LONG)
            .multiple(true),
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
        Arg::flag("quiet", "q")
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
    args.push(Arg::flag("plain", "p").help(SHORT).long_help(LONG));
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
        Arg::flag("preserve-line-numbers", "N")
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
        Arg::flag("remove-blank-lines", "n")
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
    args.push(Arg::flag("hide-context", "@").help(SHORT).long_help(LONG));
}

fn flag_hide_project(args: &mut Vec<Arg>) {
    const SHORT: &str = "Hide task projects from output.";
    const LONG: &str = long!(
        "\
Hide task projects from output. Use twice to unhide projects, which
returns to the default behavior of showing projects."
    );
    args.push(Arg::flag("hide-project", "+").help(SHORT).long_help(LONG));
}

fn flag_hide_priority(args: &mut Vec<Arg>) {
    const SHORT: &str = "Hide task priorities from output.";
    const LONG: &str = long!(
        "\
Hide task priorities from output. Use twice to show priorities, which
returns to the default behavior of showing priorities."
    );

    args.push(Arg::flag("hide-priority", "P").help(SHORT).long_help(LONG));
}

fn flag_date_on_add(args: &mut Vec<Arg>) {
    const SHORT: &str = "Prepend current date to new task";
    const LONG: &str = long!("Prepend current date to new task");
    args.push(
        Arg::flag("date-on-add", "t")
            .help(SHORT)
            .long_help(LONG)
            .overrides_with("no-date-on-add"),
    );
}

fn flag_no_date_on_add(args: &mut Vec<Arg>) {
    const SHORT: &str = "Don't prepend current date to new task";
    const LONG: &str = long!("Don't prepend current date to new task");
    args.push(
        Arg::flag("no-date-on-add", "T")
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
        Arg::option("config-file", "CONFIG_FILE")
            .short("d")
            .help(SHORT)
            .long_help(LONG)
            .env("TODORS_CFG_FILE"),
    );
}

fn command_list(cmds: &mut Vec<App>) {
    const ABOUT: &str =
        "Displays all tasks that contain TERM(s) sorted by priority with line numbers.";
    let cmd = SubCommand::with_name("list")
        .alias("ls")
        .about(ABOUT)
        .arg(arg_terms());
    cmds.push(cmd);

    // TODO: make sure list filter actually works according to help
    // local args
    fn arg_terms() -> Arg {
        const SHORT: &str = "Term to filter task list by.";
        const LONG: &str = long!("\
Term to filter task list by.

Each task must match all TERM(s) (logical AND); to display tasks that contain any TERM (logical OR), use
\"TERM1\\|TERM2\\|...\" (with quotes), or TERM1|TERM2 (unquoted).

Hides all tasks that contain TERM(s) preceded by a minus sign (i.e. -TERM).");
        Arg::positional("terms", "TERM")
            .help(SHORT)
            .long_help(LONG)
            .multiple(true)
    }
}
fn command_add(cmds: &mut Vec<App>) {
    const ABOUT: &str = "Add a line to your todo.txt file.";
    cmds.push(
        SubCommand::with_name("add")
            .alias("a")
            .about(ABOUT)
            .arg(arg_task()),
    );

    // local args
    fn arg_task() -> Arg {
        const SHORT: &str = "Todo item";
        const LONG: &str = long!(
            "\
THING I NEED TO DO +project @context

Adds THING I NEED TO DO to your todo.txt file on its own line.
Project and context notation optional.
Quotes optional."
        );
        Arg::positional("task", "TASK")
            .help(SHORT)
            .long_help(LONG)
            .required(true)
    }
}

fn command_addm(cmds: &mut Vec<App>) {
    const ABOUT: &str = "Add multiple lines to todo.txt file";
    cmds.push(SubCommand::with_name("addm").about(ABOUT).arg(arg_tasks()));

    fn arg_tasks() -> Arg {
        const SHORT: &str = "Todo items (line separated)";
        const LONG: &str = long!(
            "
\"FIRST THING I NEED TO DO +project1 @context
SECOND THING I NEED TO DO +project2 @context\"

Adds FIRST THING I NEED TO DO on its own line and SECOND THING I NEED TO DO on its own line.
Project and context notation optional.
Quotes required."
        );
        Arg::positional("tasks", "TASKS")
            .help(SHORT)
            .long_help(LONG)
            .value_delimiter("\n")
            .required(true)
    }
}

fn command_del(cmds: &mut Vec<App>) {
    const SHORT: &str = "Deletes the task on line of todo.txt";
    const LONG: &str = long!(
        "\
Deletes the task on line of todo.txt.
If TERM specified, deletes only TERM from the task"
    );
    cmds.push(
        SubCommand::with_name("del")
            .alias("rm")
            .about(SHORT)
            .long_about(LONG)
            .args(&[arg_item(), arg_term()]),
    );

    fn arg_item() -> Arg {
        const SHORT: &str = "Line number of task to delete";
        Arg::positional("item", "ITEM").help(SHORT).required(true)
    }

    fn arg_term() -> Arg {
        const SHORT: &str = "Optional term to remove from item";
        const LONG: &str = long!(
            "\
Optional term to remove from item.

If TERM is specified, only the TERM is removed from ITEM.

If no TERM is specified, the entire ITEM is deleted."
        );
        Arg::positional("term", "TERM").help(SHORT).long_help(LONG)
    }
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
    command_add(&mut cmds);
    command_addm(&mut cmds);
    command_list(&mut cmds);
    command_del(&mut cmds);
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
        .usage(USAGE.as_str())
        .template(TEMPLATE)
        .max_term_width(100);

    for arg in base_args() {
        app = app.arg(arg);
    }
    app = app.subcommands(commands());
    app
}

/// Parse the clap matches into Command.
/// Will return an error if required arguments are missing or invalid.
fn handle_subcommand(cmd: (&str, Option<&clap::ArgMatches<'static>>), opt: &mut Opt) -> Result {
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
        ("", None) => (),
        _ => unreachable!(),
    }
    Ok(())
}

/// Parse clap matches into Opt object.
/// The result will now be decoupled from clap, so it isn't needed elsewhere.
pub fn parse() -> Result<Opt> {
    let cli = build_app().get_matches();
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
