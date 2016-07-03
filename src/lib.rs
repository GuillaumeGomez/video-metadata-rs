extern crate libc;
#[macro_use]
extern crate lazy_static;

pub use enums::{
    KnownTypes,
};
pub use video_metadata::{
    get_format_from_file,
    get_format_from_slice,
};
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
