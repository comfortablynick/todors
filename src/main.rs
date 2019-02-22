use env_logger;
use log::info;
use regex::Regex;
use std::io::prelude::{BufRead, Read};
use std::{env, fs, io, path::PathBuf};

struct Lines<R> {
    reader: io::BufReader<R>,
    buf: String,
}

impl<R: Read> Lines<R> {
    fn new(r: R) -> Lines<R> {
        Lines {
            reader: io::BufReader::new(r),
            buf: String::new(),
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

    let _re = Regex::new(r"\d{4}-\d{2}-\d{2}").unwrap();
    let home = dirs::home_dir().expect("error getting home dir!");
    let mut path = PathBuf::from(home);
    path.push("Dropbox");
    path.push("todo");
    path.push("todo.txt");
    info!("path to read: {:?}", &path);

    let file = fs::File::open(&path)?;

    let mut lines = Lines::new(file);
    while let Some(line) = lines.next() {
        let line = line?;

        // Example regex
        // let caps = re.captures(&line);
        // match caps {
        //     Some(c) => println!("Date found! {:?}", c),
        //     None => (),
        // }

        println!("{}", line);
    }
    Ok(())
}
