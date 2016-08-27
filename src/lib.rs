extern crate mp4parse as mp4;
extern crate ogg_metadata as ogg;

pub use enums::{
    KnownTypes,
};
pub use video_metadata::{
    get_format_from_file,
    get_format_from_slice,
};
pub use types::{
    Error,
    Metadata,
    Size,
};

pub mod enums;
pub mod video_metadata;
pub mod types;
