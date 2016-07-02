use types::Metadata;

#[derive(Clone, Debug, PartialEq)]
pub enum Result {
    Complete(Metadata),
    Unknown(String),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum KnownTypes {
    WebM,
    MP4,
    Ogg,
}

impl KnownTypes {
    pub fn from(s: &str) -> Option<KnownTypes> {
        let formats = [("webm", KnownTypes::WebM),
                       ("mp4", KnownTypes::MP4),
                       ("ogg", KnownTypes::Ogg)];
        let s = s.to_lowercase();
        for &(ref key, ref format) in formats.iter() {
            if s.contains(key) {
                return Some(*format);
            }
        }
        None
    }
}
