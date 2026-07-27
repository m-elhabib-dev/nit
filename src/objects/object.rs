use std::fmt;

#[derive(Debug, PartialEq, Eq)]
pub enum Kind {
    Blob,
    Tree,
    Commit,
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Kind::Blob => write!(f, "blob"),
            Kind::Tree => write!(f, "tree"),
            Kind::Commit => write!(f, "commit"),
        }
    }
}

pub struct Object<R> {
    pub kind: Kind,
    pub expected_size: usize,
    pub reader: R,
}
