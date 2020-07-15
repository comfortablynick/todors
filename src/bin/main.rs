use clap::Clap;
use exitfailure::ExitFailure;
use log::{error, info};
use std::env;
use termcolor::{BufferWriter, ColorChoice};
use todors::{actions::handle_command, config, util::init_env_logger};

fn main() -> Result<(), ExitFailure> {
    let args: Vec<String> = env::args().collect();

    // let opts = cli::parse(&args).map_err(|e| failure::err_msg(e))?;
    let opts = todors::app::Opt::parse();
    if !opts.quiet {
        init_env_logger(opts.verbosity);
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
    let cfg = config::read_config(cfg_file)?;
    let mut ctx = config::Context {
        opts,
        settings: cfg.general,
        styles: cfg.styles,
        ..Default::default()
    };
    if let Err(e) = handle_command(&mut ctx, &mut buf) {
        error!("{:?}", e); // log all errors here
        return Err(e.into());
    }
    bufwtr.print(&buf).map_err(failure::Error::from)?;
    Ok(())
}