//! Build cli app using #[derive(Clap)]

use clap::Clap;
use std::path::PathBuf;

#[derive(Clap, Debug, PartialEq, Default)]
#[clap(author, about, version, max_term_width = 80)]
pub struct Opt {
    /// Hide task contexts from output.
    ///
    /// Use twice to unhide contexts, which returns to the default
    /// behavior of showing contexts.
    #[clap(short = "@", long, parse(from_occurrences))]
    pub hide_context:          u8,
    /// Hide task projects from output.
    ///
    /// Use twice to unhide projects, which returns to the default
    /// behavior of showing projects.
    #[clap(short = "+", long, parse(from_occurrences))]
    pub hide_project:          u8,
    /// Hide task priorities from output.
    ///
    /// Use twice to show priorities, which returns to the default
    /// behavior of showing priorities.
    #[clap(short = "P", long, parse(from_occurrences))]
    pub hide_priority:         u8,
    /// Don't preserve line (task) numbers.
    ///
    /// Opposite of -N. When a task is deleted, the following tasks will
    /// be moved up one line.
    #[clap(short = "n", long)]
    pub remove_blank_lines:    bool,
    /// Preserve line (task) numbers.
    ///
    /// This allows consistent access of the tasks by the same id each time.
    /// When a task is deleted, it will remain blank.
    #[clap(short = "N", long, overrides_with("remove-blank-lines"))]
    pub preserve_line_numbers: bool,
    /// Plain mode turns off colors.
    ///
    /// It overrides environment settings that control terminal colors.
    /// Color settings in config will have no effect.
    #[clap(short, long)]
    pub plain:                 bool,
    ///Increase log verbosity printed to console.
    ///
    /// Log verbosity increases each time the flag is found.
    ///
    /// For example: -v, -vv, -vvv
    ///
    /// The quiet flag -q will override this setting and will silence
    /// log output.
    #[clap(short, long = "verbose", parse(from_occurrences))]
    pub verbosity:             u8,
    /// Quiet debug messages on console.
    ///
    /// Overrides verbose (-v) setting. The arguments -vvvq will produce no
    /// console debug output.
    #[clap(short, long, overrides_with("verbose"))]
    pub quiet:                 bool,
    /// Prepend current date to new task.
    #[clap(short = "t", long)]
    pub date_on_add:           bool,
    #[clap(short = "T", long, overrides_with("date-on-add"))]
    /// Don't prepend current date to new task.
    pub no_date_on_add:        bool,
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
    #[clap(subcommand)]
    pub cmd:                   Option<SubCommand>,
}

#[derive(Clap, Debug, Clone, Eq, PartialEq)]
pub enum SubCommand {
    /// Add a line to your todo.txt file.
    #[clap(long_about = r"THING I NEED TO DO +project @context

Adds THING I NEED TO DO to your todo.txt file on its own line.
Project and context notation optional.
Quotes optional.")]
    Add {
        task: String,
    },
    /// Add multiple lines to your todo.txt file.
    #[clap(long_about = r"FIRST THING I NEED TO DO +project1 @context
SECOND THING I NEED TO DO +project2 @context

Adds FIRST THING I NEED TO DO on its own line and SECOND THING I NEED TO DO on its own line.
Project and context notation optional.
Quotes required.")]
    Addm {
        tasks: Vec<String>,
    },
    Addto,
    Append {
        item: usize,
        text: String,
    },
    Delete {
        item: usize,
        term: Option<String>,
    },
    List {
        terms: Vec<String>,
    },
    Listall {
        terms: Vec<String>,
    },
    Listpri {
        priorities: Vec<String>,
    },
}
