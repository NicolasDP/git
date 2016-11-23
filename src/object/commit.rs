use protocol::hash::Hasher;
use protocol::encoder::Encoder;
use protocol::decoder::Decoder;
use std::io;
use nom;

/// Tree reference in a given commit object
/*
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
pub struct TreeRef<H: Hasher>(H);
impl<H: Hasher> TreeRef<H> {
    pub fn new(h: H) -> Self { TreeRef(h) }
}
impl<H: Hasher> Encoder for TreeRef<H> {
    fn encode<W: io::Write>(&self, writer: &mut W) -> io::Result<usize> {
        let m = "tree ";
        try!(writer.write_all(m.as_bytes()));
        let h = try!(self.0.encode(writer));
        Ok(m.len() + h)
    }
}
impl<H: Hasher> Decoder for TreeRef<H> {
    fn decode(i: &[u8]) -> nom::IResult<&[u8], Self> {
        match nom_parse_tree_tag(i) {
            nom::IResult::Done(i, _) => {
                H::decode(i).map(|h| Self::new(h))
            },
            nom::IResult::Error(err) => nom::IResult::Error(err),
            nom::IResult::Incomplete(n) => nom::IResult::Incomplete(n)
        }
    }
}
named!( nom_parse_tree_tag, tag!("tree "));
*/

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
    fn encode_decode_tree_ref() {
        let sha1_hex = "2aae6c35c94fcfb415dbe95f408b9ce91ee846ed";
        let sha1 = SHA1::from_str(sha1_hex)
                        .expect("expecting a valid SHA1 encoded in hexadecimal");
        let tr = TreeRef::new(sha1);
        test_encoder_decoder(tr);
    }
    */
}
