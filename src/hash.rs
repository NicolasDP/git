use std::{fmt, io};

extern crate crypto;
use self::crypto::digest::Digest;
use self::crypto::sha1::Sha1;

pub trait Property {
    fn hash<R: io::Read>(r: &mut R) -> io::Result<Vec<u8>>;
    fn digest_size() -> usize;
    fn prefix_size() -> usize;
}

pub struct SHA1;
impl fmt::Debug for SHA1 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SHA1")
    }
}
impl Property for SHA1 {
    fn digest_size() -> usize { 20 }
    fn prefix_size() -> usize { 1 }

    fn hash<R: io::Read>(r: &mut R) -> io::Result<Vec<u8>> {
        let mut st = Sha1::new();
        let mut buf : &mut [u8;128] = &mut [0u8;128];

        loop {
            let n = try!(r.read(buf));
            if n == 0 { break; }
            st.input(&buf[0..n]);
        }

        let mut out = vec![0u8;SHA1::digest_size()];
        st.result(out.as_mut_slice());
        Ok(out)
    }
}
