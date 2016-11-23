use std::io::{Write, Result};

pub trait Encoder {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<usize>;
}
