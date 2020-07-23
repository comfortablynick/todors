//! Configure app settings and context object
use crate::{app::Opt, file::read_file_to_string, prelude::*, style::Style, task::Tasks};
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Default)]
/// Wrapper that holds all current settings, args, and data
/// that needs to be passed around to various functions. It takes
/// the place of "global" variables.
pub struct AppContext {
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

impl Config {
    /// Read and process cfg from toml into Config object
    pub fn from_toml_file<P>(file_path: P) -> Result<Self>
    where
        P: AsRef<Path> + std::fmt::Debug,
    {
        let contents = read_file_to_string(&file_path)?;
        info!("Reading config from toml file {:?}", file_path);
        toml::from_str(contents.as_str()).context("converting toml to config object")
    }
}

impl AppContext {
    /// Expand shell variables in paths and write to
    /// top-level variables in Context
    pub fn expand_paths(&mut self) -> Result {
        const ERR: &str = "Error expanding todo file path";
        self.todo_file = self
            .settings
            .todo_file
            .as_ref()
            .and_then(|s| shellexpand::env(s).ok())
            .map(|s| PathBuf::from(s.as_ref()))
            .ok_or_else(|| format_err!(ERR))?;
        self.done_file = self
            .settings
            .done_file
            .as_ref()
            .and_then(|s| shellexpand::env(s).ok())
            .map(|s| PathBuf::from(s.as_ref()))
            .ok_or_else(|| format_err!(ERR))?;
        self.report_file = self
            .settings
            .report_file
            .as_ref()
            .and_then(|s| shellexpand::env(s).ok())
            .map(|s| PathBuf::from(s.as_ref()))
            .ok_or_else(|| format_err!(ERR))?;
        Ok(())
    }
}
