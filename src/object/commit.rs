use super::tree::TreeRef;
use super::person::Person;
use protocol::{Encoder, Decoder, Hash};
use std::{io, fmt, convert, ops, iter, slice, collections, str};
use nom;
use error::Result;

/// Commit reference
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
pub struct CommitRef<H: Hash>(H);
impl<H: Hash> CommitRef<H> {
    pub fn new(h: H) -> Self { CommitRef(h) }
}
impl<H: Hash + fmt::Display> fmt::Display for CommitRef<H> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { fmt::Display::fmt(&self.0, f) }
}

impl<H: Hash> Hash for CommitRef<H> {
    fn hash<R: io::BufRead>(data: &mut R) -> Result<Self> {
        H::hash(data).map(|h| CommitRef(h))
    }

    fn from_bytes(v: Vec<u8>) -> Option<Self> {
        H::from_bytes(v).map(|h| CommitRef(h))
    }

    #[inline]
    fn digest_size() -> usize { H::digest_size() }

    #[inline]
    fn as_bytes(&self) -> &[u8] { self.0.as_bytes() }
}
impl<H: Hash> convert::AsRef<H> for CommitRef<H> {
    fn as_ref(&self) -> &H { &self.0 }
}

/// collection of commit parents
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Parents<H: Hash>(Vec<CommitRef<H>>);
impl<H: Hash> Parents<H> {
    pub fn new() -> Self { Self::new_with(Vec::new()) }
    pub fn new_with(v: Vec<CommitRef<H>>) -> Self { Parents(v) }
    pub fn push(&mut self, p: CommitRef<H>) { self.0.push(p) }
}
impl<H: Hash> ops::Deref for Parents<H> {
    type Target = [CommitRef<H>];
    fn deref(&self) -> &Self::Target { self.0.deref() }
}
impl<H: Hash> ops::DerefMut for Parents<H> {
    fn deref_mut(&mut self) -> &mut Self::Target { self.0.deref_mut() }
}
impl<H: Hash> iter::FromIterator<CommitRef<H>> for Parents<H> {
    fn from_iter<I: IntoIterator<Item=CommitRef<H>>>(iter: I) -> Self {
        Self::new_with(iter.into_iter().collect())
    }
}
impl<H: Hash> iter::IntoIterator for Parents<H> {
    type Item = CommitRef<H>;
    type IntoIter = ::std::vec::IntoIter<CommitRef<H>>;
    fn into_iter(self) -> Self::IntoIter { self.0.into_iter() }
}
impl<'a, H: Hash> iter::IntoIterator for &'a Parents<H> {
    type Item = &'a CommitRef<H>;
    type IntoIter = slice::Iter<'a, CommitRef<H>>;
    fn into_iter(self) -> Self::IntoIter { self.0.iter() }
}
impl<'a, H: Hash> IntoIterator for &'a mut Parents<H> {
    type Item = &'a mut CommitRef<H>;
    type IntoIter = slice::IterMut<'a, CommitRef<H>>;
    fn into_iter(mut self) -> Self::IntoIter { self.0.iter_mut() }
}
impl<H: Hash> Encoder for Parents<H> {
    fn required_size(&self) -> usize {
        let mut sz = 7;
        for _ in self.iter() {
            sz += H::digest_hex_size() + 1;
        }
        sz
    }
    fn encode<W: io::Write>(&self, writer: &mut W) -> io::Result<usize> {
        let mut sz = 0;
        for p in self.iter() {
            sz += try!(encode_parent(p, writer));
        }
        Ok(sz)
    }
}
impl<H: Hash> Decoder for Parents<H> {
    fn decode(b: &[u8]) -> nom::IResult<&[u8], Self> {
        let mut i = b;
        let mut parents = Self::new();
        while let nom::IResult::Done(b, parent) = nom_parse_parent::<H>(i) {
            i = b;
            parents.push(parent);
        };
        nom::IResult::Done(i, parents)
    }
}
named!(nom_parse_newline, tag!("\n"));
named!(nom_parse_parent_tag, tag!("parent "));
fn nom_parse_parent<H: Hash>(b: &[u8]) -> nom::IResult<&[u8], CommitRef<H>> {
    let (b, _) = try_parse!(b, nom_parse_parent_tag);
    let (b, cr) = try_parse!(b, CommitRef::<H>::decode_hex);
    let (b, _) = try_parse!(b, nom_parse_newline);
    nom::IResult::Done(b, cr)
}
fn encode_parent<H: Hash, W: io::Write>(commit: &CommitRef<H>, writer: &mut W) -> io::Result<usize> {
    try!(writer.write_all(b"parent "));
    let s = try!(commit.encode_hex(writer));
    try!(writer.write_all(b"\n"));
    Ok(s + 8)
}

