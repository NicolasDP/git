use hash;
use hash::{SHA1, HashRef};
use std::collections::BTreeMap;
use error::*;
use std;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::str;

use nom;

pub use objectable::Objectable;

/// Git Date
///
/// * timezone
/// * elapsed (since EPOCH)
///
/// # Example
///
/// ```
/// use git::object::{Date, Objectable};
///
/// let date = Date::new(1464729412, 60);
/// let str = format!("{}", date);
/// let date2 = Date::parse_bytes(str.as_bytes()).unwrap();
/// assert_eq!(date, date2);
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
                          , std::str::from_utf8
                          )
                , std::str::FromStr::from_str
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

impl Display for Date {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let (s, r) = if self.tz < 0 { (-1, - self.tz) } else { (1, self.tz) };
        let h = r / 60;
        let m = r % 60;
        write!(f, "{} {}", self.elapsed, s * (h * 100 + m))
    }
}
impl Objectable for Date {
    fn nom_parse(b: & [u8]) -> nom::IResult<&[u8], Self> { nom_parse_date(b) }
}

/// Git Person
///
/// * a name
/// * an email address
/// * a git::Date
///
/// # Example
///
/// ```
/// use git::object::{Date, Person, Objectable};
///
/// let date = Date::new(1464729412, 60);
/// let person = Person::new_str("Kevin Flynn", "kev@flynn.rs", date);
/// let str = format!("{}", person);
/// let person2 = Person::parse_bytes(str.as_bytes()).unwrap();
/// assert_eq!(person, person2);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Person {
    pub name: String,
    pub email: String,
    pub date: Date
}
impl Person {
    pub fn new(name: String, email: String, date: Date) -> Self {
        Person {
            name: name,
            email: email,
            date: date
        }
    }
    pub fn new_str(name: &str, email: &str, date: Date) -> Self {
        Person {
            name: name.to_string(),
            email: email.to_string(),
            date: date
        }
    }
}

named!( nom_parse_person<Person>
      , chain!( name:  map_res!(take_until_and_consume!(" <"), str::from_utf8)
              ~ email: map_res!(take_until_and_consume!("> "), str::from_utf8)
              ~ date:  call!(Date::nom_parse)
              , || Person::new_str(name, email, date)
              )
      );

impl Display for Person {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{} <{}> {}", self.name, self.email, self.date)
    }
}
impl Objectable for Person {
    fn nom_parse(b: & [u8]) -> nom::IResult<&[u8], Self> { nom_parse_person(b) }
}

/// contains the HashRef to a git tree
///
/// # Example
///
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TreeRef<Hash: hash::Property+hash::Hasher> {
    tree_ref: hash::HashRef<Hash>
}
impl<Hash: hash::Property+hash::Hasher> TreeRef<Hash> {
    pub fn new(tr: hash::HashRef<Hash>) -> Self { TreeRef { tree_ref: tr } }
}
impl<Hash: hash::Property+hash::Hasher> Display for TreeRef<Hash> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "tree {}", self.tree_ref)
    }
}
named!( nom_parse_tree_ref<TreeRef<SHA1> >
      , chain!( tag!("tree ") ~ r: call!(Objectable::nom_parse)
              , || TreeRef::new(r)
              )
      );
impl Objectable for TreeRef<SHA1> {
    fn nom_parse(b: &[u8]) -> nom::IResult<&[u8], Self> { nom_parse_tree_ref(b) }
}

named!( parse_commit_parent< HashRef<SHA1> >
      , chain!( tag!("parent ")
              ~ r: call!(HashRef::nom_parse)
              ~ char!('\n')
              , || { return r }
              )
      );
named!( parse_commit_parents< Vec<HashRef<SHA1> > >
      , many0!(parse_commit_parent)
      );

named!( parse_encoding<String>
      , chain!( tag!("encoding ")
              ~ e: map_res!(take_until!("\n"), str::from_utf8)
              , || e.to_string()
              )
      );
fn not_eol(c: u8) -> bool { c != 0xa }
named! ( parse_string_eol<&str>
       , map_res!(take_while1!(not_eol), str::from_utf8)
       );
named!( parse_extra<(String, String)>
      , chain!( k: parse_string_eol ~ char!('\n')
              ~ v: parse_string_eol ~ char!('\n')
              , || (k.to_string(), v.to_string())
              )
      );

