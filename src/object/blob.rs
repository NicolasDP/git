use protocol::hash::Hasher;
use protocol::encoder::Encoder;
use protocol::decoder::Decoder;
use std::{io, fmt, str, convert};
use nom;

/// Blob reference
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
pub struct BlobRef<H: Hasher>(H);
impl<H: Hasher> BlobRef<H> {
    pub fn new(h: H) -> Self { BlobRef(h) }
}
impl<H: Hasher + fmt::Display> fmt::Display for BlobRef<H> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { fmt::Display::fmt(&self.0, f) }
}

impl<H: Hasher> Hasher for BlobRef<H> {
    fn hash<R: io::BufRead>(data: &mut R) -> io::Result<Self> {
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
impl<H: Hasher> convert::AsRef<H> for BlobRef<H> {
    fn as_ref(&self) -> &H { &self.0 }
}


#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
pub struct Blob(Vec<u8>);
impl Blob {
    pub fn new(data: Vec<u8>) -> Self { Blob(data) }

    pub fn hash<H: Hasher>(&self) -> H {
        H::hash(&mut self.0.as_slice()).unwrap()
    }
}
impl Decoder for Blob {
    fn decode(b: &[u8]) -> nom::IResult<&[u8], Self> {
        let (i, size) = match nom_parse_blob(b) {
            nom::IResult::Done(i, b) => (i, b),
            nom::IResult::Error(err) => return nom::IResult::Error(err),
            nom::IResult::Incomplete(n) => return nom::IResult::Incomplete(n)
        };
        if i.len() < size {
            return nom::IResult::Incomplete(nom::Needed::Size(size - i.len()));
        }
        nom::IResult::Done(&i[size..], Blob::new(i[..size].iter().cloned().collect()))
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
