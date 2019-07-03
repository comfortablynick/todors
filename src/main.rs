use exitfailure::ExitFailure;
use log::error;
use termcolor::{BufferWriter, ColorChoice};

fn main() -> Result<(), ExitFailure> {
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

    if let Err(e) = todors::run(&args, &mut buf) {
        error!("{:?}", e); // log all errors here
        Err(e)?
    }
    bufwtr.print(&buf).map_err(failure::Error::from)?;
    Ok(())
}
