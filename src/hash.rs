use std::marker::PhantomData;
use std::str::{FromStr, from_utf8_unchecked};
use std::convert::From;
use std::fmt;
use std::path::PathBuf;
extern crate rustc_serialize;
use self::rustc_serialize::hex::{FromHex, ToHex};

use error::{GitError, Result};

extern crate crypto;
use self::crypto::digest::Digest;
use self::crypto::sha1::Sha1;

use objectable::{Readable, Writable};
use nom;

pub trait Property {
    const DIGEST_SIZE: usize;
    const PREFIX_SIZE: usize;

    fn new() -> Self;
    fn write<T: AsRef<[u8]>>(&mut self, bytes: T);
    fn finish(&mut self) -> Vec<u8>;
}
pub trait Hashable {
    fn get_chunk<'a>(&'a self, usize) -> &'a [u8];
    fn hash<Hash : Property> (&self) -> HashRef<Hash> {
        let mut hs = Hash::new();
        let mut i = 0;
        loop {
            let data = self.get_chunk(i);
            if (data.len() == 0) || (i >= 100) { break }
            i += 1;
            hs.write(data)
        }
        HashRef::new_with(&hs.finish())
    }
}

pub struct SHA1 {
  state: crypto::sha1::Sha1,
}
impl PartialEq for SHA1 {
    fn eq(&self, _: &Self) -> bool {true}
}
impl fmt::Debug for SHA1 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SHA1")
    }
}
impl Property for SHA1 {
    fn new() -> Self { SHA1 { state: Sha1::new() } }
    fn write<T: AsRef<[u8]>>(&mut self, bytes: T) {
        self.state.input(bytes.as_ref());
    }
    fn finish(&mut self) -> Vec<u8> {
        let mut out = vec![0u8;SHA1::DIGEST_SIZE];
        self.state.result(out.as_mut_slice());
        out
    }
    const DIGEST_SIZE: usize = 20;
    const PREFIX_SIZE: usize = 1;
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct HashRef<Hash : Property> {
    hash: Vec<u8>,
    _hash_type: PhantomData<Hash>,
}
impl<Hash : Property> Clone for HashRef<Hash> {
    fn clone(&self) -> Self { Self::new_with(&self.hash) }
}
pub trait HasHashRef<Hash: Property> {
    fn hash_ref(&self) -> HashRef<Hash>;
}
impl<Hash: Property> HasHashRef<Hash> for HashRef<Hash> {
    fn hash_ref(&self) -> HashRef<Hash> { self.clone() }
}

impl<Hash : Property> HashRef<Hash> {
    pub fn new() -> Self {
        HashRef
            { hash       : Vec::with_capacity(Hash::DIGEST_SIZE)
            , _hash_type : PhantomData
            }
    }
    pub fn new_with<T: AsRef<[u8]>>(data: T) -> Self {
        if data.as_ref().len() != Hash::DIGEST_SIZE { panic!("invalid size"); }
        let mut v = Vec::with_capacity(Hash::DIGEST_SIZE);
        v.extend_from_slice(data.as_ref());
        HashRef
            { hash : v.clone()
            , _hash_type : PhantomData
            }
    }

    pub fn capacity(&self)    -> usize { self.hash.capacity() }
    pub fn digest_size(&self) -> usize { Hash::DIGEST_SIZE }
    pub fn prefix_size(&self) -> usize { Hash::PREFIX_SIZE }
    pub fn digest(&self)      -> &[u8] { self.hash.as_slice() }
    pub fn prefix(&self)      -> &[u8] { &self.digest()[..self.prefix_size()] }
    pub fn loose(&self)       -> &[u8] { &self.digest()[self.prefix_size()..] }

    pub fn path(&self) -> PathBuf {
        PathBuf::new()
            .join(self.prefix().to_hex())
            .join(self.loose().to_hex())
    }
}

impl<Hash: Property> fmt::Display for HashRef<Hash> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.digest().to_hex())
    }
}
impl<Hash: Property> Writable for HashRef<Hash> {
    fn provide_size(&self) -> usize { Hash::DIGEST_SIZE * 2 }
    fn serialise(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.digest().to_hex())
    }
}
impl<Hash: Property> Readable for HashRef<Hash> {
    fn nom_parse(b: &[u8]) -> nom::IResult<&[u8], Self> {
        let hex_size = Hash::DIGEST_SIZE * 2;
        if b.len() < hex_size {
            return nom::IResult::Incomplete(nom::Needed::Size(hex_size));
        }
        let hex_str = unsafe { from_utf8_unchecked(&b[..hex_size]) };
        let remain  = &b[hex_size..];

        match hex_str.from_hex() {
            Ok(v)  => nom::IResult::Done(remain, HashRef::from(v)),
            Err(_) => nom::IResult::Error(nom::Err::Code(nom::ErrorKind::HexDigit))
        }
    }
}

impl<Hash: Property> FromStr for HashRef<Hash> {
    type Err = GitError;
    fn from_str(s: &str) -> Result<Self> {
        s.from_hex()
            .map_err(|err| GitError::Unknown(format!("{}", err)))
            .and_then(|v| {
                if v.len() != Hash::DIGEST_SIZE {
                    Err(GitError::Unknown(format!("{}, expecting length {}", s, Hash::DIGEST_SIZE)))
                } else {
                    Ok (HashRef::new_with(&v))
                }
            })
    }
}

impl<Hash: Property> From<Vec<u8>> for HashRef<Hash> {
    fn from(data: Vec<u8>) -> Self { HashRef { hash: data, _hash_type: PhantomData } }
}
impl<'a> From<&'a Vec<u8>> for HashRef<SHA1> {
    fn from(data:&'a Vec<u8>) -> Self { HashRef::new_with(data) }
}
impl<'a> From<&'a [u8;20]> for HashRef<SHA1> {
    fn from(data:&'a [u8;20]) -> Self {
        let mut r = HashRef::new();
        r.hash.extend_from_slice(data);
        r
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hash;

    struct MockHashable {
        data : String
    }
    impl MockHashable {
        fn new(d: &str) -> Self { MockHashable { data : d.to_string() } }
    }
    impl hash::Hashable for MockHashable {
        fn get_chunk(&self, count: usize) -> &[u8] {
            if count > 0 { return &b""[..] }
            self.data.as_bytes()
        }
    }

    #[test]
    fn capacity_equals_hash_size() {
        let r : HashRef<hash::SHA1> = HashRef::new();
        assert_eq!(20, r.digest_size());
        assert_eq!(1,  r.prefix_size());
        assert!(r.prefix_size() <= r.digest_size())
    }

    #[test]
    fn set_hash_ok() {
        let data = MockHashable::new("The quick brown fox jumps over the lazy cog");
        let r : HashRef<hash::SHA1> = data.hash();
        assert_eq!(r.capacity(), r.digest_size())
    }
    #[test]
    fn prefix_ok() {
        let data = MockHashable::new("The quick brown fox jumps over the lazy dog");
        let expected_digest = [47, 212, 225, 198, 122, 45, 40, 252, 237, 132, 158, 225, 187, 118, 231, 57, 27, 147, 235, 18];
        let expected_prefix = &expected_digest[..1];
        let expected_loose  = &expected_digest[1..];
        let r : HashRef<hash::SHA1> = data.hash();
        assert_eq!(expected_prefix, r.prefix());
        assert_eq!(expected_loose,  r.loose());
        assert_eq!(expected_digest, r.digest())
    }
}
