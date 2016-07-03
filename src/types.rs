use std::time::Duration;
use KnownTypes;
use video_metadata::{av_strerror_safe, vmrs_result};

#[derive(Clone, Debug, PartialEq)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Error(pub i32);

impl Error {
    // We can't use std::error::Error, because it'd require a borrowed string we
    // can't provide.
    pub fn error_description(&self) -> String {
        if self.0 < 0 {
            return av_strerror_safe(self.0)
                   .unwrap_or_else(|| "Unknown libav error".to_owned())
        }

        if self.0 == 0 {
            return "Invalid media format".to_owned();
        }

        if self.0 == vmrs_result::VMRS_ERROR_INPUT_FAILURE as i32 {
            "Somehow bad data was provided"
        } else if self.0 == vmrs_result::VMRS_ERROR_ALLOC as i32 {
            "Alloc failure"
        } else if self.0 == vmrs_result::VMRS_FORMAT_NOT_AVAILABLE as i32 {
            "Format wasn't available"
        } else {
            "vmrs error not handled, this is a bug"
        }.to_owned()
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
