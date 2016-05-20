use std::marker::PhantomData;
use std::convert::From;

extern crate crypto;
use self::crypto::digest::Digest;
use self::crypto::sha1::Sha1;

pub trait Hasher {
    fn new() -> Self;
    fn write(&mut self, bytes: &[u8]);
    fn finish(&mut self) -> Vec<u8>;
}
pub trait Property {
    const DIGEST_SIZE: usize;
    const PREFIX_SIZE: usize;
}

pub struct SHA1 {
  state: crypto::sha1::Sha1,
}
impl Hasher for SHA1 {
    fn new() -> Self { SHA1 { state: Sha1::new() } }
    fn write(&mut self, bytes: &[u8]) {
        self.state.input(bytes);
    }
    fn finish(&mut self) -> Vec<u8> {
        let mut out = vec![0u8;20];
        self.state.result(out.as_mut_slice());
        out
    }
}

pub trait Hashable {
    fn get_chunk(&self, usize) -> Option<Vec<u8>>;
    fn hash<Hash : Property + Hasher> (&self) -> Ref<Hash> {
        let mut hs = Hash::new();
        let mut i = 0;
        while let Some(data) = self.get_chunk(i) {
            hs.write(&data);
            if i == 100 { break; } else { i = i + 1; }
        };
        Ref::new_with(&hs.finish())
    }
}

impl Property for SHA1 {
    const DIGEST_SIZE: usize = 20;
    const PREFIX_SIZE: usize = 1;
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Ref<Hash : Property> {
    hash: Vec<u8>,
    _hash_type: PhantomData<Hash>,
}

impl<Hash : Property + Hasher> Ref<Hash> {
    pub fn new() -> Self {
        Ref
            { hash       : Vec::with_capacity(Hash::DIGEST_SIZE)
            , _hash_type : PhantomData
            }
    }
    pub fn new_with(data: &[u8]) -> Self {
        let mut v = Vec::with_capacity(20);
        v.extend_from_slice(data);
        Ref
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
}

impl<'a> From<&'a Vec<u8>> for Ref<SHA1> {
    fn from(data:&'a Vec<u8>) -> Self { Ref::new_with(data) }
}
impl<'a> From<&'a [u8;20]> for Ref<SHA1> {
    fn from(data:&'a [u8;20]) -> Self {
        let mut r = Ref::new();
        r.hash.extend_from_slice(data);
        r
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ::git::hash;

    struct MockHashable {
        data : String
    }
    impl MockHashable {
        fn new(d: &str) -> Self { MockHashable { data : d.to_string() } }
    }
    impl hash::Hashable for MockHashable {
        fn get_chunk(&self, count: usize) -> Option<Vec<u8>> {
            if count > 0 { return None }
            let mut v = Vec::with_capacity(self.data.len());
            v.extend_from_slice(self.data.as_bytes());
            Some(v.clone())
        }
    }

    #[test]
    fn capacity_equals_hash_size() {
        let r : Ref<hash::SHA1> = Ref::new();
        assert_eq!(20, r.digest_size());
        assert_eq!(1,  r.prefix_size());
        assert!(r.prefix_size() <= r.digest_size())
    }

    #[test]
    fn set_hash_ok() {
        let data = MockHashable::new("The quick brown fox jumps over the lazy cog");
        let r : Ref<hash::SHA1> = data.hash();
        assert_eq!(r.capacity(), r.digest_size())
    }
    #[test]
    fn prefix_ok() {
        let data = MockHashable::new("The quick brown fox jumps over the lazy dog");
        let expected_digest = [47, 212, 225, 198, 122, 45, 40, 252, 237, 132, 158, 225, 187, 118, 231, 57, 27, 147, 235, 18];
        let expected_prefix = &expected_digest[..1];
        let expected_loose  = &expected_digest[1..];
        let r : Ref<hash::SHA1> = data.hash();
        assert_eq!(expected_prefix, r.prefix());
        assert_eq!(expected_loose,  r.loose());
        assert_eq!(expected_digest, r.digest())
    }
}
