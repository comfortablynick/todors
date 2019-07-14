// methods adapted from ripgrep
use crate::errors::Result;
use clap::{value_t, values_t, AppSettings, SubCommand};
use log::{debug, log_enabled, trace};
use std::convert::TryInto;

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
    pub config_file:           Option<std::path::PathBuf>,
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

type Arg = clap::Arg<'static, 'static>;
type App = clap::App<'static, 'static>;

#[derive(Clone)]
pub struct CliArg {
    claparg:       Arg,
    pub name:      &'static str,
    pub doc_short: &'static str,
    pub doc_long:  &'static str,
    pub hidden:    bool,
    pub kind:      CliArgKind,
}

#[derive(Clone)]
pub enum CliArgKind {
    Positional {
        value_name: &'static str,
        multiple:   bool,
    },
    Switch {
        name:     &'static str,
        short:    &'static str,
        long:     Option<&'static str>,
        multiple: bool,
    },
    Flag {
        name:            &'static str,
        long:            Option<&'static str>,
        short:           &'static str,
        value_name:      &'static str,
        multiple:        bool,
        possible_values: Vec<&'static str>,
    },
}

impl CliArg {
    /// Create a positional argument
    pub fn positional(name: &'static str, value_name: &'static str) -> CliArg {
        CliArg {
            claparg: Arg::with_name(name).value_name(value_name),
            name,
            doc_short: "",
            doc_long: "",
            hidden: false,
            kind: CliArgKind::Positional {
                value_name,
                multiple: false,
            },
        }
    }

    /// Create a boolean switch
    pub fn switch(name: &'static str, short: &'static str) -> CliArg {
        CliArg {
            claparg: Arg::with_name(name).short(short),
            name,
            doc_short: "",
            doc_long: "",
            hidden: false,
            kind: CliArgKind::Switch {
                name,
                long: None,
                short,
                multiple: false,
            },
        }
    }

    /// Create a flag. A flag always accepts exactly one argument.
    pub fn flag(name: &'static str, short: &'static str, value_name: &'static str) -> CliArg {
        CliArg {
            claparg: Arg::with_name(name)
                .value_name(value_name)
                .takes_value(true)
                .number_of_values(1),
            name,
            doc_short: "",
            doc_long: "",
            hidden: false,
            kind: CliArgKind::Flag {
                name,
                long: None,
                short,
                value_name,
                multiple: false,
                possible_values: vec![],
            },
        }
    }

    /// Set the short flag name.
    ///
    /// This panics if this arg isn't a switch or a flag.
    pub fn short(mut self, short_name: &'static str) -> CliArg {
        match self.kind {
            CliArgKind::Positional { .. } => panic!("expected switch or flag"),
            CliArgKind::Switch { ref mut short, .. } => {
                *short = short_name;
            }
            CliArgKind::Flag { ref mut short, .. } => {
                *short = short_name;
            }
        }
        self.claparg = self.claparg.short(short_name);
        self
    }

    /// Set the "short" help text.
    ///
    /// This should be a single line. It is shown in the `-h` output.
    pub fn help(mut self, text: &'static str) -> CliArg {
        self.doc_short = text;
        self.claparg = self.claparg.help(text);
        self
    }

    /// Set the "long" help text.
    ///
    /// This should be at least a single line, usually longer. It is shown in `--help` output.
    pub fn long_help(mut self, text: &'static str) -> CliArg {
        self.doc_long = text;
        self.claparg = self.claparg.long_help(text);
        self
    }

    /// Enable this argument to accept multiple values.
    ///
    /// Note that while switches and flags can always be repeated an arbitrary
    /// number of times, this particular method enables the flag to be
    /// logically repeated where each occurrence of the flag may have
    /// significance. That is, when this is disabled, then a switch is either
    /// present or not and a flag has exactly one value (the last one given).
    /// When this is enabled, then a switch has a count corresponding to the
    /// number of times it is used and a flag's value is a list of all values
    /// given.
    ///
    /// For the most part, this distinction is resolved by consumers of clap's
    /// configuration.
    pub fn multiple(mut self) -> CliArg {
        match self.kind {
            CliArgKind::Positional {
                ref mut multiple, ..
            } => {
                *multiple = true;
            }
            CliArgKind::Switch {
                ref mut multiple, ..
            } => {
                *multiple = true;
            }
            CliArgKind::Flag {
                ref mut multiple, ..
            } => {
                *multiple = true;
            }
        }
        self.claparg = self.claparg.multiple(true);
        self
    }

