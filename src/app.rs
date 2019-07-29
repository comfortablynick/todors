//! Helper methods for defining clap apps

use clap::{App, Arg};

/// Add an extra space to long descriptions so that a blank line is inserted
/// between flag descriptions in --help output.
#[macro_export]
macro_rules! long {
    ($lit:expr) => {
        concat!($lit, " ")
    };
}

/// Additional methods extending the clap App struct
pub trait AppExt {
    /// Subcommand with sensible defaults
    fn command(name: &'static str) -> Self;
}

impl AppExt for App<'static> {
    fn command(name: &'static str) -> Self {
        App::new(name)
            .arg(
                Arg::flag("quiet", 'q')
                    .help("Silence debug messages")
                    .overrides_with("quiet"),
            )
            .arg(
                Arg::flag("verbose", 'v')
                    .help("Output debug messages to console")
                    .overrides_with("verbose"),
            )
    }
}

/// Additional methods extending the clap Arg struct
pub trait ArgExt {
    /// Create a boolean flag. Flags take no values.
    /// Flag name is assigned as long name.
    /// Short name can be a blank string to use long name only.
    fn flag(name: &'static str, short: char) -> Self;
    /// Create an option. A option always accepts exactly one argument.
    /// A short name may be supplied. The `name` will be used as long name.
    /// If no long name is desired, create a clap::Arg from scratch.
    fn option(name: &'static str, value_name: &'static str) -> Self;
    /// Create a positional argument
    fn positional(name: &'static str, value_name: &'static str) -> Self;
    /// Indicate that any value given to this argument should be a number. If
    /// it's not a number, then clap will report an error to the end user.
    fn number(self) -> Self;
    /// Remove long option name that was set with helper functions
    /// May fail if used on positional arg
    fn no_long(self, remove: bool) -> Self;
}

impl ArgExt for Arg<'static> {
    fn flag(name: &'static str, short: char) -> Self {
        Arg::with_name(name).long(name).short(short)
    }

    fn option(name: &'static str, value_name: &'static str) -> Self {
        Arg::with_name(name)
            .long(name)
            .value_name(value_name)
            .takes_value(true)
            .number_of_values(1)
    }

    fn positional(name: &'static str, value_name: &'static str) -> Self {
        Arg::with_name(name).value_name(value_name)
    }

    fn number(self) -> Self {
        self.validator(|val| {
            val.parse::<usize>()
                .map(|_| ())
                .map_err(|err| err.to_string())
        })
    }

    fn no_long(mut self, remove: bool) -> Self {
        if remove {
            self.long = None;
        }
        self
    }
}