named!( parse_extras<BTreeMap<String, String> >
      , chain!( mut acc: value!(BTreeMap::new())
              ~ many0!( tap!(v: parse_extra => acc.insert(v.0.clone(), v.1.clone())))
              , || { return acc }
              )
      );

named!( parse_commit<Commit<SHA1> >
      , chain!( tag!("commit ") ~ many1!(nom::digit) ~ char!('\0')
              ~ tree_ref: call!(TreeRef::nom_parse) ~ char!('\n')
              ~ parents:  parse_commit_parents
              ~ tag!("author ")    ~ author: call!(Person::nom_parse) ~ char!('\n')
              ~ tag!("committer ") ~ committer: call!(Person::nom_parse) ~ char!('\n')
              ~ encoding: opt!(parse_encoding)
              ~ extras: parse_extras
              ~ char!('\n')
              ~ message: map_res!(nom::non_empty, str::from_utf8)
              , || Commit{
                  tree_ref: tree_ref,
                  parents: parents,
                  author: author,
                  committer: committer,
                  encoding: encoding,
                  extras: extras,
                  message: message.to_string()
              }
              )
      );

#[derive(Debug)]
pub struct Commit<Hash: hash::Property+hash::Hasher> {
    pub tree_ref: TreeRef<Hash>,
    pub parents: Vec<hash::HashRef<Hash>>,
    pub author: Person,
    pub committer: Person,
    pub encoding: Option<String>,
    pub extras: BTreeMap<String, String>,
    pub message: String
}

/*
impl Objectable for Commit<SHA1> {
    fn nom_parse(b: &[u8]) -> nom::IResult<&[u8], Self> {
        parse_commit(b)
    }
}
impl<Hash: hash::Property+hash::Hasher> Display for Commit<Hash> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!( f
              , "commit {}\0"
              , self.digest().to_hex()
              )
    }
}
*/
impl Commit<SHA1> {
    pub fn parse(buf: &[u8]) -> Result<Commit<SHA1>> {
        match parse_commit(buf) {
            nom::IResult::Done(_, c)    => Ok(c),
            nom::IResult::Error(err)    => Err(GitError::ParsingErrorUnknown(format!("{:?}", err))),
            nom::IResult::Incomplete(n) => Err(GitError::ParsingErrorUnknown(format!("{:?}", n))),
        }
    }
}

pub struct Blob {
  data: Vec<u8>,
  size: Vec<u8>
}
impl Blob {
    fn new(d : &Vec<u8>) -> Self {
        Blob {
            data : d.clone(),
            size : format!("{}", d.len()).into_bytes()
        }
    }
}
impl<'a> From<&'a[u8]> for Blob {
    fn from(d: &'a[u8]) -> Self {
        let mut v = Vec::new();
        v.extend_from_slice(d);
        Blob::new(&v)
    }
}
impl<'a> From<&'a str> for Blob {
    fn from(d: &'a str) -> Self { From::from(d.as_bytes()) }
}

impl hash::Hashable for Blob {
    fn get_chunk(&self, count: usize) -> &[u8] {
        match count {
            0 => &b"blob "[..],
            1 => self.size.as_ref(),
            2 => &b"\0"[..],
            3 => self.data.as_ref(),
            _ => &b""[..]
        }
    }
}

#[cfg(test)]
mod tests {
    use hash;
    use hash::Hashable;
    use object;

    #[test]
    fn test_blob() {
        let data : object::Blob = From::from("The quick brown fox jumps over the lazy cog");
        let expected_digest = [18, 224, 96, 142, 217, 247, 183, 20, 57, 121, 97, 167, 8, 7, 75, 151, 22, 166, 74, 33];
        let expected_prefix = &expected_digest[..1];
        let expected_loose  = &expected_digest[1..];
        let r : hash::HashRef<hash::SHA1> = data.hash();
        assert_eq!(expected_prefix, r.prefix());
        assert_eq!(expected_loose,  r.loose());
        assert_eq!(expected_digest, r.digest())
    }
}

#[cfg(test)]
mod bench {
    use hash;
    use hash::Hashable;
    use object;
    use test::Bencher;

    #[bench]
    pub fn hash_(bh: & mut Bencher) {
        let v : Vec<u8> = vec![0; 65536];
        let bytes : object::Blob = From::from(&v as &[u8]);
        bh.iter( || {
            let _ : hash::HashRef<hash::SHA1> = bytes.hash();
        });
        bh.bytes = bytes.data.len() as u64;
    }
}
