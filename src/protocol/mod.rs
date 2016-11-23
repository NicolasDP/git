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
use std::fmt::Debug;

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
      Incomplete(needed) => panic!()
  };
  assert_eq!(t, t_);
  assert!(v_.is_empty());
}
