//! Helper methods for defining clap apps

use clap::Arg;

/// Add an extra space to long descriptions so that a blank line is inserted
/// between flag descriptions in --help output.
#[macro_export]
macro_rules! long {
    ($lit:expr) => {
        concat!($lit, " ")
    };
}

pub trait ArgExt {
    fn switch(name: &'static str, short: &'static str) -> Self;
    fn flag(name: &'static str, value_name: &'static str) -> Self;
    fn positional(name: &'static str, value_name: &'static str) -> Self;
    fn number(self) -> Self;
}

/// Helper methods to extend the clap::Arg type.
/// Trait must be imported to use these methods.
impl ArgExt for Arg<'static, 'static> {
    /// Create a boolean switch. Switches take no values.
    /// Switch name is assigned as long name.
    /// Short name can be a blank string to use long name only.
    fn switch(name: &'static str, short: &'static str) -> Self {
        Arg::with_name(name).long(name).short(short)
    }

    /// Create a flag. A flag always accepts exactly one argument.
    /// A short name may be supplied. The `name` will be used as long name.
    /// If no long name is desired, create a clap::Arg from scratch.
    fn flag(name: &'static str, value_name: &'static str) -> Self {
        Arg::with_name(name)
            .long(name)
            .value_name(value_name)
            .takes_value(true)
            .number_of_values(1)
    }

    /// Create a positional argument
    fn positional(name: &'static str, value_name: &'static str) -> Self {
        Arg::with_name(name).value_name(value_name)
    }

    /// Indicate that any value given to this argument should be a number. If
    /// it's not a number, then clap will report an error to the end user.
    fn number(self) -> Self {
        self.validator(|val| {
            val.parse::<usize>()
                .map(|_| ())
                .map_err(|err| err.to_string())
        })
    }
}
