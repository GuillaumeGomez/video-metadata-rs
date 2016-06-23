use types::Metadata;

#[derive(Clone, Debug)]
pub enum Result {
    Complete(Metadata),
    Incomplete(Vec<KnownTypes>),
    Unknown,
}

#[derive(Clone, Debug)]
pub enum KnownTypes {
    Webm,
}

#[derive(Clone, Debug)]
pub enum VideoCodec {
    VP8,
    VP9,
}

#[derive(Clone, Debug)]
pub enum AudioCodec {
    Opus,
    Vorbis,
}
