use std::io;

#[derive(Debug)]
pub enum Error {
    NotAFile,
    Io(io::Error),
    Json(serde_json::Error),
    LineParse(String, usize),
    InvalidMatchTemplate(String),
    InvalidInputFormat(String),
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::Io(e)
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::Json(e)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
