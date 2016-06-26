use std::time::Duration;
use KnownTypes;

#[derive(Clone, Debug, PartialEq)]
pub struct Size {
    pub width: u16,
    pub height: u16,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Metadata {
    pub format: KnownTypes,
    pub duration: Duration,
    pub size: Size,
    pub video: String,
    pub audio: Option<String>,
}
