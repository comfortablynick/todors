/** Interact with todo.txt file **/
use crate::{args, err, logger, util::AnyError};
use regex::{self, Regex};
use std::{
    fs,
    io::{self, Write},
    path::PathBuf,
};
use structopt::StructOpt;
use termcolor::{BufferWriter, Color, ColorChoice, ColorSpec, WriteColor};

/// Colors
const HOTPINK: u8 = 198;
const LIME: u8 = 154;
const LIGHTORANGE: u8 = 215;
const GREEN: u8 = 2;
const BLUE: u8 = 4;
const TURQUOISE: u8 = 37;
const TAN: u8 = 179;
// const GREY: u8 = 246;
// const SKYBLUE: u8 = 111;
// const OLIVE: u8 = 113;

/// Get color for a given priority
fn get_priority_color(c: char) -> Result<ColorSpec, io::Error> {
    let mut color = ColorSpec::new();
    match c {
        'A' => color.set_fg(Some(Color::Ansi256(HOTPINK))),
        'B' => color.set_fg(Some(Color::Ansi256(GREEN))),
        'C' => color.set_fg(Some(Color::Ansi256(BLUE))).set_bold(true),
        'D' => color.set_fg(Some(Color::Ansi256(TURQUOISE))).set_bold(true),
        'E'...'Z' => color.set_fg(Some(Color::Ansi256(TAN))),
        _ => err!("color for priority '{}' not found!", &c),
    };
    Ok(color)
}

/// Use regex to add color to priorities, projects and contexts
fn format_buffer(s: Vec<String>, bufwtr: BufferWriter) -> Result<(), AnyError> {
    lazy_static! {
        static ref RE_PRIORITY: Regex = Regex::new(r"(?m)\(([A-Z])\).*$").unwrap();
        static ref RE_PROJECT: Regex = Regex::new(r"(\+\w+)").unwrap();
        static ref RE_CONTEXT: Regex = Regex::new(r"(@\w+)").unwrap();
    }
    let mut buf = bufwtr.buffer();
    let mut color = ColorSpec::new();
    // let mut ctr = 0;
    for ln in s {
        let line = ln;
        if RE_PRIORITY.is_match(&line) {
            // get priority
            let caps = RE_PRIORITY.captures(&line).unwrap();
            let pri = caps
                .get(1)
                .map_or("", |c| c.as_str())
                .chars()
                .next()
                .unwrap();
            color = get_priority_color(pri)?;
            buf.set_color(&color)?;
        } else {
            buf.reset()?;
        }
        for word in line.split_whitespace() {
            let first_char = word.chars().next();
            let prev_color = color.fg().cloned();
            match first_char {
                Some('+') => {
                    color.set_fg(Some(Color::Ansi256(LIME)));
                    buf.set_color(&color)?;
                    write!(&mut buf, "{} ", word)?;
                    color.set_fg(prev_color);
                    buf.set_color(&color)?;
                }
                Some('@') => {
                    color.set_fg(Some(Color::Ansi256(LIGHTORANGE)));
                    buf.set_color(&color)?;
                    write!(&mut buf, "{} ", word)?;
                    color.set_fg(prev_color);
                    buf.set_color(&color)?;
                }
                _ => {
                    write!(&mut buf, "{} ", word)?;
                }
            }
        }
        buf.reset()?;
        writeln!(&mut buf)?;
    }
    bufwtr.print(&buf)?;
    Ok(())
}

/// Gets path based on default location
fn get_todo_file_path() -> Result<PathBuf, AnyError> {
    let home = dirs::home_dir().ok_or("error getting home directory")?;
    let mut path: PathBuf = home;
    path.push("Dropbox");
    path.push("todo");
    path.push("todo.txt");
    Ok(path)
}

/// Entry point for main program logic
pub fn run(args: Vec<String>) -> Result<(), AnyError> {
    let opts = args::Opt::from_iter(&args);

    if !opts.quiet {
        // init logger
        logger::Logger::init().expect("error initializing logger");
        log::set_max_level(match opts.verbose {
            0 => log::LevelFilter::Warn,
            1 => log::LevelFilter::Info,
            2 => log::LevelFilter::Debug,
            _ => log::LevelFilter::Trace,
        });
    }

    log::trace!("Running with args: {:?}", args);
    log::debug!("Parsed options:\n{:#?}", opts);

    let todo_file = fs::read_to_string(get_todo_file_path()?)?;

    let mut ctr = 0;
    let lines = todo_file
        .lines()
        .map(|ln| {
            ctr += 1;
            format!("{:02} {}", ctr, ln)
        })
        .collect::<Vec<String>>();

    let bufwtr = BufferWriter::stdout(ColorChoice::Auto);
    format_buffer(lines, bufwtr)?;

    Ok(())
}
