use std::{error::Error, fmt, io};

#[derive(Debug)]
pub enum KeySetError {
    InvalidInput,
    HasOnlyValid,
    HasInvalid,
}

impl fmt::Display for KeySetError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            KeySetError::InvalidInput => return write!(f, "Invalid key"),
            KeySetError::HasOnlyValid => return write!(f, "Single Key Slot"),
            KeySetError::HasInvalid => return write!(f, "Contains Invalid Key"),
        }
    }
}

// TODO: Should these return info on why they tripped?
impl Error for KeySetError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        return match self {
            KeySetError::InvalidInput | KeySetError::HasOnlyValid | KeySetError::HasInvalid => {
                None
            }
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
