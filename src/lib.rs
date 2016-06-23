#[macro_use] extern crate nom;

pub use enums::{
    AudioCodec,
    KnownTypes,
    Result,
    VideoCodec,
};
pub use formats::get_format;
pub use types::{
    Metadata,
    Size,
};

pub mod enums;
pub mod formats;
pub mod types;
