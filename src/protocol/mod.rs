/*! git protocols and interfaces
!*/

pub mod hash;
pub mod compression;
pub mod decoder;
pub mod encoder;

#[cfg(test)]
use self::encoder::Encoder;
#[cfg(test)]
use self::decoder::Decoder;
#[cfg(test)]
use std::fmt::{Debug, Display};

#[cfg(test)]
pub fn test_encoder_decoder<T: Encoder+Decoder+Eq+Debug>(t: T) {
  use nom::IResult::*;
  let mut v = Vec::new();
  t.encode(&mut v).expect("encoding into buffer");
  let (v_, t_) = match T::decode(&mut v.as_slice()) {
      Done(i, o) => (i, o),
      Error(err) => {
          println!("decoding: {:?}", t);
          println!("buffer: {:?}", String::from_utf8(v.clone()).unwrap());
          Err(err).expect("decoding from buffer")
      },
      Incomplete(needed) => {
          println!("decoding: {:?}", t);
          println!("buffer: {:?}", String::from_utf8(v.clone()).unwrap());
          panic!(format!("not enough data: needed({:?})", needed));
      }
  };
  assert_eq!(t, t_);
  unsafe { println!("{:?}", String::from_utf8_unchecked(v_.iter().cloned().collect())) };
  assert!(v_.is_empty());
}


#[cfg(test)]
pub fn test_decode_encode<T: Encoder+Decoder+Eq+Debug+Display>(data: Vec<u8>) {
  use nom::IResult::*;
  let (v_, t) = match T::decode(&mut data.as_slice()) {
      Done(i, o) => (i, o),
      Error(err) => {
          Err(err).expect("decoding from buffer")
      },
      Incomplete(needed) => {
          panic!(format!("not enough data: needed({:?})", needed));
      }
  };
  println!("decoded:\n{}", t);
  // we expect to read all the buffer
  assert!(v_.is_empty());
  let mut v = Vec::new();
  t.encode(&mut v).expect("encoding into buffer");
  assert_eq!(data, v);
}
