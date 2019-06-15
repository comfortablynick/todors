/// Custom errors and traits

pub mod error {
    use std::io;

    #[derive(Debug)]
    pub enum CliError {
        Io(io::Error),
        Unicode(std::str::Utf8Error),
    }

    impl From<io::Error> for CliError {
        fn from(error: io::Error) -> Self {
            CliError::Io(error)
        }
    }

    impl From<std::str::Utf8Error> for CliError {
        fn from(error: std::str::Utf8Error) -> Self {
            CliError::Unicode(error)
        }
    }

    #[derive(Debug)]
    pub struct AppError {
        pub kind: String,
        pub message: String,
    }

    impl From<io::Error> for AppError {
        fn from(error: io::Error) -> Self {
            AppError {
                kind: String::from("io"),
                message: error.to_string(),
            }
        }
    }

    impl From<std::str::Utf8Error> for AppError {
        fn from(error: std::str::Utf8Error) -> Self {
            AppError {
                kind: String::from("UTF-8"),
                message: error.to_string(),
            }
        }
    }

    impl From<std::string::FromUtf8Error> for AppError {
        fn from(error: std::string::FromUtf8Error) -> Self {
            AppError {
                kind: String::from("UTF-8"),
                message: error.to_string(),
            }
        }
    }

    impl From<&str> for AppError {
        fn from(msg: &str) -> Self {
            AppError {
                kind: String::from("other"),
                message: String::from(msg),
            }
        }
    }

    impl From<toml::de::Error> for AppError {
        fn from(error: toml::de::Error) -> Self {
            AppError {
                kind: String::from("toml"),
                message: error.to_string(),
            }
        }
    }
}

pub mod logger {

    /** Format output of env_logger buffer */
    use chrono::Local;
    use env_logger::{fmt::Color, Env};
    use log::{self, Level};
    use std::io::Write;

    // Colors
    const DIM_CYAN: u8 = 37;
    const DIM_GREEN: u8 = 34;
    const DIM_YELLOW: u8 = 142;
    const DIM_ORANGE: u8 = 130;
    const DIM_MAGENTA: u8 = 127;

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
                Level::Trace => level_style.set_color(Color::Ansi256(DIM_YELLOW)),
                Level::Debug => level_style.set_color(Color::Ansi256(DIM_CYAN)),
                Level::Info => level_style.set_color(Color::Ansi256(DIM_GREEN)),
                Level::Warn => level_style.set_color(Color::Ansi256(DIM_ORANGE)),
                Level::Error => level_style.set_color(Color::Ansi256(DIM_MAGENTA)),
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
}
