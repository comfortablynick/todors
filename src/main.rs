use ansi_term::Color::Fixed;
use env_logger;
use log::info;
use regex::{Captures, Regex};
use std::io::prelude::{BufRead, Read};
use std::{env, fs, io, path::PathBuf};

const LIME: u8 = 154;
const LIGHTORANGE: u8 = 215;

struct Lines<R> {
    reader: io::BufReader<R>,
    buf: String,
    length: u32,
}

impl<R: Read> Lines<R> {
    fn new(r: R) -> Lines<R> {
        Lines {
            reader: io::BufReader::new(r),
            buf: String::new(),
            length: 0,
        }
    }
    fn next<'a>(&'a mut self) -> Option<io::Result<&'a str>> {
        self.buf.clear();
        match self.reader.read_line(&mut self.buf) {
            Ok(nbytes) => {
                if nbytes == 0 {
                    None // no more lines!
                } else {
                    let line = self.buf.trim_right();
                    self.length += 1;
                    Some(Ok(line))
                }
            }
            Err(e) => Some(Err(e)),
        }
    }
}

fn main() -> io::Result<()> {
    env::set_var("RUST_LOG", "warning");
    env_logger::init();

    // let re_date = Regex::new(r"(?P<y>\d{4})-(?P<m>\d{2})-(?P<d>\d{2})").unwrap();
    let re_project = Regex::new(r"(\+\w+)").unwrap();
    let re_context = Regex::new(r"(.*)(@\S+)(.*)").unwrap();
    let home = dirs::home_dir().expect("error getting home dir!");
    let mut path = PathBuf::from(home);
    path.push("Dropbox");
    path.push("todo");
    path.push("todo.txt");
    info!("path to read: {:?}", &path);

    let file = fs::File::open(&path)?;

    let mut lines = Lines::new(file);
    let mut ctr = 1;

    while let Some(line) = lines.next() {
        let line = line?;
        let line = re_project.replace_all(&line, |c: &Captures| {
            format!("{}", Fixed(LIME).paint(&c[0]))
        });
        let line = re_context.replace_all(&line, |caps: &Captures| {
            format!(
                "{}{}{}",
                &caps[1],
                Fixed(LIGHTORANGE).paint(&caps[2]),
                &caps[3]
            )
        });
        println!("{:02} {}", ctr, line);
        ctr += 1;
    }
    println!("--\nTODO: {} of {} tasks shown", ctr - 1, lines.length);
    Ok(())
}
