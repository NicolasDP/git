use nom;
use std::{fmt, str};
use std::marker::PhantomData;
use std::path::PathBuf;
use std::convert::TryFrom;
extern crate rustc_serialize;
use self::rustc_serialize::hex::{FromHex, ToHex};

use ::error::{GitError, Result};
use ::objectable::{Readable, Writable};
// TODO remove SHA1 from the dependency
pub use ::hash::{Property, SHA1};

/// Git Hash object identifier
///
/// # Create a HashRef
///
/// Creating a new instance of HashRef is limited in order
/// to provide some limited possibilities to hold on
///
/// ## From bytes
///
/// ```
/// #![feature(try_from)]
/// use git::object::elements::hash::*;
/// use std::convert::TryFrom;
///
/// let v : Vec<u8> = vec![1;20];
/// let hash : HashRef<SHA1> = HashRef::try_from(&v).unwrap();
/// println!("hash: {}", hash);
/// ```
///
/// #
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct HashRef<Hash : Property> {
    hash: Vec<u8>,
    _hash_type: PhantomData<Hash>,
}
impl<Hash : Property> HashRef<Hash> {
    // This function is not exported as it is not valid to create
    // an empty hash reference.
    //
    // /!\ use internally only
    fn new() -> Self {
        HashRef
            { hash       : Vec::with_capacity(Hash::DIGEST_SIZE)
            , _hash_type : PhantomData
            }
    }

    // create a HashRef from an array of bytes
    //
    // the given input must be an already generated Hash
    // for the associated type.
    //
    // use TryFrom::try_from instead
    fn new_with<T: AsRef<[u8]>>(data: T) -> Self {
        let mut v = Vec::with_capacity(Hash::DIGEST_SIZE);
        v.extend_from_slice(data.as_ref());
        HashRef
            { hash : v.clone()
            , _hash_type : PhantomData
            }
    }

    /// little helper to get the capacity of the underlying vector of the
    /// HashRef. This function should always returned you: Hash::DIGEST_SIZE
    ///
    /// This funciton is used for test only in order to verify that the size
    /// of the HashRef is always not much than the size of the DIGEST_SIZE.
    pub fn capacity(&self)    -> usize { self.hash.capacity() }

    /// alias function to get the digest size of the current HashRef.
    /// equivalent to Hash::PREFIX_SIZE
    pub fn digest_size(&self) -> usize { Hash::DIGEST_SIZE }

    /// alias function to get the prefix size of the current HashRef.
    pub fn prefix_size(&self) -> usize { Hash::PREFIX_SIZE }

    pub fn prefix(&self)      -> &[u8] { &self.hash[..self.prefix_size()] }
    pub fn loose(&self)       -> &[u8] { &self.hash[self.prefix_size()..] }

    pub fn path(&self) -> PathBuf {
        PathBuf::new()
            .join(self.prefix().to_hex())
            .join(self.loose().to_hex())
    }
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
impl<Hash: Property> AsRef<[u8]> for HashRef<Hash> {
    fn as_ref(&self) -> &[u8] { self.hash.as_ref() }
}

impl<Hash: Property> fmt::Display for HashRef<Hash> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.hash.to_hex())
    }
}
impl<Hash: Property> Writable for HashRef<Hash> {
    fn provide_size(&self) -> usize { Hash::DIGEST_SIZE * 2 }
    fn serialise(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.hash.to_hex())
    }
}
impl<Hash: Property> Readable for HashRef<Hash> {
    fn nom_parse(b: &[u8]) -> nom::IResult<&[u8], Self> {
        let hex_size = Hash::DIGEST_SIZE * 2;
        if b.len() < hex_size {
            return nom::IResult::Incomplete(nom::Needed::Size(hex_size));
        }
        let hex_str = unsafe { str::from_utf8_unchecked(&b[..hex_size]) };
        let remain  = &b[hex_size..];

        match hex_str.from_hex() {
            Ok(v)  => nom::IResult::Done(remain, HashRef::from(v)),
            Err(_) => nom::IResult::Error(nom::Err::Code(nom::ErrorKind::HexDigit))
        }
    }
}

impl<Hash: Property> str::FromStr for HashRef<Hash> {
    type Err = GitError;
    fn from_str(s: &str) -> Result<Self> {
        s.from_hex()
            .map_err(|err| GitError::Unknown(format!("{}", err)))
            .and_then(|v| {
                if v.len() != Hash::DIGEST_SIZE {
                    Err(GitError::InvalidHashSize(Hash::DIGEST_SIZE, v.len()))
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
impl<'a, T: AsRef<[u8]>, Hash: Property> TryFrom<T> for HashRef<Hash> {
    type Err = GitError;
    fn try_from(t: T) -> Result<Self> {
        let r : &[u8] = t.as_ref();
        if r.len() != Hash::DIGEST_SIZE {
            Err(GitError::InvalidHashSize(Hash::DIGEST_SIZE, r.len()))
        } else {
            Ok(HashRef::new_with(t.as_ref()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hash;
    use hash::Hashable;

    struct MockHashable {
        data : String
    }
    impl MockHashable {
        fn new(d: &str) -> Self { MockHashable { data : d.to_string() } }
    }
    impl Hashable for MockHashable {
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
        let r : HashRef<hash::SHA1> = HashRef::new_with(data.hash::<hash::SHA1>());
        assert_eq!(r.capacity(), r.digest_size())
    }
    #[test]
    fn prefix_ok() {
        let data = MockHashable::new("The quick brown fox jumps over the lazy dog");
        let expected_digest = [47, 212, 225, 198, 122, 45, 40, 252, 237, 132, 158, 225, 187, 118, 231, 57, 27, 147, 235, 18];
        let expected_prefix = &expected_digest[..1];
        let expected_loose  = &expected_digest[1..];
        let r : HashRef<hash::SHA1> = HashRef::new_with(data.hash::<hash::SHA1>());
        assert_eq!(expected_prefix, r.prefix());
        assert_eq!(expected_loose,  r.loose());
        assert_eq!(expected_digest, r.as_ref())
    }
}