/// Git commit message encoding information
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Encoding {
    raw : String
}
impl Encoding {
    pub fn new(raw: String) -> Self { Encoding { raw: raw } }
    pub fn new_str<'a>(raw: &'a str) -> Self { Encoding { raw: raw.to_string() } }
}
impl Encoder for Encoding {
    fn required_size(&self) -> usize { self.raw.len() + 9 }
    fn encode<W: io::Write>(&self, writer: &mut W) -> io::Result<usize> {
        let data = format!( "{}", self);
        try!(writer.write_all(data.as_bytes()));
        Ok(data.len())
    }
}
impl fmt::Display for Encoding {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "encoding {}", self.raw)
    }
}
named!( nom_parse_encoding<Encoding>
      , chain!( tag!("encoding ")
              ~ e: map_res!(take_while1!(is_valid_encoding_char), str::from_utf8)
              , || Encoding::new_str(e)
              )
      );

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Extras(collections::BTreeMap<String, String>);
impl Extras {
    pub fn new() -> Self { Extras::new_with(collections::BTreeMap::new()) }
    fn new_with(btm: collections::BTreeMap<String, String>) -> Self {
        Extras(btm)
    }
}
impl IntoIterator for Extras {
    type Item = (String, String);
    type IntoIter = ::std::collections::btree_map::IntoIter<String, String>;
    fn into_iter(self) -> ::std::collections::btree_map::IntoIter<String, String> {
        self.0.into_iter()
    }
}
impl<'a> IntoIterator for &'a Extras {
    type Item = (&'a String, &'a String);
    type IntoIter = ::std::collections::btree_map::Iter<'a, String, String>;
    fn into_iter(self) -> ::std::collections::btree_map::Iter<'a, String, String> {
        self.0.iter()
    }
}
impl<'a> IntoIterator for &'a mut Extras {
    type Item = (&'a String, &'a mut String);
    type IntoIter = ::std::collections::btree_map::IterMut<'a, String, String>;
    fn into_iter(self) -> ::std::collections::btree_map::IterMut<'a, String, String> {
        self.0.iter_mut()
    }
}
impl iter::FromIterator<(String, String)> for Extras {
    fn from_iter<T: IntoIterator<Item=(String, String)>>(iter: T) -> Extras {
        Extras::new_with(collections::BTreeMap::from_iter(iter))
    }
}
impl Extend<(String, String)> for Extras {
    fn extend<T: IntoIterator<Item=(String, String)>>(&mut self, iter: T) {
        self.0.extend(iter)
    }
}

#[inline(always)]
fn is_valid_encoding_char(c: u8) -> bool {
    nom::is_alphanumeric(c)
        || c == 0x20 // white spaces
        || c == 0x09 // tabulation
        || c == 0x2d // '-' hyphen
        || c == 0x5f // '_' underscore
}
#[inline(always)]
named! ( parse_string_eol<&str>
       , map_res!(take_while1!(is_valid_encoding_char), str::from_utf8)
       );
named!( parse_extra<(String, String)>
      , chain!( k: parse_string_eol ~ char!('\n')
              ~ mut acc: value!(String::new())
              ~ many0!(chain!( char!(' ') ~ v: parse_string_eol ~ char!('\n')
                             , || { acc.push_str(v); acc.push('\n')} ))
              , || (k.to_string(), acc)
              )
      );
named!( nom_parse_extras<Extras>
      , chain!( mut acc: value!(collections::BTreeMap::new())
              ~ many0!( tap!(v: parse_extra => acc.insert(v.0.clone(), v.1.clone())))
              , || Extras::new_with(acc)
              )
      );
impl Encoder for Extras {
    fn required_size(&self) -> usize {
        let mut sum : usize = 0;
        for (key, value) in self.0.iter() {
            sum += key.len() + 1;
            for line in value.lines() {
                sum += 1 + line.len() + 1;
            }
        }
        sum
    }
    fn encode<W: io::Write>(&self, writer: &mut W) -> io::Result<usize> {
        let mut sz = 0;
        for (key, value) in self.0.iter() {
            let kd = format!("{}\n", key);
            try!(writer.write_all(kd.as_bytes()));
            sz += kd.len();
            for line in value.lines() {
                let kv = format!(" {}\n", line);
                try!(writer.write_all(kv.as_bytes()));
                sz += kv.len();
            }
        }
        Ok(sz)
    }
}
impl fmt::Display for Extras {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (key, value) in self.0.iter() {
            try!(write!(f, "{}\n", key));
            for line in value.lines() {
                try!(write!(f, " {}\n", line));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Commit<H: Hash> {
    pub tree_ref: TreeRef<H>,
    pub parents: Parents<H>,
    pub author: Person,
    pub committer: Person,
    pub encoding: Option<Encoding>,
    pub extras: Extras,
    pub message: String
}
impl<H: Hash> fmt::Display for Commit<H> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!( f, "tree {}\n", self.tree_ref.to_hexadecimal()));
        for p in self.parents.iter() {
            try!(write!(f, "parent {}\n", p.to_hexadecimal()));
        }
        try!(write!(f, "author {}\ncommitter {}\n", self.author, self.committer));
        if let &Some(ref e) = &self.encoding {
            try!(write!(f, "encoding {}\n", e.raw));
        }
        write!(f, "{}{}", self.extras, self.message)
    }
}
impl<H: Hash> Decoder for Commit<H> {
    fn decode(b: &[u8]) -> nom::IResult<&[u8], Self> {
        nom_parse_commit(b)
    }
}
named!(nom_parse_commit_tag, tag!("commit "));
named!(nom_parse_commit_size<usize>
      , map_res!( map_res!( nom::digit, str::from_utf8), str::FromStr::from_str)
      );
