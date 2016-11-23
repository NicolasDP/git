use nom;
use std::{fmt, str};

use ::objectable::{Readable, Writable};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Elapsed(pub i64);
impl fmt::Display for Elapsed {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.serialise(f)
    }
}
impl Writable for Elapsed {
    fn serialise(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
    fn provide_size(&self) -> usize { format!("{}", self).len() }
}
impl Readable for Elapsed {
    fn nom_parse(b: & [u8]) -> nom::IResult<&[u8], Self> { nom_parse_elapsed(b) }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Timezone(pub i64);
impl fmt::Display for Timezone {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.serialise(f)
    }
}
impl Writable for Timezone {
    fn serialise(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (c, r) = if self.0 < 0 { ('-', - self.0) } else { ('+', self.0) };
        let h = r / 60;
        let m = r % 60;
        let pad = if h < 10 { "0" } else { "" };
        write!(f, "{}{}{}", c, pad, h * 100 + m)
    }
    fn provide_size(&self) -> usize { format!("{}", self).len() }
}
impl Readable for Timezone {
    fn nom_parse(b: & [u8]) -> nom::IResult<&[u8], Self> { nom_parse_timezone(b) }
}

/// Git Date
///
/// * timezone
/// * elapsed (since EPOCH)
///
/// # Example
///
/// ```
/// use git::object::elements::date::*;
///
/// let date = Date::new(Elapsed(1464729412), Timezone(60));
/// println!("{}", date);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Date
{ elapsed: Elapsed
, tz     : Timezone
}

impl Date {
    /// create a new Date from the given timezone and the given timestamp
    pub fn new( elapsed: Elapsed
              , tz     : Timezone
              )
        -> Self
    {
      Date
        { elapsed: elapsed
        , tz     : tz
        }
    }
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.serialise(f)
    }
}
impl Writable for Date {
    fn serialise(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", self.elapsed, self.tz)
    }
    fn provide_size(&self) -> usize { format!("{}", self).len() }
}
impl Readable for Date {
    fn nom_parse(b: & [u8]) -> nom::IResult<&[u8], Self> { nom_parse_date(b) }
}

// ---------------------------------------------------------------------------
// --                       NOM parsing functions                           --
// ---------------------------------------------------------------------------

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
named!( nom_parse_elapsed<Elapsed>
      , chain!(time: parse_digit_i64, || Elapsed(time))
      );
named!( nom_parse_timezone<Timezone>
      , chain!( tz_sign: parse_time_zone_sign
              ~ tz_fmt: parse_digit_i64
              , || {
                  let h = tz_fmt / 100;
                  let m = tz_fmt % 100;
                  Timezone(tz_sign * (h * 60 + m))
              })
      );
named!( nom_parse_date<Date>
      , chain!( time: call!(Elapsed::nom_parse)
              ~ tag!(" ")
              ~ tz: call!(Timezone::nom_parse)
              , || {
                  Date::new(time, tz)
              })
      );

