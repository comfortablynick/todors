/// Common utilities for todors
pub type AnyError = Box<dyn std::error::Error + 'static>;

// Env logger builder (not used currently) {{{
// use chrono::Local;
// use env_logger::fmt::{Color, Style};
// use log::{debug, error, info, log_enabled, trace, warn, Level, LevelFilter};
// use std::io::Write;
// pub fn init_logger(verbosity: u8) {
//     env_logger::Builder::new()
//         .format(|buf, record| {
//             let mut level_style = buf.style();
//             match record.level() {
//                 Level::Trace => level_style.set_color(Color::Black).set_intense(true),
//                 Level::Debug => level_style.set_color(Color::White),
//                 Level::Info => level_style.set_color(Color::Green),
//                 Level::Warn => level_style.set_color(Color::Yellow),
//                 Level::Error => level_style.set_color(Color::Red).set_bold(true),
//             };
//             let level = level_style.value(format!("{:>5}", record.level()));
//             // let tm_fmt = "%H:%M:%S%.6f";
//             // let tm_fmt = "%S%.6f";
//             // let tm_fmt = "%FT%H:%M:%S%.6f";
//             let tm_fmt = "%F %H:%M:%S";
//             let time = Local::now().format(tm_fmt);
//
//             let mut dim_white_style = buf.style();
//             dim_white_style.set_color(Color::White);
//
//             let mut subtle_style = buf.style();
//             subtle_style.set_color(Color::Black).set_intense(true);
//
//             writeln!(
//                 buf,
//                 "\
//                  {lbracket}\
//                  {time}\
//                  {rbracket} \
//                  {level} \
//                  {lbracket}\
//                  {file}\
//                  {colon}\
//                  {line_no}\
//                  {rbracket} \
//                  {record_args}\
//                  ",
//                 lbracket = subtle_style.value("["),
//                 rbracket = subtle_style.value("]"),
//                 colon = subtle_style.value(":"),
//                 file = record.file().unwrap_or("<unnamed>"),
//                 time = time,
//                 level = level,
//                 line_no = record.line().unwrap_or(0),
//                 record_args = &record.args(),
//             )
//         })
//         .filter(
//             Some("todors"),
//             match verbosity {
//                 0 => LevelFilter::Warn,
//                 1 => LevelFilter::Info,
//                 2 => LevelFilter::Debug,
//                 _ => LevelFilter::Trace,
//             },
//         )
//         .init();
// }
// }}}
