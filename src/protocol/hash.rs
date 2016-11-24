/*! hash protocol

Originally, git has been using SHA1 to generate unique identifier (ref)
for objects, commits. In order to prepare the field for a other Hash algorithm
(but also to be able to mock when testing) we provide a protocol for hashing.

# Common interface

The idea is to provide a simple interface so we can easily change/customise
the hashing mechanism later on.

# Example

```
use git::protocol::hash::{SHA1, Hasher};

let data = "hello world";
let hash = SHA1::hash(&mut data.as_bytes()).unwrap();
assert_eq!(hash.to_hexadecimal(), "2aae6c35c94fcfb415dbe95f408b9ce91ee846ed");
```

!*/

use nom;

extern crate crypto;
use self::crypto::digest::Digest;
use self::crypto::sha1::Sha1;
extern crate rustc_serialize;
use self::rustc_serialize::hex::{FromHex, ToHex};
use std::io::{Result, BufRead};
use std::{str, io, fmt};

/// Hasher Protocol
///
/// simple interface to hash different stream
///
pub trait Hasher : Sized {
    /// function to hash a stream
    fn hash<R: BufRead>(data: &mut R) -> Result<Self>;

    fn from_bytes(Vec<u8>) -> Option<Self>;
    #[inline]
    fn from_hex(s: &str) -> Option<Self> {
        if let Ok(b) = s.from_hex() {
            Self::from_bytes(b)
        } else { None }
    }

    /// the size of the digest
    #[inline]
    fn digest_size() -> usize;

    /// the size of the digest in hexadecimal
    #[inline]
    fn digest_hex_size() -> usize { Self::digest_size() * 2}

    /// return exadecimal encoded of the digest
    #[inline]
    fn to_hexadecimal(&self) -> String { self.as_bytes().to_hex().to_string() }

    #[inline]
    fn as_bytes(&self) -> &[u8];

    #[inline]
    fn decode_bytes(i: &[u8]) -> nom::IResult<&[u8], Self> { decode_bytes_(i) }
    #[inline]
    fn encode_bytes<W: io::Write>(&self, w: &mut W) -> io::Result<usize> { encode_bytes_(self, w) }
    #[inline]
    fn decode_hex(i: &[u8]) -> nom::IResult<&[u8], Self> { decode_hex_(i) }
    #[inline]
    fn encode_hex<W: io::Write>(&self, w: &mut W) -> io::Result<usize> { encode_hex_(self, w) }
}

/// Hash SHA1.
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct SHA1(Vec<u8>);
impl Hasher for SHA1 {
    #[inline]
    fn from_bytes(b: Vec<u8>) -> Option<Self> {
        if b.len() == Self::digest_size() {
            Some(SHA1(b))
        } else { None }
    }
    fn hash<R: BufRead>(data: &mut R) -> Result<Self> {
        let mut st = Sha1::new();
        let mut buf : &mut [u8;128] = &mut [0u8;128];
        let mut res = [0;128];

        loop {
            let n = try!(data.read(buf));
            if n == 0 { break; }
            st.input(&buf[0..n]);
        }

        st.result(&mut res);
        Ok(SHA1(res[0..20].iter().cloned().collect()))
    }
    #[inline]
    fn digest_size() -> usize { 20 }

    #[inline]
    fn to_hexadecimal(&self) -> String { self.0.as_slice().to_hex().to_string() }

    #[inline]
    fn as_bytes(&self) -> &[u8] { self.0.as_slice() }
}
impl fmt::Display for SHA1 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{}", self.to_hexadecimal()) }
}

fn decode_bytes_<H: Hasher>(i: &[u8]) -> nom::IResult<&[u8], H> {
    let size = H::digest_size();
    let input = &i[..size];
    let output = match H::from_bytes(input.iter().cloned().collect()) {
        Some(output) => output,
        None => {
            return nom::IResult::Incomplete(nom::Needed::Size(size));
        }
    };
    let remain = &i[size..];

    nom::IResult::Done(remain, output)
}
fn encode_bytes_<H, W>(hash: &H, writer: &mut W) -> io::Result<usize>
  where H: Hasher
      , W: io::Write
{
    writer.write_all(hash.as_bytes()).map(|()| H::digest_size())
}

