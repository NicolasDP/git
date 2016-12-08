use std::{fmt, convert, io, path};
use std::collections::BTreeSet;
use nom;

use ::protocol::Hash;
use error::{Result, GitError};
use super::util::get_all_files_in;
use super::GitFS;

pub mod index;

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
pub struct PackRef<H: Hash>(H);
impl<H: Hash> PackRef<H> {
    pub fn new(h: H) -> Self { PackRef(h) }
}
impl<H: Hash + fmt::Display> fmt::Display for PackRef<H> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { fmt::Display::fmt(&self.0, f) }
}
impl<H: Hash> Hash for PackRef<H> {
    fn hash<R: io::BufRead>(data: &mut R) -> io::Result<Self> {
        H::hash(data).map(|h| PackRef(h))
    }

    fn from_bytes(v: Vec<u8>) -> Option<Self> {
        H::from_bytes(v).map(|h| PackRef(h))
    }

    #[inline]
    fn digest_size() -> usize { H::digest_size() }

    #[inline]
    fn as_bytes(&self) -> &[u8] { self.0.as_bytes() }
}
impl<H: Hash> convert::AsRef<H> for PackRef<H> {
    fn as_ref(&self) -> &H { &self.0 }
}
