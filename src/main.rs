use exitfailure::ExitFailure;
use log::error;
use termcolor::{BufferWriter, ColorChoice};

fn main() -> Result<(), ExitFailure> {
    let args: Vec<String> = std::env::args().collect();

    // turn on ANSI escape support on Windows to use color
    #[cfg(windows)]
    ansi_term::enable_ansi_support().expect("Enable ANSI support on Windows");

    let bufwtr = BufferWriter::stdout(ColorChoice::Auto);
    let mut buf = bufwtr.buffer();

    if let Err(e) = todors::run(&args, &mut buf) {
        error!("{:?}", e); // log all errors here
        return Err(e.into());
    }
    bufwtr.print(&buf).map_err(failure::Error::from)?;
    Ok(())
}
