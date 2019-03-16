#![allow(dead_code)]
use crate::{args, err, logger, util::AnyError};
use ansi_term::Color::Fixed;
use regex::{self, Captures, Regex};
use std::{
    fs,
    io::{self, Write},
    path::PathBuf,
};
use structopt::StructOpt;
use termcolor::{BufferWriter, Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

/// Colors
const HOTPINK: u8 = 198;
const GREY: u8 = 246;
const SKYBLUE: u8 = 111;
const OLIVE: u8 = 113;
const LIME: u8 = 154;
const LIGHTORANGE: u8 = 215;
const GREEN: u8 = 2;
const BLUE: u8 = 4;
const TURQUOISE: u8 = 37;
const TAN: u8 = 179;

fn format_colors(s: String) -> String {
    lazy_static! {
        static ref RE_PROJECT: Regex = Regex::new(r"(\+\w+)").unwrap();
        static ref RE_CONTEXT: Regex = Regex::new(r"(@\w+)").unwrap();
    }
    let s = RE_PROJECT.replace_all(&s, |c: &Captures| format!("{}", Fixed(LIME).paint(&c[0])));
    let s = RE_CONTEXT.replace_all(&s, |caps: &Captures| {
        format!("{}", Fixed(LIGHTORANGE).paint(&caps[0]),)
    });
    s.to_string()
}

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

fn format_buffer(s: String, bufwtr: BufferWriter) -> Result<(), AnyError> {
    lazy_static! {
        static ref RE_PRIORITY: Regex = Regex::new(r"(?m)\(([A-Z])\).*$").unwrap();
        static ref RE_PROJECT: Regex = Regex::new(r"(\+\w+)").unwrap();
        static ref RE_CONTEXT: Regex = Regex::new(r"(@\w+)").unwrap();
    }
    let mut buf = bufwtr.buffer();
    let mut color = ColorSpec::new();
    let mut ctr = 0;
    for ln in s.lines() {
        let line = ln;
        // TODO: handle blank lines earlier
        if line == "" {
            continue;
        }
        ctr += 1;
        let line = format!("{:02} {}", ctr, line);
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
        write!(&mut buf, "\n")?;
    }
    bufwtr.print(&buf)?;
    Ok(())
}

fn print_todos(s: String) {
    let lines = s.lines();
    let mut ctr = 0;
    for line in lines {
        if line != "" {
            println!("{:02} {}", ctr + 1, line);
            ctr += 1;
        }
    }
    println!("--\nTODO: {} of {} tasks shown", ctr, ctr,);
}

fn test_termcolor(s: &str) -> Result<(), AnyError> {
    let mut buf = StandardStream::stderr(ColorChoice::Always);
    for n in 0..255 {
        buf.set_color(
            ColorSpec::new()
                .set_fg(Some(Color::Ansi256(n + 1)))
                .set_bold(true),
        )?;
        writeln!(&mut buf, "{} {}", s, n + 1).expect("error writing to buffer");
    }
    buf.reset()?;
    Ok(())
}

fn get_todo_file_path() -> Result<PathBuf, AnyError> {
    let home = dirs::home_dir().ok_or("error getting home directory")?;
    let mut path: PathBuf = home;
    path.push("Dropbox");
    path.push("todo");
    path.push("todo.txt");
    Ok(path)
}

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

    let lines = todo_file.lines().collect::<Vec<&str>>();
    // TODO: any processing before format/display
    // can be done on this vector, such as `lines.sort()`
    let todos = lines.join("\n");
    let bufwtr = BufferWriter::stdout(ColorChoice::Auto);
    format_buffer(todos, bufwtr)?;

    // tests
    // test_termcolor("test")?;
    // assert_eq!(198, get_priority_color("A")?);

    Ok(())
}
