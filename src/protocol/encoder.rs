use std::io::{Write, Result};

pub trait Encoder {
    /// encode into the given `std::io::Write`.
    fn encode<W: Write>(&self, writer: &mut W) -> Result<usize>;

    /// when encoding into git objects, the size is needed to be known prior
    /// encoding. It is because the size is encoded prior the object.
    fn required_size(&self) -> usize;
}
