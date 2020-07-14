#[macro_use]
extern crate lazy_static;

pub mod actions;
pub mod app;
pub mod cli;
pub mod color;
pub mod config;
pub mod errors;
pub mod file;
pub mod style;
pub mod task;
pub mod util;

use exitfailure::ExitFailure;
use log::error;
use termcolor::{BufferWriter, ColorChoice};

fn main() -> Result<(), ExitFailure> {
    let args: Vec<String> = std::env::args().collect();

    // turn on ANSI escape support on Windows to use color
    #[cfg(windows)]
    ansi_term::enable_ansi_support().expect("Enable ANSI support on Windows");

    let opts = cli::parse(&args).map_err(|e| failure::err_msg(e))?;
    if !opts.quiet {
        util::init_env_logger(opts.verbosity);
    }
    if opts.plain {
        std::env::set_var("TERM", "dumb");
    }
    let bufwtr = BufferWriter::stdout(ColorChoice::Auto);
    let mut buf = bufwtr.buffer();

    let mut args = args;
    args.remove(0);
    log::info!("Running with args: {:?}", args);
    let cfg_file = opts
        .config_file
        .clone()
        .expect("could not find valid cfg file path");
    let cfg = config::read_config(cfg_file)?;
    let mut ctx = config::Context {
        opts,
        settings: cfg.general,
        styles: cfg.styles,
        ..Default::default()
    };
    if let Err(e) = actions::handle_command(&mut ctx, &mut buf) {
        error!("{:?}", e); // log all errors here
        return Err(e.into());
    }
    bufwtr.print(&buf).map_err(failure::Error::from)?;
    Ok(())
}
