// #[macro_use]
// extern crate lazy_static;
// mod args;
// mod cli;
// mod util;
use termcolor::{BufferWriter, ColorChoice};

fn main() -> Result<(), failure::Error> {
    // TODO: remove this after testing and simply pass cli args
    let args: Vec<String> = if std::env::args().len() > 1 {
        std::env::args().collect()
    } else {
        ["todors", "-v"].iter().map(|s| s.to_string()).collect()
    };
    // turn on ANSI escape support on Windows to use color
    #[cfg(windows)]
    ansi_term::enable_ansi_support().expect("Enable ANSI support on Windows");

    let bufwtr = BufferWriter::stdout(ColorChoice::Auto);
    let mut buf = bufwtr.buffer();

    todors::run(&args, &mut buf)?;
    bufwtr.print(&buf).map_err(failure::Error::from)
}
