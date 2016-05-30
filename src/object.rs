use hash;
use hash::{SHA1, HashRef};
use std::collections::BTreeMap;
use error::*;
use std;
use std::str;
use std::str::FromStr;

use nom;

/// Property for object that can be saved into a Git Object
///
/// # TODO
///
/// we could consider that an Objectable must be Hashable
/// or more simply that every Objectable is Hashable as soon
/// as it is possible to collect chunks of it.
///
pub trait Objectable<'a> : hash::Hashable + From<&'a str> {
    /// a file is a blob for example
    /// a stream is a blob as well
    /// the idea being we can create high level structure whith different
    /// properties or features but being a Git Object of type Blob... or Commit
    const OBJECT_TYPE: &'static str;
    fn get_chunk(&mut self, count: usize) -> Result<Vec<u8>>;
}

#[derive(Debug)]
pub struct Date {
    pub tz: i64,
    pub elapsed: i64,
}

#[derive(Debug)]
pub struct Person {
    pub name: String,
    pub email: String,
    pub date: Date
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

named!( parse_person<Person>
      , chain!( name:  map_res!(take_until_and_consume!(" <"), str::from_utf8)
              ~ email: map_res!(take_until_and_consume!("> "), str::from_utf8)
              ~ time: parse_digit_i64
              ~ tag!(" ")
              ~ tz_sign: parse_time_zone_sign
              ~ tz_fmt: parse_digit_i64
              , || {
                  let h = tz_fmt / 100;
                  let m = tz_fmt % 100;
                  Person { name: name.to_string()
                          , email: email.to_string()
                          , date: Date { tz: tz_sign * (h * 60 + m), elapsed: time}
                          }
              }
          )
      );

named!( parse_hashref_sha1<HashRef<SHA1> >
      , map_res!(map_res!( take!(<SHA1 as hash::Property>::DIGEST_SIZE * 2)
                         , str::from_utf8
                         )
                , FromStr::from_str
                )
      );

named!( parse_commit_parent< HashRef<SHA1> >
      , chain!( tag!("parent ")
              ~ r: parse_hashref_sha1
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
              ~ tag!("tree ") ~ tree_ref: parse_hashref_sha1 ~ char!('\n')
              ~ parents:  parse_commit_parents
              ~ tag!("author ")    ~ author: parse_person    ~ char!('\n')
              ~ tag!("committer ") ~ committer: parse_person ~ char!('\n')
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
    pub tree_ref: hash::HashRef<Hash>,
    pub parents: Vec<hash::HashRef<Hash>>,
    pub author: Person,
    pub committer: Person,
    pub encoding: Option<String>,
    pub extras: BTreeMap<String, String>,
    pub message: String
}

impl Commit<SHA1> {
    pub fn parse(buf: &[u8]) -> Result<Commit<SHA1>> {
        match parse_commit(buf) {
            nom::IResult::Done(_, c) => Ok(c),
            nom::IResult::Error(err) => Err(GitError::Unknown(format!("{:?}", err))),
            _                        => unimplemented!()
        }
    }
}

pub struct Blob {
  data: Vec<u8>,
}
impl Blob {
    fn new(d : &Vec<u8>) -> Self { Blob { data : d.clone() } }
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
    fn get_chunk(&self, count: usize) -> Option<Vec<u8>> {
        if count > 0 { return None }
        let mut v = format!("blob {}\0", self.data.len()).into_bytes();
        v.extend_from_slice(self.data.as_slice());
        Some(v)
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
