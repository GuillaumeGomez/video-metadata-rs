extern crate libc;

pub use enums::{
    KnownTypes,
    Result,
};
pub use video_metadata::get_format;
pub use types::{
    Metadata,
    Size,
};

pub mod enums;
pub mod video_metadata;
pub mod types;

#[link(name = "avformat")] extern {}
#[link(name = "avcodec")] extern {}
#[link(name = "avutil")] extern {}
