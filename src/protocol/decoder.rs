use nom;

pub trait Decoder: Sized {
    fn decode(i: &[u8]) -> nom::IResult<&[u8], Self>;
}
