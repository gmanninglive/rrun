use std::{
    fmt::{self, Display},
    io::Write,
};

pub struct Logger {
    level: Option<Level>,
}

impl Logger {
    pub fn new(level: Option<Level>) -> Self {
        Self { level }
    }

    pub fn log(self, level: Level, message: impl Into<String>) {
        println!("{}", Logger::fmt(level, message))
    }

    fn fmt(level: Level, message: impl Into<String>) -> String {
        format!("{}: {}", level, message.into())
    }
}

impl Write for Logger {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        std::io::stdout().write(
            Logger::fmt(
                self.level.clone().unwrap_or(Level::Info),
                String::from_utf8_lossy(buf),
            )
            .as_bytes(),
        )
    }

    fn flush(&mut self) -> std::io::Result<()> {
        std::io::stdout().flush()
    }
}

#[derive(Clone)]
pub enum Level {
    Info,
    Child(String),
    Error,
    Warn,
}

impl Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Level::Child(s) => {
                write!(f, "{}", s)
            }
            _ => {
                write!(f, "{}", self)
            }
        }
    }
}
