use std::{error::Error, fmt, io};

#[derive(Debug)]
pub enum KeySetError {
    InvalidKey,
    SingleKeySlot,
}

impl fmt::Display for KeySetError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            KeySetError::InvalidKey => return write!(f, "Invalid key"),
            KeySetError::SingleKeySlot => return write!(f, "Single Key Slot"),
        }
    }
}

// TODO: Should these return info on why they tripped?
impl Error for KeySetError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        return match self {
            KeySetError::InvalidKey | KeySetError::SingleKeySlot => None,
        };
    }
}

#[derive(Debug)]
pub enum CorpusErr {
    EmptyCorpus,
    Io(io::Error),
}

impl fmt::Display for CorpusErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CorpusErr::EmptyCorpus => return write!(f, "No files in corpus"),
            CorpusErr::Io(e) => return write!(f, "IO error: {}", e,),
        }
    }
}

impl Error for CorpusErr {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        return match self {
            CorpusErr::EmptyCorpus => None,
            CorpusErr::Io(e) => Some(e),
        };
    }
}

impl From<io::Error> for CorpusErr {
    fn from(error: io::Error) -> Self {
        return CorpusErr::Io(error);
    }
}
