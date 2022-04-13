//! Build cli app using #[derive(Clap)]

use crate::task::SortBy;
use clap::{AppSettings, ArgEnum, IntoApp, Parser};
use clap_complete::{generate, shells::*};

const FLAG_HDG: &str = "FLAGS";
const BIN_NAME: &str = "todors";

#[derive(Parser, Debug, PartialEq, Default)]
#[clap(author,
       about,
       version,
       max_term_width = 80,
       dont_collapse_args_in_usage = true,
       propagate_version = true,
       setting = AppSettings::DeriveDisplayOrder,
    )]
pub struct Opt {
    /// Hide task contexts from output.
    ///
    /// Use twice to unhide contexts, which returns to the default
    /// behavior of showing contexts.
    #[clap(name = "@", short, parse(from_occurrences), help_heading = FLAG_HDG)]
    pub hide_context:          u8,
    /// Hide task projects from output.
    ///
    /// Use twice to unhide projects, which returns to the default
    /// behavior of showing projects.
    #[clap(name = "+", short, parse(from_occurrences), help_heading = FLAG_HDG)]
    pub hide_project:          u8,
    /// Color mode
    #[clap(short, help_heading = FLAG_HDG)]
    pub color:                 bool,
    /// Location of toml config file.
    ///
    /// Various options can be set, including colors and styles.
    #[clap(
        name = "CFG_FILE",
        short = 'd',
        parse(from_os_str),
        env = "TODORS_CFG_FILE",
        hide_env_values = true
    )]
    pub config_file:           Option<std::path::PathBuf>,
    /// Force actions without confirmation or input
    #[clap(short, help_heading = FLAG_HDG)]
    pub force:                 bool,
    /// Hide task priorities from output.
    ///
    /// Use twice to show priorities, which returns to the default
    /// behavior of showing priorities.
    #[clap(name = "P", short, parse(from_occurrences), help_heading = FLAG_HDG)]
    pub hide_priority:         u8,
    /// Don't preserve line (task) numbers.
    ///
    /// Opposite of -N. When a task is deleted, the following tasks will
    /// be moved up one line.
    #[clap(name = "n", short, help_heading = FLAG_HDG)]
    pub remove_blank_lines:    bool,
    /// Preserve line (task) numbers.
    ///
    /// This allows consistent access of the tasks by the same id each time.
    /// When a task is deleted, it will remain blank.
    #[clap(name = "N", short, overrides_with("n"), help_heading = FLAG_HDG)]
    pub preserve_line_numbers: bool,
    /// Plain mode turns off colors.
    ///
    /// It overrides environment settings that control terminal colors.
    /// Color settings in config will have no effect.
    #[clap(short, overrides_with("c"), help_heading = FLAG_HDG)]
    pub plain:                 bool,
    ///Increase log verbosity printed to console.
    ///
    /// Log verbosity increases each time the flag is found.
    ///
    /// For example: -v, -vv, -vvv
    ///
    /// The quiet flag -q will override this setting and will silence
    /// log output.
    #[clap(short, parse(from_occurrences), help_heading = FLAG_HDG)]
    pub verbosity:             u8,
    /// Quiet debug messages on console.
    ///
    /// Overrides verbose (-v) setting. The arguments -vvvq will produce no
    /// console debug output.
    #[clap(short, overrides_with("v"), help_heading = FLAG_HDG)]
    pub quiet:                 bool,
    /// Prepend current date to new task.
    #[clap(name = "t", short, help_heading = FLAG_HDG)]
    pub date_on_add:           bool,
    /// Don't prepend current date to new task.
    #[clap(name = "T", short, overrides_with("t"), help_heading = FLAG_HDG)]
    pub no_date_on_add:        bool,
    /// Sort tasks by property
    #[clap(short, arg_enum)]
    pub sort_by:                  Vec<SortBy>,
    #[clap(subcommand)]
    pub cmd:                   Option<Commands>,
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

#[derive(clap::Subcommand, Debug, Clone, Eq, PartialEq)]
pub enum Commands {
    /// Adds a line of text to todo.txt.
    Add {
        #[clap(name = "TASK", long_help = ADD_TASK)]
        task: String,
    },
    /// Adds multiple lines of text to todo.txt.
    Addm {
        #[clap(name = "TASKS", long_help = ADDM_TASKS)]
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
    /// Generates shell completions to stdout.
    Complete {
        /// Generate completions for this shell.
        #[clap(arg_enum, name = "SHELL")]
        shell: Shell,
    },
    /// Removes duplicate lines from todo.txt.
    Deduplicate,
    /// Deletes a task or part of a task from todo.txt.
    #[clap(alias = "rm")]
    Del {
        /// Line number in todo.txt.
        #[clap(name = "ITEM")]
        item: usize,
        #[clap(name = "TERM", long_help = DEL_TERM)]
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
        #[clap(name = "TERM", long_help = LS_TERM)]
        terms: Vec<String>,
    },
    /// Displays all lines in todo.txt AND done.txt with optional filtering.
    ///
    /// Sorted by priority with line numbers.
    #[clap(alias = "lsa")]
    Listall {
        #[clap(name = "TERM", long_help = LS_TERM)]
        terms: Vec<String>,
    },
    #[clap(alias = "lsp")]
    Listpri { priorities: Vec<String> },
}

#[derive(ArgEnum, Copy, Clone, Debug, Eq, PartialEq)]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
    Powershell,
    Elvish,
}

impl Shell {
    pub(crate) fn generate(&self) {
        let mut app = Opt::command();
        let mut fd = std::io::stdout();
        match self {
            Self::Bash => generate(Bash, &mut app, BIN_NAME, &mut fd),
            Self::Zsh => generate(Zsh, &mut app, BIN_NAME, &mut fd),
            Self::Fish => generate(Fish, &mut app, BIN_NAME, &mut fd),
            Self::Powershell => generate(PowerShell, &mut app, BIN_NAME, &mut fd),
            Self::Elvish => generate(Elvish, &mut app, BIN_NAME, &mut fd),
        }
    }
}