    /// Hide this flag from all documentation.
    pub fn hidden(mut self) -> CliArg {
        self.hidden = true;
        self.claparg = self.claparg.hidden(true);
        self
    }

    /// Set the possible values for this argument. If this argument is not
    /// a flag, then this panics.
    ///
    /// If the end user provides any value other than what is given here, then
    /// clap will report an error to the user.
    ///
    /// Note that this will suppress clap's automatic output of possible values
    /// when using -h/--help, so users of this method should provide
    /// appropriate documentation for the choices in the "long" help text.
    pub fn possible_values(mut self, values: &[&'static str]) -> CliArg {
        match self.kind {
            CliArgKind::Positional { .. } => panic!("expected flag"),
            CliArgKind::Switch { .. } => panic!("expected flag"),
            CliArgKind::Flag {
                ref mut possible_values,
                ..
            } => {
                *possible_values = values.to_vec();
                self.claparg = self
                    .claparg
                    .possible_values(values)
                    .hide_possible_values(true);
            }
        }
        self
    }

    /// Add an alias to this argument.
    ///
    /// Aliases are not show in the output of -h/--help.
    pub fn alias(mut self, name: &'static str) -> CliArg {
        self.claparg = self.claparg.alias(name);
        self
    }

    /// Permit this flag to have values that begin with a hypen.
    ///
    /// This panics if this arg is not a flag.
    pub fn allow_leading_hyphen(mut self) -> CliArg {
        match self.kind {
            CliArgKind::Positional { .. } => panic!("expected flag"),
            CliArgKind::Switch { .. } => panic!("expected flag"),
            CliArgKind::Flag { .. } => {
                self.claparg = self.claparg.allow_hyphen_values(true);
            }
        }
        self
    }

    /// Sets this argument to a required argument, unless one of the given
    /// arguments is provided.
    pub fn required_unless(mut self, names: &[&'static str]) -> CliArg {
        self.claparg = self.claparg.required_unless_one(names);
        self
    }

    /// Sets conflicting arguments. That is, if this argument is used whenever
    /// any of the other arguments given here are used, then clap will report
    /// an error.
    pub fn conflicts(mut self, names: &[&'static str]) -> CliArg {
        self.claparg = self.claparg.conflicts_with_all(names);
        self
    }

    /// Sets an overriding argument. That is, if this argument and the given
    /// argument are both provided by an end user, then the "last" one will
    /// win. the cli will behave as if any previous instantiations did not
    /// happen.
    pub fn overrides(mut self, name: &'static str) -> CliArg {
        self.claparg = self.claparg.overrides_with(name);
        self
    }

    /// Sets the default value of this argument if and only if the argument
    /// given is present.
    pub fn default_value_if(mut self, value: &'static str, arg_name: &'static str) -> CliArg {
        self.claparg = self.claparg.default_value_if(arg_name, None, value);
        self
    }

    /// Indicate that any value given to this argument should be a number. If
    /// it's not a number, then clap will report an error to the end user.
    pub fn number(mut self) -> CliArg {
        self.claparg = self.claparg.validator(|val| {
            val.parse::<usize>()
                .map(|_| ())
                .map_err(|err| err.to_string())
        });
        self
    }

    /// Sets an environment variable default for the argument.
    pub fn env(mut self, var_name: &'static str) -> CliArg {
        self.claparg = self.claparg.env(var_name);
        self
    }
}

/// Add an extra space to long descriptions so that a blank line is inserted
/// between flag descriptions in --help output.
macro_rules! long {
    ($lit:expr) => {
        concat!($lit, " ")
    };
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

    let arg = CliArg::switch("verbosity", "v")
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

    let arg = CliArg::switch("quiet", "q")
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

    let arg = CliArg::switch("plain", "p").help(SHORT).long_help(LONG);
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

    let arg = CliArg::switch("preserve_line_numbers", "N")
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

    let arg = CliArg::switch("remove_blank_lines", "n")
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

    let arg = CliArg::switch("hide_context", "@")
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

    let arg = CliArg::switch("hide_project", "+")
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

    let arg = CliArg::switch("hide_priority", "P")
        .help(SHORT)
        .long_help(LONG);
    args.push(arg);
}

fn flag_config_file(args: &mut Vec<CliArg>) {
    const SHORT: &str = "Location of config toml file.";
    const LONG: &str = long!(
        "\
Location of toml config file. Various options can be set, including 
colors and styles."
    );

    let arg = CliArg::flag("config", "d", "CONFIG_FILE")
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
        .setting(AppSettings::VersionlessSubcommands);

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
