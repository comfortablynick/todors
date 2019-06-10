#[macro_use]
extern crate lazy_static;
mod args;
mod cli;
mod logger;

pub type AnyError = Box<dyn std::error::Error + 'static>;

#[macro_export]
macro_rules! err(
    ($($arg:tt)*) => (return Err(std::io::Error::new(std::io::ErrorKind::Other, format!($($arg)*))))
);

fn main() -> Result<(), AnyError> {
    // TODO: remove this after testing and simply pass cli args
    let args: Vec<String> = if std::env::args().len() > 1 {
        std::env::args().collect()
    } else {
        ["todors", "-vvvv"].iter().map(|s| s.to_string()).collect()
    };
    // turn on ANSI escape support on Windows to use color
    #[cfg(windows)]
    ansi_term::enable_ansi_support().expect("Enable ANSI support on Windows");
    cli::run(&args)
}
