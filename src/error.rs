use std::path::PathBuf;
use std::fmt::Display;
use std::{io, result, fmt};
use std::error::Error;

use refs::RefName;

#[derive(Debug)]
pub enum GitError {
    InvalidHashSize(usize, usize),
    MissingDirectory(PathBuf),
    MissingFile(PathBuf),
    InvalidRef(RefName),
    InvalidBranch(RefName),
    InvalidTag(RefName),
    InvalidRemote(RefName),
    ParsingErrorNotEnough(Option<usize>),
    ParsingError(String),
    IoError(io::Error),
    Other(String),
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
