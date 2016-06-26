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

impl KnownTypes {
    pub fn from(s: &str) -> Option<KnownTypes> {
        if s.contains("webm") {
            Some(KnownTypes::Webm)
        } else {
            None
        }
    }
}
