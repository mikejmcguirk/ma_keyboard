// use std::{error::Error, fmt, io};
use core::{error::Error, fmt};
use std::io;

#[derive(Debug)]
pub enum CorpusErr {
    EmptyCorpus,
    Io(io::Error),
}

#[expect(clippy::pattern_type_mismatch)] // Unable to unwind this because of borrowing rules
impl fmt::Display for CorpusErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CorpusErr::EmptyCorpus => return write!(f, "No files in corpus"),
            CorpusErr::Io(e) => return write!(f, "IO error: {}", e,),
        }
    }
}

impl Error for CorpusErr {
    #[expect(clippy::pattern_type_mismatch)] // Unable to unwind this because of borrowing rules
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            CorpusErr::EmptyCorpus => return None,
            CorpusErr::Io(e) => return Some(e),
        }
    }

    fn cause(&self) -> Option<&dyn Error> {
        return self.source();
    }

    #[expect(clippy::pattern_type_mismatch)] // Unable to unwind this because of borrowing rules
    fn description(&self) -> &str {
        match self {
            CorpusErr::EmptyCorpus => return "corpus is empty",
            CorpusErr::Io(_) => return "IO error",
        }
    }
}

impl From<io::Error> for CorpusErr {
    fn from(error: io::Error) -> Self {
        return CorpusErr::Io(error);
    }
}