fn decode_hex_<H: Hasher>(i: &[u8]) -> nom::IResult<&[u8], H> {
    let (i, bytes) = match nom_parse_hex(i) {
        nom::IResult::Done(i, b) => (i, b),
        nom::IResult::Error(err) => return nom::IResult::Error(err),
        nom::IResult::Incomplete(n) => return nom::IResult::Incomplete(n)
    };
    let output = String::from_utf8(bytes.iter().cloned().collect())
                    .expect("nom_parse_hex should have only parsed valid hexadecimal");
    let output = match H::from_hex(output.as_ref()) {
        Some(output) => output,
        None => {
            return nom::IResult::Incomplete(nom::Needed::Size(H::digest_hex_size()));
        }
    };

    nom::IResult::Done(i, output)
}
named!(nom_parse_hex, take_while1!(nom::is_hex_digit));
pub fn encode_hex_<H, W>(hash: &H, writer: &mut W) -> io::Result<usize>
  where H: Hasher
      , W: io::Write
{
    writer.write_all(hash.to_hexadecimal().as_bytes()).map(|()| H::digest_size())
}

#[cfg(test)]
mod test {
    //! contract test. It's more to detect changes and make sure
    //! things don't break under our feet without knowing it.

    use super::*;
    use nom;
    use std::io;
    use ::protocol::encoder::Encoder;
    use ::protocol::decoder::Decoder;
    use ::protocol::test_encoder_decoder;

    #[test]
    fn sha1_empty() {
        let data = String::new();
        let hash = SHA1::hash(&mut data.as_bytes()).unwrap();
        assert_eq!(hash.to_hexadecimal(), "da39a3ee5e6b4b0d3255bfef95601890afd80709");
    }

    #[test]
    fn sha1_basic() {
        let data = "hello world";
        let hash = SHA1::hash(&mut data.as_bytes()).unwrap();
        assert_eq!(hash.to_hexadecimal(), "2aae6c35c94fcfb415dbe95f408b9ce91ee846ed");
    }

    #[derive(PartialEq, Eq, Debug)]
    struct Bytes<H: Hasher>(H);
    impl<H: Hasher> Hasher for Bytes<H> {
        fn hash<R: io::BufRead>(data: &mut R) -> io::Result<Self> {
            H::hash(data).map(|h| Bytes(h))
        }
        fn from_bytes(v: Vec<u8>) -> Option<Self> {
            H::from_bytes(v).map(|h| Bytes(h))
        }
        fn digest_size() -> usize { H::digest_size() }
        fn as_bytes(&self) -> &[u8] { self.0.as_bytes() }
    }
    impl<H: Hasher> Encoder for Bytes<H> {
        fn encode<W: io::Write>(&self, w: &mut W) -> io::Result<usize> {
            self.encode_bytes(w)
        }
    }
    impl<H: Hasher> Decoder for Bytes<H> {
        fn decode(i: &[u8]) -> nom::IResult<&[u8], Self> {
            Self::decode_bytes(i)
        }
    }


    #[derive(PartialEq, Eq, Debug)]
    struct Hex<H: Hasher>(H);
    impl<H: Hasher> Hasher for Hex<H> {
        fn hash<R: io::BufRead>(data: &mut R) -> io::Result<Self> {
            H::hash(data).map(|h| Hex(h))
        }
        fn from_bytes(v: Vec<u8>) -> Option<Self> {
            H::from_bytes(v).map(|h| Hex(h))
        }
        fn digest_size() -> usize { H::digest_size() }
        fn as_bytes(&self) -> &[u8] { self.0.as_bytes() }
    }
    impl<H: Hasher> Encoder for Hex<H> {
        fn encode<W: io::Write>(&self, w: &mut W) -> io::Result<usize> {
            self.encode_hex(w)
        }
    }
    impl<H: Hasher> Decoder for Hex<H> {
        fn decode(i: &[u8]) -> nom::IResult<&[u8], Self> {
            Self::decode_hex(i)
        }
    }

    #[test]
    fn sha1_hex_serialisable() {
        let sha1_hex = "2aae6c35c94fcfb415dbe95f408b9ce91ee846ed";
        let sha1 = Hex::<SHA1>::from_hex(sha1_hex)
                        .expect("expecting a valid SHA1 encoded in hexadecimal");
        test_encoder_decoder(sha1);
    }
    #[test]
    fn sha1_bytes_serialisable() {
        let sha1_hex = "2aae6c35c94fcfb415dbe95f408b9ce91ee846ed";
        let sha1 = Bytes::<SHA1>::from_hex(sha1_hex)
                        .expect("expecting a valid SHA1 encoded in bytes");
        test_encoder_decoder(sha1);
    }
}
