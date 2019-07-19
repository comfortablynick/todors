//! Helper methods for defining clap apps

#![allow(dead_code)]
use clap::{App as ClapApp, Arg as ClapArg};

pub type Arg = ClapArg<'static, 'static>;
pub type App = ClapApp<'static, 'static>;

#[derive(Clone)]
pub struct CliArg {
    pub claparg:   Arg,
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
        short:           &'static str,
        long:            Option<&'static str>,
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

    /// Create a boolean switch. Switches take no values.
    /// A short name or long name is required.
    pub fn switch(name: &'static str, short: &'static str, long: Option<&'static str>) -> CliArg {
        if short == "" && long.is_none() {
            panic!(
                "error on switch '{}': either a short or long name must be supplied",
                name
            );
        }
        let claparg = Arg::with_name(name).short(short);
        CliArg {
            claparg: if let Some(l) = long {
                claparg.long(l)
            } else {
                claparg
            },
            name,
            doc_short: "",
            doc_long: "",
            hidden: false,
            kind: CliArgKind::Switch {
                name,
                long,
                short,
                multiple: false,
            },
        }
    }

    /// Create a flag. A flag always accepts exactly one argument.
    /// A short name must be supplied, but it can be blank if a long name only is desired.
    /// Either a non-blank short name or a long name must be supplied.
    pub fn flag(
        name: &'static str,
        short: &'static str,
        long: Option<&'static str>,
        value_name: &'static str,
    ) -> CliArg {
        if short == "" && long.is_none() {
            panic!(
                "error on flag '{}': either a short or long name must be supplied",
                name
            );
        }
        let claparg = Arg::with_name(name)
            .short(short)
            .value_name(value_name)
            .takes_value(true)
            .number_of_values(1);
        CliArg {
            claparg: if let Some(l) = long {
                claparg.long(l)
            } else {
                claparg
            },
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
#[macro_export]
macro_rules! long {
    ($lit:expr) => {
        concat!($lit, " ")
    };
}

pub trait ArgExt {
    fn switch(name: &'static str, short: &'static str, long: Option<&'static str>) -> Self;
    fn flag(
        name: &'static str,
        short: &'static str,
        long: Option<&'static str>,
        value_name: &'static str,
    ) -> Self;
    fn positional(name: &'static str, value_name: &'static str) -> Self;
    fn number(self) -> Self;
}

/// Helper methods to extend the clap::Arg type.
/// Trait must be imported to use these methods.
impl ArgExt for Arg {
    /// Create a boolean switch. Switches take no values.
    /// A short name or long name is required.
    fn switch(name: &'static str, short: &'static str, long: Option<&'static str>) -> Self {
        if short == "" && long.is_none() {
            panic!(
                "error on switch '{}': either a short or long name must be supplied",
                name
            );
        }
        let arg = Arg::with_name(name).short(short);
        if let Some(l) = long { arg.long(l) } else { arg }
    }

    /// Create a flag. A flag always accepts exactly one argument.
    /// A short name must be supplied, but it can be blank if a long name only is desired.
    /// Either a non-blank short name or a long name must be supplied.
    fn flag(
        name: &'static str,
        short: &'static str,
        long: Option<&'static str>,
        value_name: &'static str,
    ) -> Self {
        if short == "" && long.is_none() {
            panic!(
                "error on flag '{}': either a short or long name must be supplied",
                name
            );
        }
        let claparg = Arg::with_name(name)
            .short(short)
            .value_name(value_name)
            .takes_value(true)
            .number_of_values(1);
        if let Some(l) = long {
            claparg.long(l)
        } else {
            claparg
        }
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
