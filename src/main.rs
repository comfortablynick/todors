#[macro_use]
extern crate lazy_static;
mod args;
mod cli;
mod logger;
mod util;

#[macro_export]
macro_rules! err(
    ($($arg:tt)*) => (return Err(std::io::Error::new(std::io::ErrorKind::Other, format!($($arg)*))))
);

fn main() -> Result<(), util::AnyError> {
    // TODO: remove test vector after testing
    let args: Vec<String> = if std::env::args().len() > 1 {
        std::env::args().collect()
    } else {
        vec!["todors", "-vvvv"]
            .iter()
            .map(|s| s.to_string())
            .collect()
    };
    // turn on ANSI escape support on Windows to use color
    #[cfg(windows)]
    ansi_term::enable_ansi_support().expect("Enable ANSI support on Windows");
    cli::run(args)
}
