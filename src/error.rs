use std::path::PathBuf;
use std::result;
//use std::io;
use std::fmt::Display;
use std::fmt;
use std::error::Error;

use refs::RefName;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum GitError {
    MissingDirectory(PathBuf),
    MissingFile(PathBuf),
    InvalidRef(RefName),
    InvalidBranch(RefName),
    InvalidTag(RefName),
    InvalidRemote(RefName),
    ParsingErrorNotEnough(Option<usize>),
    ParsingErrorUnknown(String),
    IoError(String),
    Unknown(String)
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
