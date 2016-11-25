/*! Git date object

based on the [chrono](https://crates.io/crates/chrono) crate.

# Example

Creating a new date for *now*:

```
use git::object::date::Date;

let now = Date::now();
println!("Today is: {}", now);
```

Creating a custom date:

```
use git::object::date::Date;

let date = Date::ymd_hms(2013, 1, 29, 15, 34, 11)
            .expect("to have a valid date and time");
println!("First steps in UK: {}", date);
```

!*/

extern crate chrono;
use self::chrono::{DateTime, Local, FixedOffset, NaiveDateTime, TimeZone};
use protocol::decoder::*;
use protocol::encoder::*;
use std::{fmt, str, io};

use nom;

/// Git date object
///
/// It is always at the current `Local` time, without nanoseconds.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Date(DateTime<Local>);

unsafe impl Send for Date {}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { self.0.fmt(f) }
}

impl Date {
    /// create a new date set to know and the system local timezone
    pub fn now() -> Self { Date::new(Local::now()) }

    /// create a new date from the given `DateTime`
    ///
    /// This function will filter out the nano second precisions (if any).
    pub fn new(dt: DateTime<Local>) -> Self {
        let ndt = NaiveDateTime::from_timestamp(dt.timestamp(), 0);
        Date(DateTime::from_utc(ndt, dt.offset().clone()))
    }

    /// create a new date from EPOCH with the given local timezone
    fn from_epoch(dt: NaiveDateTime, fo: FixedOffset) -> Self {
        Date::new(DateTime::from_utc(dt, fo))
    }

    /// create custom time from seconds since epoch (using local timezone)
    pub fn seconds_since_epoch(seconds: i64) -> Self {
        Date(Local.timestamp(seconds,0))
    }

    /// Convenient function to make up date from human logic
    pub fn ymd_hms( year: i32
                  , month: u32
                  , day: u32
                  , hour: u32
                  , minutes: u32
                  , seconds: u32
                  ) -> Option<Self>
    {
        Local.ymd_opt(year, month, day)
            .and_hms_opt(hour, minutes, seconds)
            .earliest()
            .map(|dt| Date::new(dt))
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
                  if tz_sign { FixedOffset::east(h * 3600 + m * 60) }
                  else { FixedOffset::west(h * 3600 + m * 60) }
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
