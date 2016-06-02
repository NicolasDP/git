use error::*;
use std::fmt::Display;
use std::str;

use nom;
use nom::{IResult, Needed};

/// Property for object that can be saved into a Git Object
///
/// # TODO
///
/// we could consider that an Objectable must be Hashable
/// or more simply that every Objectable is Hashable as soon
/// as it is possible to collect chunks of it.
///
pub trait Objectable : Sized + Display {
    fn nom_parse(b: &[u8]) -> nom::IResult<&[u8], Self>;
    fn provide_size(&self) -> usize;

    fn parse_bytes(b: &[u8]) -> Result<Self> {
        match <Self as Objectable>::nom_parse(b) {
            IResult::Done(_, c) => Ok(c),
            IResult::Error(err) => Err(GitError::ParsingErrorUnknown(format!("{:?}", err))),
            IResult::Incomplete(Needed::Unknown) => Err(GitError::ParsingErrorNotEnough(None)),
            IResult::Incomplete(Needed::Size(s)) => Err(GitError::ParsingErrorNotEnough(Some(s)))
        }
    }
}
