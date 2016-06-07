use std::fmt;

extern crate crypto;
use self::crypto::digest::Digest;
use self::crypto::sha1::Sha1;

pub trait Property {
    const DIGEST_SIZE: usize;
    const PREFIX_SIZE: usize;

    fn new() -> Self;
    fn write<T: AsRef<[u8]>>(&mut self, bytes: T);
    fn finish(&mut self) -> Vec<u8>;
}
pub trait Hashable {
    fn get_chunk<'a>(&'a self, usize) -> &'a [u8];
    fn hash<Hash : Property> (&self) -> Vec<u8> {
        let mut hs = Hash::new();
        let mut i = 0;
        loop {
            let data = self.get_chunk(i);
            if (data.len() == 0) || (i >= 100) { break }
            i += 1;
            hs.write(data)
        }
        hs.finish()
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
