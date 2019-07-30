use crate::{cli::*, style::Style, task::Tasks};
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Default)]
/// Wrapper that holds all current settings, args, and data
/// that needs to be passed around to various functions. It takes
/// the place of "global" variables.
pub struct Context {
    pub opts:        Opt,
    pub settings:    Settings,
    pub styles:      Vec<Style>,
    pub tasks:       Tasks,
    pub done:        Tasks,
    pub task_ct:     usize,
    pub done_ct:     usize,
    pub todo_file:   PathBuf,
    pub done_file:   PathBuf,
    pub report_file: PathBuf,
}

/// General app settings
#[derive(Debug, Deserialize, Default)]
pub struct Settings {
    pub todo_file:      Option<String>,
    pub done_file:      Option<String>,
    pub report_file:    Option<String>,
    pub date_on_add:    Option<bool>,
    pub default_action: Option<String>,
}

/// All configuration settings from toml
#[derive(Debug, Deserialize)]
pub struct Config {
    pub general: Settings,
    pub styles:  Vec<Style>,
}

/// Read and process cfg from toml into Config object
pub fn read_config<P>(file_path: P) -> Result<Config>
where
    P: AsRef<Path>,
    P: std::fmt::Debug,
{
    use std::io::prelude::*;
    let mut config_toml = String::new();
    let mut file = std::fs::File::open(&file_path)
        .context(format!("could not open file {:?}", file_path))
        .map_err(Error::from)?;
    info!("Found config file at {:?}", file_path);
    file.read_to_string(&mut config_toml)?;
    toml::from_str(&config_toml)
        .context("could not convert toml config data")
        .map_err(Error::from)
}

/// Expand shell variables in paths and write to
/// top-level variables in Context
pub fn expand_paths(ctx: &mut Context) -> Result {
    ctx.todo_file = ctx
        .settings
        .todo_file
        .as_ref()
        .and_then(|s| shellexpand::env(s).ok())
        .map(|s| PathBuf::from(s.into_owned()))
        .ok_or_else(|| err_msg("could not get todo file"))?;
    ctx.done_file = ctx
        .settings
        .done_file
        .as_ref()
        .and_then(|s| shellexpand::env(s).ok())
        .map(|s| PathBuf::from(s.into_owned()))
        .ok_or_else(|| err_msg("could not get todo file"))?;
    ctx.report_file = ctx
        .settings
        .report_file
        .as_ref()
        .and_then(|s| shellexpand::env(s).ok())
        .map(|s| PathBuf::from(s.into_owned()))
        .ok_or_else(|| err_msg("could not get todo file"))?;
    Ok(())
}
