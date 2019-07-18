use structopt::{clap::AppSettings, StructOpt};

/// Command line options
#[derive(Debug, StructOpt)]
#[structopt(
        name = env!("CARGO_PKG_NAME"),
        about = env!("CARGO_PKG_DESCRIPTION"),
        setting = AppSettings::DontCollapseArgsInUsage,
        setting = AppSettings::VersionlessSubcommands,
        setting = AppSettings::DeriveDisplayOrder,  // Help shows in order of this struct
        setting = AppSettings::UnifiedHelpMessage,  // Options and flags are combined
        setting = AppSettings::AllArgsOverrideSelf, // Last value of arg is kept
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
    #[structopt(short = "H", hidden = true)]
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

    // TODO: figure out how to handle negating flags
    /// Prepend current date to new task
    #[structopt(short = "t", overrides_with = "no_date_on_add")]
    pub date_on_add: bool,

    /// Don't prepend current date to new task
    #[structopt(short = "T", overrides_with = "date_on_add")]
    pub no_date_on_add: bool,

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
