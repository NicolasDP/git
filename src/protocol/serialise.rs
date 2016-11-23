use std::io::{BufRead, Write, Result};
use std::io;
use nom;

#[derive(Debug)]
pub enum Error {
    IoError(io::Error),
    NomError(nom::Err<u32>)
}

pub enum Decoder<B: BufRead, I> {
    Ok(B, I),
    Invalid(B, Error)
}

impl<B: BufRead, I> Decoder<B, I> {
    pub fn unwrap(self) -> I {
        use self::Decoder::*;
        match self {
            Ok(_, v) => v,
            Invalid(_, err) => panic!("{:?}", err)
        }
    }

    pub fn expect(self, expect: &str) -> I {
        use self::Decoder::*;
        match self {
            Ok(_, v) => v,
            Invalid(_, err) => panic!("{} {:?}", expect, err)
        }
    }

    pub fn or<A>(self, alt: A) -> Decoder<B, I>
      where A: FnOnce(B) -> Decoder<B, I>
    {
        use self::Decoder::*;
        match self {
            Ok(b, v) => Ok(b, v),
            Invalid(b, _) => alt(b)
        }
    }
}

/// Serialisation protocol
///
/// interfaces to generically encore element into bytes
pub trait Serialisable: Sized {
    fn serialise<W: Write>(&self, writer: W) -> Result<W>;
    fn deserialise<R: BufRead>(reader: R) -> Decoder<R, Self>;
}

impl Serialisable for Vec<u8> {
    fn serialise<W: Write>(&self, w: W) -> Result<W> {
        let mut writer = w;
        try!(writer.write_all(&mut self.as_slice()));
        Ok(writer)
    }
    fn deserialise<R: BufRead>(r: R) -> Decoder<R, Self> {
        let mut reader = r;
        let mut buf = Vec::new();
        match reader.read_to_end(&mut buf) {
            Ok(_) => (),
            Err(err) => return Decoder::Invalid(reader, Error::IoError(err))
        }
        Decoder::Ok(reader, buf)
    }
}

impl Serialisable for String {
    fn serialise<W: Write>(&self, w: W) -> Result<W> {
        let mut writer = w;
        try!(writer.write_all(&mut self.as_bytes()));
        Ok(writer)
    }
    fn deserialise<R: BufRead>(r: R) -> Decoder<R, Self> {
        let mut reader = r;
        let mut buf = String::new();
        match reader.read_to_string(&mut buf) {
            Ok(_) => (),
            Err(err) => return Decoder::Invalid(reader, Error::IoError(err))
        }
        Decoder::Ok(reader, buf)
    }
}

pub mod read {
    /*! helper to read from a given stream
    !*/
    use std::io::{Read, Result};

    pub fn string<R: Read>(r: R, m: &str) -> Result<R> {
        let mut match_ = String::new();
        let mut taken = r.take(m.len() as u64);
        try!(taken.read_to_string(&mut match_));
        if match_ == m { Ok(taken.into_inner())}
        else { println!("{:?} != {:?}", m, match_); ; unimplemented!() }
    }

    #[inline]
    pub fn u8<R: Read>(r: &mut R) -> Result<u8> {
        let mut buf = [0u8;1];
        try!(r.read_exact(&mut buf));
        Ok(buf[0])
    }
    #[inline]
    pub fn u16<R: Read>(r: &mut R) -> Result<u16> {
        let v1 = try!(self::u8(r)) as u16;
        let v2 = try!(self::u8(r)) as u16;
        Ok(v1 << 8 | v2)
    }
    #[inline]
    pub fn u32<R: Read>(r: &mut R) -> Result<u32> {
        let v1 = try!(self::u16(r)) as u32;
        let v2 = try!(self::u16(r)) as u32;
        Ok(v1 << 16 | v2)
    }
    #[inline]
    pub fn u64<R: Read>(r: &mut R) -> Result<u64> {
        let v1 = try!(self::u32(r)) as u64;
        let v2 = try!(self::u32(r)) as u64;
        Ok(v1 << 32 | v2)
    }
    #[inline]
    pub fn i8<R: Read>(r: &mut R) -> Result<i8> {
        self::u8(r).map(|v| v as i8)
    }
    #[inline]
    pub fn i16<R: Read>(r: &mut R) -> Result<i16> {
        self::u16(r).map(|v| v as i16)
    }
    #[inline]
    pub fn i32<R: Read>(r: &mut R) -> Result<i32> {
        self::u32(r).map(|v| v as i32)
    }
    #[inline]
    pub fn i64<R: Read>(r: &mut R) -> Result<i64> {
        self::u64(r).map(|v| v as i64)
    }


    #[cfg(test)]
    mod test {
        use std::io::BufRead;
        use super::string;

        #[test]
        fn read_string() {
            let buf : Vec<u8> = "test".to_string().into_bytes();
            let buf = string(buf.as_slice(), "test").unwrap();
            assert_eq!(buf.len(), 0);
        }

        #[test]
        fn read_strings() {
            let buf : Vec<u8> = "hello world!\n".to_string().into_bytes();
            let mut buf = string(buf.as_slice(), "hello").unwrap();
            buf.consume(1);
            let buf = string(buf, "world").unwrap();
            let buf = string(buf, "!").unwrap();
            assert_eq!(buf.len(), 1);
            assert_eq!(buf[0], 0x0a);
        }
    }
}


/// method to test a serialisable property
///
/// This method will be available only when testing so we can re-use it
/// in some places here and there.
#[cfg(test)]
pub fn serialisable_property<S: Serialisable + ::std::fmt::Debug + Eq>(data: S) -> S {
    let buf : Vec<u8> = Vec::new();
    let buf = data.serialise(buf).unwrap();
    let data_ = S::deserialise(buf.as_slice()).unwrap();
    assert_eq!(data, data_);
    data_
}


#[cfg(test)]
mod test {
    //! contract test. It's more to detect changes and make sure
    //! things don't break under our feet without knowing it.

    use super::*;
    use std;

    #[test]
    fn string_empty() {
        let data = String::new();
        let data_ = serialisable_property(data);
        assert!(data_.is_empty());
    }
    #[test]
    fn string_1() {
        let data = "1".to_string();
        serialisable_property(data);
    }
    #[test]
    fn string_2() {
        let data = "some longer string".to_string();
        serialisable_property(data);
    }
    #[test]
    fn string_3() {
        let data = std::iter::repeat("X").take(4096).collect::<String>();
        serialisable_property(data);
    }

    #[test]
    fn vec_empty() {
        let data = Vec::new();
        let data_ = serialisable_property(data);
        assert!(data_.is_empty());
    }
    #[test]
    fn vec_1() {
        let data = vec![0x20u8];
        serialisable_property(data);
    }
    #[test]
    fn vec_2() {
        let data : Vec<u8> = (0x41u8..0x50u8).collect();
        serialisable_property(data);
    }
    #[test]
    fn vec_3() {
        let data : Vec<u8> = std::iter::repeat(0x41u8).take(4096).collect();
        serialisable_property(data);
    }
}
