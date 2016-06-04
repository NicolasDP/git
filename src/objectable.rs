use error::*;
use std::fmt;
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
pub trait Readable : Sized {
    fn nom_parse(b: &[u8]) -> nom::IResult<&[u8], Self>;
    fn parse_bytes(b: &[u8]) -> Result<Self> {
        match <Self as Readable>::nom_parse(b) {
            IResult::Done(_, c) => Ok(c),
            IResult::Error(err) => Err(GitError::ParsingErrorUnknown(format!("{:?}", err))),
            IResult::Incomplete(Needed::Unknown) => Err(GitError::ParsingErrorNotEnough(None)),
            IResult::Incomplete(Needed::Size(s)) => Err(GitError::ParsingErrorNotEnough(Some(s)))
        }
    }
}

pub trait Writable : Sized {
    fn serialise(&self, f: &mut fmt::Formatter) -> fmt::Result;
    fn provide_size(&self) -> usize;
}

pub trait Objectable : Readable + Writable { }

impl Writable for String {
    fn serialise(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{}", self) }
    fn provide_size(&self) -> usize { self.len() }
}
impl<'a> Writable for &'a str {
    fn serialise(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{}", self) }
    fn provide_size(&self) -> usize { self.len() }
}
#[inline(always)]
fn count_number_of_digit(s_: usize) -> usize {
    let mut s = s_;
    let mut n = 0;
    while s > 0 {
        s = s / 10;
        n += 1;
    }
    n
}
impl Writable for usize {
    fn serialise(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{}", self) }
    fn provide_size(&self) -> usize { count_number_of_digit(*self) }
}

macro_rules! encode_obj

#[macro_export]
macro_rules! serialise {
    ($fmt:expr, $($arg:expr),+) => {{
        $(
            match $arg.serialise($fmt) {
                Ok(_) => (),
                Err(err) => return Err(err)
            };
        )+
        Ok(())
    }};
}
