// use std::{error::Error, fmt, io};
use core::{error::Error, fmt};
use std::io;

#[derive(Debug)]
pub enum CorpusErr {
    EmptyCorpus,
    Io(io::Error),
}

// TODO: Is this okay?
#[expect(clippy::pattern_type_mismatch)]
impl fmt::Display for CorpusErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CorpusErr::EmptyCorpus => return write!(f, "No files in corpus"),
            CorpusErr::Io(e) => return write!(f, "IO error: {}", e,),
        }
    }
}

// TODO: Is this okay?
#[expect(clippy::pattern_type_mismatch)]
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
