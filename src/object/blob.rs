use protocol::hash::Hasher;
use protocol::encoder::Encoder;
use protocol::decoder::Decoder;
use std::io;
use nom;

/// Blob reference
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
pub struct BlobRef<H: Hasher>(H);
impl<H: Hasher> BlobRef<H> {
    pub fn new(h: H) -> Self { BlobRef(h) }
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

pub struct Blob;

// -- --------------------------------------------------------------------- --
// --                                 Tests                                 --
// -- --------------------------------------------------------------------- --

#[cfg(test)]
mod test {
    //! contract test. It's more to detect changes and make sure
    //! things don't break under our feet without knowing it.

    use super::*;
    use ::protocol::test_encoder_decoder;
    use ::protocol::hash::SHA1;

/*
    #[test]
    fn encode_decode_blob_ref() {
        let sha1_hex = "2aae6c35c94fcfb415dbe95f408b9ce91ee846ed";
        let sha1 = SHA1::from_str(sha1_hex)
                        .expect("expecting a valid SHA1 encoded in hexadecimal");
        let tr = BlobRef::new(sha1);
        test_encoder_decoder(tr);
    }
*/
}
