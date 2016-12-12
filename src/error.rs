use std::path::PathBuf;
use std::fmt::Display;
use std::{io, result, fmt};
use std::error::Error;

use refs::RefName;

/// *try* the IO operation, wrap the IOError in a GitError if failed
macro_rules! io_try {
    ($expression:expr) => ({
        use ::error::{GitError};
        match $expression {
            Ok(v) => v,
            Err(err) => return Err(GitError::ioerror(err))
        }
    })
}

/// *try* to run a nom parser, wrap the Nom's Error in a GitError if failed
macro_rules! nom_try {
    ($expression:expr) => ({
        use nom::{IResult, Needed};
        use ::error::{GitError};
        match $expression {
            IResult::Done(_, v) => v,
            IResult::Incomplete(Needed::Unknown) => {
                return Err(GitError::ParsingErrorNotEnough(None))
            },
            IResult::Incomplete(Needed::Size(s)) => {
                return Err(GitError::ParsingErrorNotEnough(Some(s)))
            },
            IResult::Error(err) => {
                return Err(GitError::ParsingError(format!("{:?}", err).to_string()))
            }
        }
    })
}

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
