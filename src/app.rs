//! Build cli app using #[derive(Clap)]

use clap::{AppSettings, Clap};
use std::path::PathBuf;

#[derive(Clap, Debug, PartialEq, Default)]
#[clap(author,
       about,
       version,
       max_term_width = 80,
       setting = AppSettings::ColoredHelp,
       setting = AppSettings::DontCollapseArgsInUsage, // Doesn't seem to work
       setting = AppSettings::UnifiedHelpMessage,
       setting = AppSettings::DeriveDisplayOrder,
       setting = AppSettings::VersionlessSubcommands,
    )]
pub struct Opt {
    /// Hide task contexts from output.
    ///
    /// Use twice to unhide contexts, which returns to the default
    /// behavior of showing contexts.
    #[clap(name = "@", short, parse(from_occurrences))]
    pub hide_context:          u8,
    /// Hide task projects from output.
    ///
    /// Use twice to unhide projects, which returns to the default
    /// behavior of showing projects.
    #[clap(name = "+", short, parse(from_occurrences))]
    pub hide_project:          u8,
    /// Color mode
    #[clap(short)]
    pub color:                 bool,
    /// Location of toml config file.
    ///
    /// Various options can be set, including colors and styles.
    #[clap(
        name = "CONFIG_FILE",
        short = "d",
        parse(from_os_str),
        env = "TODORS_CFG_FILE",
        hide_env_values = true
    )]
    pub config_file:           Option<PathBuf>,
    /// Force actions without confirmation or input
    #[clap(short)]
    pub force:                 bool,
    /// Hide task priorities from output.
    ///
    /// Use twice to show priorities, which returns to the default
    /// behavior of showing priorities.
    #[clap(name = "P", short, parse(from_occurrences))]
    pub hide_priority:         u8,
    /// Don't preserve line (task) numbers.
    ///
    /// Opposite of -N. When a task is deleted, the following tasks will
    /// be moved up one line.
    #[clap(name = "n", short)]
    pub remove_blank_lines:    bool,
    /// Preserve line (task) numbers.
    ///
    /// This allows consistent access of the tasks by the same id each time.
    /// When a task is deleted, it will remain blank.
    #[clap(name = "N", short, overrides_with("n"))]
    pub preserve_line_numbers: bool,
    /// Plain mode turns off colors.
    ///
    /// It overrides environment settings that control terminal colors.
    /// Color settings in config will have no effect.
    #[clap(short, overrides_with("c"))]
    pub plain:                 bool,
    ///Increase log verbosity printed to console.
    ///
    /// Log verbosity increases each time the flag is found.
    ///
    /// For example: -v, -vv, -vvv
    ///
    /// The quiet flag -q will override this setting and will silence
    /// log output.
    #[clap(short, parse(from_occurrences))]
    pub verbosity:             u8,
    /// Quiet debug messages on console.
    ///
    /// Overrides verbose (-v) setting. The arguments -vvvq will produce no
    /// console debug output.
    #[clap(short, overrides_with("v"))]
    pub quiet:                 bool,
    /// Prepend current date to new task.
    #[clap(name = "t", short)]
    pub date_on_add:           bool,
    #[clap(name = "T", short, overrides_with("t"))]
    /// Don't prepend current date to new task.
    pub no_date_on_add:        bool,
    #[clap(subcommand)]
    pub cmd:                   Option<SubCommand>,
}

const ADD_TASK: &str = "\
THING I NEED TO DO +project @context

Adds THING I NEED TO DO to your todo.txt file on its own line.
Project and context notation optional.
Quotes optional.";

const ADDM_TASKS: &str = "\
FIRST THING I NEED TO DO +project1 @context
SECOND THING I NEED TO DO +project2 @context

Adds FIRST THING I NEED TO DO on its own line and
SECOND THING I NEED TO DO on its own line.

Project and context notation optional.
Quotes required.";

const DEL_TERM: &str = "\
Optional term to remove from ITEM.

If TERM is specified, only the TERM is removed from ITEM.
If no TERM is specified, the entire ITEM is deleted.";

const LS_TERM: &str = "\
Term to filter task list by.

Each task must match all TERM(s) (logical AND);
to display tasks that contain any TERM (logical OR),
use \"TERM1\\|TERM2\\|...\" (with quotes), or TERM1|TERM2
(unquoted).

Hide all tasks that contain TERM(s) preceded by a minus
sign (i.e. -TERM).";

#[derive(Clap, Debug, Clone, Eq, PartialEq)]
pub enum SubCommand {
    /// Adds a line of text to todo.txt.
    Add {
        #[clap(name = "TASK", long_about = ADD_TASK)]
        task: String,
    },
    /// Adds multiple lines of text to todo.txt.
    Addm {
        #[clap(name = "TASKS", long_about = ADDM_TASKS)]
        tasks: Vec<String>,
    },
    /// Adds a line of text to any file located in the todo.txt directory.
    Addto,
    /// Adds text to end of task.
    Append {
        /// Line number of todo.txt to append TEXT.
        #[clap(name = "ITEM")]
        item: usize,
        /// Text to append to ITEM.
        #[clap(name = "TEXT")]
        text: String,
    },
    /// Moves all done tasks from todo.txt to done.txt and removes blank lines.
    Archive,
    /// Removes duplicate lines from todo.txt.
    Deduplicate,
    /// Deletes a task or part of a task from todo.txt.
    #[clap(alias = "rm")]
    Del {
        /// Line number in todo.txt.
        #[clap(name = "ITEM")]
        item: usize,
        #[clap(name = "TERM", long_about = DEL_TERM)]
        term: Option<String>,
    },
    /// Deprioritizes (removes the priority) from the task(s) on line ITEM in todo.txt.
    #[clap(alias = "dp")]
    Depri {
        /// Line number in todo.txt to remove priority.
        #[clap(name = "ITEM")]
        items: Vec<usize>,
    },
    /// Displays all the lines in todo.txt with optional filtering.
    ///
    /// Sorted by priority with line numbers.
    #[clap(alias = "ls")]
    List {
        #[clap(name = "TERM", long_about = LS_TERM)]
        terms: Vec<String>,
    },
    /// Displays all lines in todo.txt AND done.txt with optional filtering.
    ///
    /// Sorted by priority with line numbers.
    #[clap(alias = "lsa")]
    Listall {
        #[clap(name = "TERM", long_about = LS_TERM)]
        terms: Vec<String>,
    },
    #[clap(alias = "lsp")]
    Listpri { priorities: Vec<String> },
}
