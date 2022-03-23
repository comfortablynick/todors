mod logger;
use clap::Parser;
use log::{info, log_enabled};
use logger::init_logger;
use std::env;
use termcolor::{BufferWriter, ColorChoice};
use todors::{
    actions::handle_command,
    config::{AppContext, Config},
    prelude::*,
};

fn main() -> Result {
    let opts = todors::app::Opt::parse();
    if !opts.quiet {
        init_logger(opts.verbosity);
    }
    info!("{:#?}", opts);
    if opts.plain {
        env::set_var("TERM", "dumb");
    }

    let bufwtr = BufferWriter::stdout(ColorChoice::Auto);
    let mut buf = bufwtr.buffer();

    if log_enabled!(log::Level::Debug) {
        let mut args: Vec<String> = env::args().collect();
        args.remove(0);
        debug!("Running with args: {:?}", args);
    }
    let cfg_file = opts
        .config_file
        .clone()
        .ok_or_else(|| format_err!("could not find valid cfg file path"))?;
    let cfg = Config::from_toml_file(cfg_file)?;
    let mut ctx = AppContext {
        opts,
        settings: cfg.general,
        styles: cfg.styles,
        ..Default::default()
    };
    handle_command(&mut ctx, &mut buf)?;
    bufwtr.print(&buf)?;
    Ok(())
}
