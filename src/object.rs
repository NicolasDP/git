use hash;
use hash::{SHA1, HashRef};
use std::collections::BTreeMap;
use std;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::str;
use std::ops::{Deref, DerefMut};
use std::iter::{FromIterator, IntoIterator};
use std::slice;

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
/// assert_eq!(date.provide_size(), str.len());
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
    fn provide_size(&self) -> usize {
        let b = format!("{}", self);
        b.len()
    }
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
/// assert_eq!(person.provide_size(), str.len());
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
    fn provide_size(&self) -> usize {
        self.name.len() + self.email.len() + self.date.provide_size() + 4
    }
    fn nom_parse(b: & [u8]) -> nom::IResult<&[u8], Self> { nom_parse_person(b) }
}

/// contains the HashRef to a git tree
///
/// # Example
///
/// ```
/// use git::object::{TreeRef, Objectable};
///
/// let bytes = &b"tree 3351570ee30575ccfc99b2ef17348515c54289e8"[..];
/// let tree_ref = TreeRef::parse_bytes(bytes).unwrap();
///
/// let bytes2 = format!("{}", tree_ref);
/// assert_eq!(bytes, bytes2.as_bytes());
/// assert_eq!(tree_ref.provide_size(), bytes.len());
/// ```
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
    fn provide_size(&self) -> usize {
        self.tree_ref.provide_size() + 5
    }
    fn nom_parse(b: &[u8]) -> nom::IResult<&[u8], Self> { nom_parse_tree_ref(b) }
}

/// parent commit ref
///
/// # Example
///
/// ```
/// use git::object::{Parent, Objectable};
///
/// let bytes = &b"parent 3351570ee30575ccfc99b2ef17348515c54289e8"[..];
/// let parent = Parent::parse_bytes(bytes).unwrap();
///
/// let bytes2 = format!("{}", parent);
/// assert_eq!(bytes, bytes2.as_bytes());
/// assert_eq!(parent.provide_size(), bytes.len());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Parent<Hash: hash::Property+hash::Hasher> {
    parent_ref: hash::HashRef<Hash>
}
impl<Hash: hash::Property+hash::Hasher> Parent<Hash> {
    pub fn new(pr: hash::HashRef<Hash>) -> Self { Parent { parent_ref: pr } }
}
impl<Hash: hash::Property+hash::Hasher> Display for Parent<Hash> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "parent {}", self.parent_ref)
    }
}
named!( nom_parse_commit_parent<Parent<SHA1> >
      , chain!( tag!("parent ")
              ~ r: call!(HashRef::nom_parse)
              , || Parent::new(r)
              )
      );
impl Objectable for Parent<SHA1> {
    fn provide_size(&self) -> usize { self.parent_ref.provide_size() + 7 }
    fn nom_parse(b: &[u8]) -> nom::IResult<&[u8], Self> { nom_parse_commit_parent(b) }
}

