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

#[derive(Clap, Debug, Clone, Eq, PartialEq)]
pub enum SubCommand {
    /// Add a line to your todo.txt file.
    Add {
        /// THING I NEED TO DO +project @context
        ///
        /// Adds THING I NEED TO DO to your todo.txt file on its own line.
        /// Project and context notation optional.
        /// Quotes optional.
        #[clap(name = "TASK")]
        task: String,
    },
    /// Add multiple lines to your todo.txt file.
    Addm {
        #[clap(
            name = "TASKS",
            long_about = r"FIRST THING I NEED TO DO +project1 @context
SECOND THING I NEED TO DO +project2 @context

Adds FIRST THING I NEED TO DO on its own line and SECOND THING I NEED TO DO on its own line.
Project and context notation optional.
Quotes required."
        )]
        tasks: Vec<String>,
    },
    /// TODO: unimplemented
    Addto,
    /// TODO: unimplemented
    Append {
        /// Line number of todo.txt to append.
        #[clap(name = "ITEM")]
        item: usize,
        /// Text to append to ITEM.
        #[clap(name = "TEXT")]
        text: String,
    },
    /// Delete a line in todo.txt.
    ///
    /// If TERM specified, deletes only the TERM from the ITEM.
    #[clap(alias = "rm")]
    Del {
        /// Line number of task to delete.
        #[clap(name = "ITEM")]
        item: usize,
        /// Optional term to remove from item.
        ///
        /// If TERM is specified, only the TERM is removed from ITEM.
        /// If no TERM is specified, the entire ITEM is deleted.
        #[clap(name = "TERM")]
        term: Option<String>,
    },
    /// Display all the lines in todo.txt with optional filtering.
    ///
    /// Sorted by priority with line numbers.
    #[clap(alias = "ls")]
    List {
        /// Term to filter task list by.
        ///
        /// Each task must match all TERM(s) (logical AND); to display tasks
        /// that contain any TERM (logical OR), use "TERM1\|TERM2\|..." (with quotes),
        /// or TERM1|TERM2 (unquoted).
        ///
        /// Hide all tasks that contain TERM(s) preceded by a minus sign (i.e. -TERM).
        #[clap(name = "TERM")]
        terms: Vec<String>,
    },
    /// Display all lines in todo.txt AND done.txt with optional filtering.
    ///
    /// Sorted by priority with line numbers.
    #[clap(alias = "lsa")]
    Listall {
        /// Term to filter task list by.
        ///
        /// Each task must match all TERM(s) (logical AND); to display tasks
        /// that contain any TERM (logical OR), use "TERM1\|TERM2\|..." (with quotes),
        /// or TERM1|TERM2 (unquoted).
        ///
        /// Hide all tasks that contain TERM(s) preceded by a minus sign (i.e. -TERM).
        #[clap(name = "TERM")]
        terms: Vec<String>,
    },
    #[clap(alias = "lsp")]
    Listpri { priorities: Vec<String> },
}
