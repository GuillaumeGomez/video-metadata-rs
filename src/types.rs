use std::time::Duration;
use KnownTypes;

#[derive(Clone, Debug, PartialEq)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    FileError,
    UnknownFormat,
    CostumError(String),
}

impl Error {
    // We can't use std::error::Error, because it'd require a borrowed string we
    // can't provide.
    pub fn error_description(&self) -> String {
        match self {
            &Error::FileError => "FileError".to_owned(),
            &Error::UnknownFormat => "UnknownFormat".to_owned(),
            &Error::CostumError(ref e) => e.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Metadata {
    pub format: KnownTypes,
    pub duration: Option<Duration>,
    pub size: Size,
    pub video: String,
    pub audio: Option<String>,
}
