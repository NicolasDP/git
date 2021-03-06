//! Git's Blob (i.e. file)

use protocol::{Encoder, Decoder, Hash};
use std::{io, fmt, str, convert};
use nom;
use error::Result;

/// Blob reference
///
/// This is simply a strongly typed version of the `Hash` given Hash
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
pub struct BlobRef<H: Hash>(H);
impl<H: Hash> BlobRef<H> {
    pub fn new(h: H) -> Self { BlobRef(h) }
}
impl<H: Hash + fmt::Display> fmt::Display for BlobRef<H> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { fmt::Display::fmt(&self.0, f) }
}

impl<H: Hash> Hash for BlobRef<H> {
    fn hash<R: io::BufRead>(data: &mut R) -> Result<Self> {
        H::hash(data).map(|h| BlobRef(h))
    }

    fn from_bytes(v: Vec<u8>) -> Option<Self> {
        H::from_bytes(v).map(|h| BlobRef(h))
    }

    #[inline]
    fn digest_size() -> usize { H::digest_size() }

    #[inline]
    fn as_bytes(&self) -> &[u8] { self.0.as_bytes() }
}
impl<H: Hash> convert::AsRef<H> for BlobRef<H> {
    fn as_ref(&self) -> &H { &self.0 }
}

/// `Blob` data
///
/// So far this is only in-memory data but it should become something more
/// efficient in near future.
///
/// A `Blob` is referenced by a `BlobRef<H>`.
///
/// # Discussion
///
/// This part is still under construction and the API might (certainly) change.
/// So far we are storing everything in memory (a Vec<u8>). It is does not make
/// much sense to do so as we could blow the memory and be quite slow to process
/// the data.
///
/// Ideally, in the future, the Blob may become a `trait` so we could use
/// streamable objects or in memory data depending on what is better.
///
/// The composition of a Blob may also differes depending of the backend
/// in use. So far we will use in the filesystem as it is the legacy one
/// but ideally we could change the backend to a key-value database which
/// won't be any different.
///
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
pub struct Blob(Vec<u8>);
impl Blob {
    /// create a blob from the given data.
    pub fn new(data: Vec<u8>) -> Self { Blob(data) }

    /// access the inner data as an immutable slice of bytes
    pub fn as_slice(&self) -> &[u8] { self.0.as_slice() }
}
impl Decoder for Blob {
    fn decode(b: &[u8]) -> nom::IResult<&[u8], Self> {
        let (b, size) = try_parse!(b, nom_parse_blob);
        if b.len() < size {
            return nom::IResult::Incomplete(nom::Needed::Size(size - b.len()));
        }
        nom::IResult::Done(
            &b[size..],
            Blob::new(b[..size].iter().cloned().collect())
        )
    }
}
impl fmt::Display for Blob {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", unsafe { String::from_utf8_unchecked(self.0.clone()) } )
    }
}
impl Encoder for Blob {
    fn required_size(&self) -> usize { self.0.len() }
    fn encode<W: io::Write>(&self, writer: &mut W) -> io::Result<usize> {
        let header = format!("blob {}\0", self.0.len());
        try!(writer.write_all(header.as_bytes()));
        try!(writer.write_all(self.0.as_slice()));
        Ok(header.len() + self.0.len())
    }
}

named!(nom_parse_blob_tag, tag!("blob "));
named!(nom_parse_blob_size<usize>
      , map_res!( map_res!( nom::digit, str::from_utf8), str::FromStr::from_str)
      );
named!(nom_parse_blob<usize>
      , chain!(nom_parse_blob_tag ~ r: nom_parse_blob_size ~ char!('\0'), || r)
      );

// -- --------------------------------------------------------------------- --
// --                                 Tests                                 --
// -- --------------------------------------------------------------------- --

#[cfg(test)]
mod test {
    use super::*;
    use ::protocol::test_encoder_decoder;

    #[test]
    fn blob_serialisable() {
        let data = (0x00u8..0xff).collect();
        let blob = Blob::new(data);
        test_encoder_decoder(blob);
    }
}
