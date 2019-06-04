use std::path::PathBuf;
use structopt::StructOpt;

/// Command line options
#[derive(Debug, StructOpt)]
#[structopt(
    name = "todors",
    about = "View and edit a file in todo.txt format",
    raw(setting = "structopt::clap::AppSettings::DontCollapseArgsInUsage")
)]
pub struct Opt {
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[structopt(short = "v", long = "verbose", parse(from_occurrences))]
    pub verbose: u8,

    /// Quiet debug messages
    #[structopt(short = "q", long = "quiet")]
    pub quiet: bool,

    /// Usage information
    #[structopt(long = "usage")]
    pub usage: bool,

    /// Use a config file other than the default ~/.todo/config
    #[structopt(short = "d", name = "CONFIG_FILE", parse(from_os_str))]
    pub config_file: Option<PathBuf>,

    /// List contents of todo.txt file
    #[structopt(subcommand)]
    pub cmd: Option<Command>,
}

#[derive(StructOpt, Debug)]
pub enum Command {
    /// Add line to todo.txt file
    #[structopt(name = "add", visible_alias = "a")]
    Add {
        #[structopt(name = "todo")]
        /// Todo item
        ///
        /// "THING I NEED TO DO +project @context"
        todo: String,
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
        #[structopt(name = "todo")]
        todo: String,
    },

    /// Add line of text to any file in the todo.txt directory
    #[structopt(name = "addto")]
    Addto,

    /// Add text to end of the item
    #[structopt(name = "append", visible_alias = "app")]
    Append {
        /// Append text to end of this line number
        #[structopt(name = "item")]
        item: u32,

        /// Text to append (quotes optional)
        #[structopt(name = "text")]
        text: String,
    },

    /// List todos
    #[structopt(name = "list", visible_alias = "ls")]
    List,

    /// List all todos
    #[structopt(name = "listall", visible_alias = "lsa")]
    Listall,
}
