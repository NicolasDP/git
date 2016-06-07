use nom;
use std::{fmt, str};

use ::objectable::{Readable, Writable};

/// Git Date
///
/// * timezone
/// * elapsed (since EPOCH)
///
/// # Example
///
/// ```
/// use git::object::{Date};
///
/// let date = Date::new(1464729412, 60);
/// println!("{}", date);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Date {
    tz: i64,
    elapsed: i64,
}
impl Date {
    pub fn new(tz: i64, elapsed: i64) -> Self { Date { tz: tz, elapsed: elapsed } }
}

named!(parse_time_zone_sign_plus <i64>, chain!(tag!("+"), || { 1 }));
named!(parse_time_zone_sign_minus<i64>, chain!(tag!("-"), || { -1 }));
named!(parse_time_zone_sign_def  <i64>, value!(1) );
named!( parse_time_zone_sign     <i64>
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
named!( nom_parse_date<Date>
      , chain!( time: parse_digit_i64
              ~ tag!(" ")
              ~ tz_sign: parse_time_zone_sign
              ~ tz_fmt: parse_digit_i64
              , || {
                  let h = tz_fmt / 100;
                  let m = tz_fmt % 100;
                  Date::new(tz_sign * (h * 60 + m), time)
              })
      );

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}s {}", self.elapsed, self.tz)
    }
}
impl Writable for Date {
    fn serialise(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (c, r) = if self.tz < 0 { ('-', - self.tz) } else { ('+', self.tz) };
        let h = r / 60;
        let m = r % 60;
        let pad = if h < 10 { "0" } else { "" };
        write!(f, "{} {}{}{}", self.elapsed, c, pad, h * 100 + m)
    }
    fn provide_size(&self) -> usize { format!("{}", self).len() }
}
impl Readable for Date {
    fn nom_parse(b: & [u8]) -> nom::IResult<&[u8], Self> { nom_parse_date(b) }
}
