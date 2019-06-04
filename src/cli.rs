/** Interact with todo.txt file **/
// extern crate todotxt;
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
    }
    let mut buf = bufwtr.buffer();
    let mut color = ColorSpec::new();
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
        logger::init_logger(opts.verbose);
    }

    log::info!("Running with args: {:?}", args);
    log::info!("Parsed options:\n{:#?}", opts);

    let todo_file = fs::read_to_string(get_todo_file_path()?)?;

    for line in todo_file.lines() {
        log::debug!("{:#?}", line.parse::<todo_txt::Task>());
    }

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

#[cfg(test)]
mod test {
    #[test]
    fn str_to_task() {
        use std::str::FromStr;
        let line = "x (C) 2019-12-18 Get new +pricing for +item @work due:2019-12-31";
        let task = todo_txt::Task::from_str(line).expect("error parsing task");
        assert_eq!(task.subject, "Get new +pricing for +item @work");
        assert_eq!(task.priority, 2);
        assert_eq!(task.contexts, vec!("work".to_owned()));
        assert_eq!(task.projects, vec!("item".to_owned(), "pricing".to_owned()));
        assert_eq!(task.finish_date, None);
        assert_eq!(task.due_date, Some(todo_txt::Date::from_ymd(2019, 12, 31)));
        assert_eq!(task.threshold_date, None);
    }
}