named!(nom_parse_commit_head<usize>
      , chain!(nom_parse_commit_tag ~ r: nom_parse_commit_size ~ char!('\0'), || r)
      );
fn nom_parse_commit<H: Hash>(b: &[u8]) -> nom::IResult<&[u8], Commit<H>> {
    let (b, _) = try_parse!(b, nom_parse_commit_head);
    let (b, _) = try_parse!(b, tag!("tree "));
    let (b, tr) = try_parse!(b, H::decode_hex);
    let (b, _) = try_parse!(b, tag!("\n"));
    let (b, parents) = try_parse!(b, Parents::<H>::decode);
    let (b, _) = try_parse!(b, tag!("author "));
    let (b, a) = try_parse!(b, Person::decode);
    let (b, _) = try_parse!(b, tag!("\n"));
    let (b, _) = try_parse!(b, tag!("committer "));
    let (b, c) = try_parse!(b, Person::decode);
    let (b, _) = try_parse!(b, tag!("\n"));
    let (b, en) = try_parse!(b, opt!(chain!(e: nom_parse_encoding ~ char!('\n'), || e)));
    let (b, e) = try_parse!(b, nom_parse_extras);
    let (b, m) = try_parse!(b, map_res!(nom::rest, str::from_utf8));
    nom::IResult::Done(
        b,
        Commit {
            tree_ref: TreeRef::new(tr),
            parents: parents,
            author: a, committer: c,
            extras: e,
            encoding: en,
            message: m.to_string()
        }
    )
}
impl<H: Hash> Encoder for Commit<H> {
    fn required_size(&self) -> usize {
        0 + H::digest_hex_size() + 6
          + self.parents.required_size()
          + self.author.required_size() + 1
          + self.committer.required_size() + 1
          + match &self.encoding { &Some(ref e) => e.required_size() + 1, &None => 0 }
          + self.extras.required_size()
          + self.message.len()
    }
    fn encode<W: io::Write>(&self, writer: &mut W) -> io::Result<usize> {
        let data = format!("{}", self);
        let head = format!("commit {}\0", data.len());
        try!(writer.write_all(head.as_bytes()));
        try!(writer.write_all(data.as_bytes()));
        Ok(head.len() + data.len())
    }
}

// -- --------------------------------------------------------------------- --
// --                                 Tests                                 --
// -- --------------------------------------------------------------------- --

#[cfg(test)]
mod test {
    //! contract test. It's more to detect changes and make sure
    //! things don't break under our feet without knowing it.

    use super::*;
    use ::protocol::test_decode_encode;
    use rustc_serialize::base64::FromBase64;
    use ::protocol::SHA1;

    const SMOCK_TEST : &'static str =
        "Y29tbWl0IDI0MgB0cmVlIDJlZjk1OTE2MzU2NmYyOWI0YTVhY2I4Y2JlMjE3YzhiMDM2\
         NzQ3YmMKcGFyZW50IDFmYTY4MTFjZjIyYTRjYmVmNWJiMjhlNjhmZTI4ZDcyOGNmMmY2\
         NGQKYXV0aG9yIE5pY29sYXMgRGkgUHJpbWEgPG5pY29sYXNAZGktcHJpbWEuZnI+IDE0\
         ODAwMDc4MzIgKzAxMDAKY29tbWl0dGVyIE5pY29sYXMgRGkgUHJpbWEgPG5pY29sYXNA\
         ZGktcHJpbWEuZnI+IDE0ODAwMDc4MzIgKzAxMDAKCmFkZCB0cmVlIGVuY29kaW5nCg==";

    #[test]
    fn regression_test() {
        let data = SMOCK_TEST.from_base64().unwrap();
        test_decode_encode::<Commit<SHA1>>(data);
    }
}