/// collection of commit parents
///
/// # Examples
///
/// ```
/// use git::object::{Parent, Parents, Objectable};
/// use git::hash::SHA1;
///
/// let bytes1 = &b"parent 3351570ee30575ccfc99b2ef17348515c54289e8"[..];
/// let bytes2 = &b"parent 48234be6fe82eebd92f70a8add2a1fbab64f6707"[..];
/// let parent1 = Parent::parse_bytes(bytes1).unwrap();
/// let parent2 = Parent::parse_bytes(bytes2).unwrap();
///
/// let mut parents : Parents<SHA1> = Parents::new();
/// parents.push(parent1);
/// parents.push(parent2);
///
/// let serialised = format!("{}", parents);
/// let parents2 = Parents::parse_bytes(serialised.as_bytes()).unwrap();
/// assert_eq!(parents, parents2);
/// assert_eq!(parents.provide_size(), bytes1.len() + bytes2.len() + 2);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Parents<Hash: hash::Property+hash::Hasher> {
    parents: Vec<Parent<Hash>>
}
impl<Hash: hash::Property+hash::Hasher> Parents<Hash> {
    pub fn new() -> Self { Self::new_with(Vec::new()) }
    pub fn new_with(v: Vec<Parent<Hash>>) -> Self { Parents { parents: v } }
    pub fn push(&mut self, p: Parent<Hash>) { self.parents.push(p) }
}
impl<Hash: hash::Property+hash::Hasher> Deref for Parents<Hash> {
    type Target = [Parent<Hash>];
    fn deref(&self) -> &[Parent<Hash>] { self.parents.deref() }
}
impl<Hash: hash::Property+hash::Hasher> DerefMut for Parents<Hash> {
    fn deref_mut(&mut self) -> &mut [Parent<Hash>] { self.parents.deref_mut() }
}
impl<Hash: hash::Property+hash::Hasher> FromIterator<Parent<Hash>> for Parents<Hash> {
    fn from_iter<I: IntoIterator<Item=Parent<Hash>>>(iter: I) -> Self {
        Self::new_with(iter.into_iter().collect())
    }
}
impl<Hash: hash::Property+hash::Hasher> IntoIterator for Parents<Hash> {
    type Item = Parent<Hash>;
    type IntoIter = ::std::vec::IntoIter<Parent<Hash>>;
    fn into_iter(self) -> Self::IntoIter { self.parents.into_iter() }
}
impl<'a, Hash: hash::Property+hash::Hasher> IntoIterator for &'a Parents<Hash> {
    type Item = &'a Parent<Hash>;
    type IntoIter = slice::Iter<'a, Parent<Hash>>;
    fn into_iter(self) -> slice::Iter<'a, Parent<Hash>> { self.parents.iter() }
}
impl<'a, Hash: hash::Property+hash::Hasher> IntoIterator for &'a mut Parents<Hash> {
    type Item = &'a mut Parent<Hash>;
    type IntoIter = slice::IterMut<'a, Parent<Hash>>;
    fn into_iter(mut self) -> slice::IterMut<'a, Parent<Hash>> { self.parents.iter_mut() }
}

impl<Hash: hash::Property+hash::Hasher> Display for Parents<Hash> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut t = Ok(());
        for ref parent in self {
            t = write!(f, "{}\n", parent);
        }
        t
    }
}

named!( nom_parse_commit_parents<Parents<SHA1> >
      , chain!( v: many0!(chain!(parent: call!(Parent::nom_parse) ~ char!('\n'), || parent))
              , || Parents::new_with(v)
              )
      );
impl Objectable for Parents<SHA1> {
    fn provide_size(&self) -> usize {
        let mut sum = 0;
        for ref parent in self.parents.iter() {
            sum += parent.provide_size() + 1;
        }
        sum
    }
    fn nom_parse(b: &[u8]) -> nom::IResult<&[u8], Self> { nom_parse_commit_parents(b) }
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
impl Display for Encoding {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "encoding {}", self.raw)
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
named!( nom_parse_encoding<Encoding>
      , chain!( tag!("encoding ")
              ~ e: map_res!(take_while1!(is_valid_encoding_char), str::from_utf8)
              , || Encoding::new_str(e)
              )
      );
impl Objectable for Encoding {
    fn provide_size(&self) -> usize { self.raw.len() + 9 }
    fn nom_parse(b: &[u8]) -> nom::IResult<&[u8], Self> { nom_parse_encoding(b) }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Extras {
    extras: BTreeMap<String, String>
}
impl Extras {
    pub fn new() -> Self { Extras { extras: BTreeMap::new() } }
    fn new_with(btm: BTreeMap<String, String>) -> Self {
        Extras { extras: btm }
    }
}
impl IntoIterator for Extras {
    type Item = (String, String);
    type IntoIter = ::std::collections::btree_map::IntoIter<String, String>;
    fn into_iter(self) -> ::std::collections::btree_map::IntoIter<String, String> {
        self.extras.into_iter()
    }
}
impl<'a> IntoIterator for &'a Extras {
    type Item = (&'a String, &'a String);
    type IntoIter = ::std::collections::btree_map::Iter<'a, String, String>;
    fn into_iter(self) -> ::std::collections::btree_map::Iter<'a, String, String> {
        self.extras.iter()
    }
}
impl<'a> IntoIterator for &'a mut Extras {
    type Item = (&'a String, &'a mut String);
    type IntoIter = ::std::collections::btree_map::IterMut<'a, String, String>;
    fn into_iter(self) -> ::std::collections::btree_map::IterMut<'a, String, String> {
        self.extras.iter_mut()
    }
}
impl FromIterator<(String, String)> for Extras {
    fn from_iter<T: IntoIterator<Item=(String, String)>>(iter: T) -> Extras {
        Extras::new_with(BTreeMap::from_iter(iter))
    }
}
impl Extend<(String, String)> for Extras {
    fn extend<T: IntoIterator<Item=(String, String)>>(&mut self, iter: T) {
        self.extras.extend(iter)
    }
}
impl Display for Extras {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (key, value) in self.extras.iter() {
            try!(write!(f, "{}\n", key));
            for line in value.lines() {
                try!(write!(f, " {}\n", line));
            }
        }
        Ok(())
    }
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
      , chain!( mut acc: value!(BTreeMap::new())
              ~ many0!( tap!(v: parse_extra => acc.insert(v.0.clone(), v.1.clone())))
              , || Extras::new_with(acc)
              )
      );
