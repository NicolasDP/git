use std::path::PathBuf;
use std::fmt::Display;
use std::{io, result, fmt};
use std::error::Error;

use refs::RefName;

#[derive(PartialEq, Debug)]
pub enum GitError {
    OutOfBound(usize, usize),
    InvalidHashSize(usize, usize),
    MissingDirectory(PathBuf),
    MissingFile(PathBuf),
    InvalidRef(RefName),
    InvalidBranch(RefName),
    InvalidTag(RefName),
    InvalidRemote(RefName),
    ParsingErrorNotEnough(Option<usize>),
    ParsingError(String),
    IoError(String),
    Other(String),
    Unknown(String)
}
impl GitError {
    #[inline(always)]
    pub fn ioerror(err: io::Error) -> Self {
        GitError::IoError(format!("{:?}", err))
    }
}

impl Display for GitError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for GitError {
    fn description(&self) -> &str { "Git Manipulation Error" }
}

pub type Result<T> = result::Result<T, GitError>;
