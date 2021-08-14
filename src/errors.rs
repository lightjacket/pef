use std::fmt::{Display, Formatter, Debug};

pub enum ErrorKind {
    UnsortedIds,
    NoIds
}

pub struct Error {
    error: ErrorKind,
}

impl Error {
    pub fn unsorted_ids() -> Self {
        Self { error: ErrorKind::UnsortedIds }
    }

    pub fn no_ids() -> Self {
        Self { error: ErrorKind::NoIds }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.error {
            ErrorKind::UnsortedIds => write!(f, "Unsorted ids cannot be compressed. Please sort."),
            ErrorKind::NoIds => write!(f, "Emptys ids cannot be compressed.")
        }
    }
}


impl Debug for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self, f)
    }
}

impl std::error::Error for Error {}