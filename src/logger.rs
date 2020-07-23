//! Configure console logging with env_logger
use env_logger::{
    fmt::{Color, Style, StyledValue},
    Env,
};
use log::{self, Level};
use std::io::Write;

/// Initialize customized instance of env_logger
pub fn init_logger(verbose: u8) {
    env_logger::Builder::from_env(Env::new().default_filter_or(match verbose {
        0 => "warn",
        1 => "info",
        2 => "debug",
        _ => "trace",
    }))
    .format(|buf, record| {
        let mut style = buf.style();
        style.set_bold(true);
        let level = colored_level(&mut style, record.level());

        let mut style = buf.style();
        style.set_color(Color::Black).set_intense(true);
        let lbracket = style.value("[");
        let rbracket = style.value("]");

        let mut style = buf.style();
        let target = style.set_bold(true).value(record.target());

        writeln!(
            buf,
            "\
             {lbracket}\
             {level} \
             {target}\
             {rbracket} \
             {record_args}\
             ",
            lbracket = lbracket,
            rbracket = rbracket,
            target = target,
            level = level,
            record_args = &record.args(),
        )
    })
    .init();
}

/// Style log level with color
fn colored_level(style: &mut Style, level: Level) -> StyledValue<&'static str> {
    match level {
        Level::Trace => style.set_color(Color::Black).value("TRACE"),
        Level::Debug => style.set_color(Color::Cyan).value("DEBUG"),
        Level::Info => style.set_color(Color::Green).value("INFO "),
        Level::Warn => style.set_color(Color::Magenta).value("WARN "),
        Level::Error => style.set_color(Color::Red).value("ERROR"),
    }
}
