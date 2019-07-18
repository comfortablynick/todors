use chrono::Local;
use env_logger::{fmt::Color, Env};
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
        let mut level_style = buf.style();
        match record.level() {
            Level::Trace => level_style.set_color(Color::Ansi256(142)), // dim yellow
            Level::Debug => level_style.set_color(Color::Ansi256(37)),  // dim cyan
            Level::Info => level_style.set_color(Color::Ansi256(34)),   // dim green
            Level::Warn => level_style.set_color(Color::Ansi256(130)),  // dim orange
            Level::Error => level_style.set_color(Color::Red).set_bold(true),
        };

        let level = level_style.value(format!("{:5}", record.level()));
        let tm_fmt = "%F %H:%M:%S%.3f";
        let time = Local::now().format(tm_fmt);

        let mut subtle_style = buf.style();
        subtle_style.set_color(Color::Black).set_intense(true);

        let mut gray_style = buf.style();
        gray_style.set_color(Color::Ansi256(250));

        writeln!(
            buf,
            "\
             {lbracket}\
             {time}\
             {rbracket}\
             {level}\
             {lbracket}\
             {file}\
             {colon}\
             {line_no}\
             {rbracket} \
             {record_args}\
             ",
            lbracket = subtle_style.value("["),
            rbracket = subtle_style.value("]"),
            colon = subtle_style.value(":"),
            file = gray_style.value(record.file().unwrap_or("<unnamed>")),
            time = gray_style.value(time),
            level = level,
            line_no = gray_style.value(record.line().unwrap_or(0)),
            record_args = &record.args(),
        )
    })
    .init();
}
