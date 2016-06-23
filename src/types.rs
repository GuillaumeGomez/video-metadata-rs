use std::time::Duration;
use {AudioCodec, KnownTypes, VideoCodec};

#[derive(Clone, Debug)]
pub struct Size {
    pub width: u16,
    pub height: u16,
}

#[derive(Clone, Debug)]
pub struct Metadata {
    pub video_name: String,
    pub format: KnownTypes,
    pub len: Duration,
    pub size: Size,
    pub video: VideoCodec,
    pub audio: Option<AudioCodec>,
}
