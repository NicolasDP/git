/*! Compression modules

so far, we only want to re-export the `flat2`'s `ZlibEncoder` and `ZlibDecoder`
!*/

extern crate flate2;
pub use self::flate2::read::{ZlibDecoder, ZlibEncoder};
