/*! Git date object

based on the [chrono](https://crates.io/crates/chrono) crate.
!*/

extern crate chrono;
use self::chrono::{DateTime, Local, FixedOffset, NaiveDateTime};
use protocol::decoder::*;
use protocol::encoder::*;
use std::{fmt, str, io};

use nom;

/// Git date object
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Date(DateTime<Local>);

unsafe impl Send for Date {}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { self.0.fmt(f) }
}

impl Date {
    /// create a new date set to know and the system local timezone
    ///
    /// This is not a nano second precision...
    pub fn now() -> Self {
        let loc = Local::now();
        let dt = NaiveDateTime::from_timestamp(loc.timestamp(), 0);
        Date(DateTime::from_utc(dt, loc.offset().clone()))
    }

    /// create a new date from the given `DateTime`
    pub fn new(dt: DateTime<Local>) -> Self { Date(dt) }

    /// create a new date from EPOCH with the given local timezone
    fn from_epoch(dt: NaiveDateTime, fo: FixedOffset) -> Self {
        Date(
            DateTime::from_utc(dt, fo)
        )
    }

    /// encode in a object
    pub fn encode_for_obj(&self) -> String { self.0.format("%s %z").to_string() }
}

impl Decoder for Date {
    fn decode(b: &[u8]) -> nom::IResult<&[u8], Self> { nom_parse_date(b) }
}
impl Encoder for Date {
    fn required_size(&self) -> usize { self.encode_for_obj().len() }
    fn encode<W: io::Write>(&self, writer: &mut W) -> io::Result<usize> {
        let hex = self.encode_for_obj();
        try!(writer.write_all(hex.as_bytes()));
        Ok(hex.len())
    }
}

// -- --------------------------------------------------------------------- --
// --                               Nom Parser                              --
// -- --------------------------------------------------------------------- --

named!(parse_time_zone_sign_plus <bool>, chain!(tag!("+"), || { true }));
named!(parse_time_zone_sign_minus<bool>, chain!(tag!("-"), || { false }));
named!(parse_time_zone_sign_def  <bool>, value!(true) );
named!( parse_time_zone_sign     <bool>
      , alt!( parse_time_zone_sign_plus
            | parse_time_zone_sign_minus
            | parse_time_zone_sign_def
            )
      );
named!( parse_digit_i64<i64>
      , map_res!( map_res!( nom::digit
                          , str::from_utf8
                          )
                , str::FromStr::from_str
                )
      );
named!( parse_digit_i32<i32>
      , map_res!( map_res!( nom::digit
                          , str::from_utf8
                          )
                , str::FromStr::from_str
                )
      );
named!( nom_parse_timezone<FixedOffset>
      , chain!( tz_sign: parse_time_zone_sign
              ~ tz_fmt: parse_digit_i32
              , || {
                  let h = tz_fmt / 100;
                  let m = tz_fmt % 100;
                  if tz_sign { FixedOffset::west(h * 60 + m) }
                  else { FixedOffset::east(h * 60 + m) }
              })
      );
named!( nom_parse_date_time<NaiveDateTime>
      , chain!( time: parse_digit_i64
              , || {
                  NaiveDateTime::from_timestamp(time, 0)
              })
      );
named!( nom_parse_date<&[u8], Date>
      , chain!( time: nom_parse_date_time
              ~ tag!(" ")
              ~ tz: nom_parse_timezone
              , || {
                  Date::from_epoch(time, tz)
              })
      );

// -- --------------------------------------------------------------------- --
// --                                 Tests                                 --
// -- --------------------------------------------------------------------- --

#[cfg(test)]
mod test {
    use super::*;
    use ::protocol::test_encoder_decoder;

    #[test]
    fn date_serialisable() {
        let date = Date::now();
        test_encoder_decoder(date);
    }
}