impl Objectable for Extras {
    fn provide_size(&self) -> usize {
        let mut sum : usize = 0;
        for (key, value) in self.extras.iter() {
            sum += key.len() + 1;
            for line in value.lines() {
                sum += 1 + line.len() + 1;
            }
        }
        sum
    }
    fn nom_parse(b: &[u8]) -> nom::IResult<&[u8], Self> { nom_parse_extras(b) }
}

named!( nom_parse_commit<Commit<SHA1> >
      , chain!( tag!("commit ") ~ many1!(nom::digit) ~ char!('\0')
              ~ tree_ref: call!(TreeRef::nom_parse) ~ char!('\n')
              ~ parents:  call!(Parents::nom_parse)
              ~ author: call!(Author::nom_parse) ~ char!('\n')
              ~ committer: call!(Committer::nom_parse) ~ char!('\n')
              ~ encoding: opt!(chain!(e: call!(Encoding::nom_parse) ~ char!('\n'), || e))
              ~ extras: call!(Extras::nom_parse)
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Author {
    author: Person
}
impl Author {
    pub fn new(p: Person) -> Self { Author { author: p } }
}
impl Display for Author {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "author {}", self.author) }
}
named!( nom_parse_author<Author>
      , chain!( tag!("author ")
              ~ author: call!(Person::nom_parse)
              , || Author::new(author)
              )
      );
impl Objectable for Author {
    fn provide_size(&self) -> usize { self.author.provide_size() + 7 }
    fn nom_parse(b: &[u8]) -> nom::IResult<&[u8], Self> { nom_parse_author(b) }
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Committer {
    committer: Person
}
impl Committer {
    pub fn new(p: Person) -> Self { Committer { committer: p } }
}
impl Display for Committer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "committer {}", self.committer) }
}
named!( nom_parse_committer<Committer>
      , chain!( tag!("committer ")
              ~ committer: call!(Person::nom_parse)
              , || Committer::new(committer)
              )
      );
impl Objectable for Committer {
    fn provide_size(&self) -> usize { self.committer.provide_size() + 9 }
    fn nom_parse(b: &[u8]) -> nom::IResult<&[u8], Self> { nom_parse_committer(b) }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Commit<Hash: hash::Property+hash::Hasher> {
    pub tree_ref: TreeRef<Hash>,
    pub parents: Parents<Hash>,
    pub author: Author,
    pub committer: Committer,
    pub encoding: Option<Encoding>,
    pub extras: Extras,
    pub message: String
}

impl Objectable for Commit<SHA1> {
    fn provide_size(&self) -> usize {
        0 + self.tree_ref.provide_size() + 1
          + self.parents.provide_size()
          + self.author.provide_size() + 1
          + self.committer.provide_size() + 1
          + match &self.encoding { &Some(ref e) => e.provide_size() + 1, &None => 0 }
          + self.extras.provide_size()
          + self.message.len()
    }
    fn nom_parse(b: &[u8]) -> nom::IResult<&[u8], Self> {
        nom_parse_commit(b)
    }
}
impl Display for Commit<SHA1> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "commit {}\0", self.provide_size()));
        try!(write!(f, "{}\n", self.tree_ref));
        try!(write!(f, "{}", self.parents));
        try!(write!(f, "{}\n", self.author));
        try!(write!(f, "{}\n", self.committer));
        if let &Some(ref e) = &self.encoding {
            try!(write!(f, "{}\n", e));
        }
        try!(write!(f, "{}", self.extras));
        write!(f, "\n{}", self.message)
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
