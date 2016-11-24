use super::tree::TreeRef;
use super::person::Person;
use protocol::hash::Hasher;
use protocol::encoder::Encoder;
use protocol::decoder::Decoder;
use std::{io, fmt, convert, ops, iter, slice, collections, str};
use nom;

/// Commit reference
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
pub struct CommitRef<H: Hasher>(H);
impl<H: Hasher> CommitRef<H> {
    pub fn new(h: H) -> Self { CommitRef(h) }
}
impl<H: Hasher + fmt::Display> fmt::Display for CommitRef<H> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { fmt::Display::fmt(&self.0, f) }
}

impl<H: Hasher> Hasher for CommitRef<H> {
    fn hash<R: io::BufRead>(data: &mut R) -> io::Result<Self> {
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
impl<H: Hasher> convert::AsRef<H> for CommitRef<H> {
    fn as_ref(&self) -> &H { &self.0 }
}

/// collection of commit parents
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Parents<H: Hasher>(Vec<CommitRef<H>>);
impl<H: Hasher> Parents<H> {
    pub fn new() -> Self { Self::new_with(Vec::new()) }
    pub fn new_with(v: Vec<CommitRef<H>>) -> Self { Parents(v) }
    pub fn push(&mut self, p: CommitRef<H>) { self.0.push(p) }
}
impl<H: Hasher> ops::Deref for Parents<H> {
    type Target = [CommitRef<H>];
    fn deref(&self) -> &Self::Target { self.0.deref() }
}
impl<H: Hasher> ops::DerefMut for Parents<H> {
    fn deref_mut(&mut self) -> &mut Self::Target { self.0.deref_mut() }
}
impl<H: Hasher> iter::FromIterator<CommitRef<H>> for Parents<H> {
    fn from_iter<I: IntoIterator<Item=CommitRef<H>>>(iter: I) -> Self {
        Self::new_with(iter.into_iter().collect())
    }
}
impl<H: Hasher> iter::IntoIterator for Parents<H> {
    type Item = CommitRef<H>;
    type IntoIter = ::std::vec::IntoIter<CommitRef<H>>;
    fn into_iter(self) -> Self::IntoIter { self.0.into_iter() }
}
impl<'a, H: Hasher> iter::IntoIterator for &'a Parents<H> {
    type Item = &'a CommitRef<H>;
    type IntoIter = slice::Iter<'a, CommitRef<H>>;
    fn into_iter(self) -> Self::IntoIter { self.0.iter() }
}
impl<'a, H: Hasher> IntoIterator for &'a mut Parents<H> {
    type Item = &'a mut CommitRef<H>;
    type IntoIter = slice::IterMut<'a, CommitRef<H>>;
    fn into_iter(mut self) -> Self::IntoIter { self.0.iter_mut() }
}
impl<H: Hasher> Encoder for Parents<H> {
    fn encode<W: io::Write>(&self, writer: &mut W) -> io::Result<usize> {
        let mut sz = 0;
        for p in self.iter() {
            sz += try!(encode_parent(p, writer));
        }
        Ok(sz)
    }
}
impl<H: Hasher> Decoder for Parents<H> {
    fn decode(b: &[u8]) -> nom::IResult<&[u8], Self> {
        let mut i = b;
        let mut parents = Self::new();
        while let nom::IResult::Done(b, parent) = nom_parse_parent::<H>(i) {
            i = b;
            parents.push(parent);
        };
        nom::IResult::Done(b, parents)
    }
}
named!(nom_parse_newline, tag!("\n"));
named!(nom_parse_parent_tag, tag!("parent "));
fn nom_parse_parent<H: Hasher>(b: &[u8]) -> nom::IResult<&[u8], CommitRef<H>> {
    let b = match nom_parse_parent_tag(b) {
        nom::IResult::Done(b, _) => b,
        nom::IResult::Error(err) => return nom::IResult::Error(err),
        nom::IResult::Incomplete(n) => return nom::IResult::Incomplete(n)
    };
    let (b, cr) = match CommitRef::<H>::decode_hex(b) {
        nom::IResult::Done(b, cr) => (b, cr),
        nom::IResult::Error(err) => return nom::IResult::Error(err),
        nom::IResult::Incomplete(n) => return nom::IResult::Incomplete(n)
    };
    nom_parse_newline(b).map(|_| cr)
}
fn encode_parent<H: Hasher, W: io::Write>(commit: &CommitRef<H>, writer: &mut W) -> io::Result<usize> {
    try!(writer.write_all("parent ".as_bytes()));
    let s = try!(commit.encode_hex(writer));
    try!(writer.write_all("\n".as_bytes()));
    Ok(s + 8)
}

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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Commit<H: Hasher> {
    pub tree_ref: TreeRef<H>,
    pub parents: Parents<H>,
    pub author: Person,
    pub committer: Person,
    pub extras: Extras,
    pub message: String
}
impl<H: Hasher> Decoder for Commit<H> {
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
fn nom_parse_commit<H: Hasher>(b: &[u8]) -> nom::IResult<&[u8], Commit<H>> {
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
    let (b, e) = try_parse!(b, nom_parse_extras);
    let (b, m) = try_parse!(b, map_res!(nom::rest, str::from_utf8));
    nom::IResult::Done(
        b,
        Commit {
            tree_ref: TreeRef::new(tr),
            parents: parents,
            author: a, committer: c,
            extras: e,
            message: m.to_string()
        }
    )
}

// -- --------------------------------------------------------------------- --
// --                                 Tests                                 --
// -- --------------------------------------------------------------------- --

#[cfg(test)]
mod test {
    //! contract test. It's more to detect changes and make sure
    //! things don't break under our feet without knowing it.

    use super::*;
    use ::protocol::test_encoder_decoder;
    use ::protocol::hash::SHA1;

/*
    #[test]
    fn encode_decode_commit_ref() {
        let sha1_hex = "2aae6c35c94fcfb415dbe95f408b9ce91ee846ed";
        let sha1 = SHA1::from_str(sha1_hex)
                        .expect("expecting a valid SHA1 encoded in hexadecimal");
        let tr = CommitRef::new(sha1);
        test_encoder_decoder(tr);
    }
    */
}
