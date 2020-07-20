use clap::Clap;
use log::info;
use rs_console_logger::Logger;
use std::env;
use termcolor::{BufferWriter, ColorChoice};
use todors::{
    actions::handle_command,
    config::{AppContext, Config},
    prelude::*,
};

fn main() -> Result {
    let args: Vec<String> = env::args().collect();
    let opts = todors::app::Opt::parse();
    if !opts.quiet {
        Logger::init(opts.verbosity).unwrap();
    }
    info!("{:#?}", opts);
    if opts.plain {
        env::set_var("TERM", "dumb");
    }

    let bufwtr = BufferWriter::stdout(ColorChoice::Auto);
    let mut buf = bufwtr.buffer();

    let mut args = args;
    args.remove(0);
    info!("Running with args: {:?}", args);
    let cfg_file = opts
        .config_file
        .clone()
        .expect("could not find valid cfg file path");
    let cfg = Config::from_toml_file(cfg_file)?;
    let mut ctx = AppContext {
        opts,
        settings: cfg.general,
        styles: cfg.styles,
        ..Default::default()
    };
    if let Err(e) = handle_command(&mut ctx, &mut buf) {
        return Err(e.into());
    }
    bufwtr.print(&buf)?;
    Ok(())
}
