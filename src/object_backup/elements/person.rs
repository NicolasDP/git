use nom;
use std::{fmt, str};

use ::elements::date::Date;
use ::objectable::{Readable, Writable};

/// Git Person
///
/// * a name
/// * an email address
/// * a git::Date
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

impl fmt::Display for Person {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} <{}> {}", self.name, self.email, self.date)
    }
}

impl Writable for Person {
    fn provide_size(&self) -> usize {
        self.name.len() + self.email.len() + self.date.provide_size() + 4
    }
    fn serialise(&self, f: &mut fmt::Formatter) -> fmt::Result {
        serialise!(f, self.name, " <", self.email, "> ", self.date)
    }
}
impl Readable for Person {
    fn nom_parse(b: & [u8]) -> nom::IResult<&[u8], Self> { nom_parse_person(b) }
}

/// Git Author
///
/// # Example
///
/// ```
/// use git::object::elements::date::*;
/// use git::object::elements::person::{Author};
/// use git::object::{Readable, Writable};
///
/// let date = Date::new(Elapsed(1464729412), Timezone(60));
/// let author = Author::new_str("Kevin Flynn", "kev@flynn.rs", date);
/// let str = format!("{}", author);
/// let author2 = Author::parse_bytes(str.as_bytes()).unwrap();
/// assert_eq!(author, author2);
/// assert_eq!(author.provide_size(), str.len());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Author {
    author: Person
}
impl Author {
    pub fn new(p: Person) -> Self { Author { author: p } }
    pub fn new_str(n: &str, e: &str, d: Date) -> Self {
        Author { author: Person::new_str(n, e, d) }
    }
}
impl fmt::Display for Author {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { self.serialise(f) }
}
named!( nom_parse_author<Author>
      , chain!( tag!("author ")
              ~ author: call!(Person::nom_parse)
              , || Author::new(author)
              )
      );
impl Writable for Author {
    fn provide_size(&self) -> usize { self.author.provide_size() + 7 }
    fn serialise(&self, f: &mut fmt::Formatter) -> fmt::Result {
        serialise!(f, "author ", self.author)
    }
}
impl Readable for Author {
    fn nom_parse(b: &[u8]) -> nom::IResult<&[u8], Self> { nom_parse_author(b) }
}
/// Git Committer
///
/// # Example
///
/// ```
/// use git::object::elements::date::*;
/// use git::object::elements::person::{Committer};
/// use git::object::{Readable, Writable};
///
/// let date = Date::new(Elapsed(1464729412), Timezone(60));
/// let committer = Committer::new_str("Kevin Flynn", "kev@flynn.rs", date);
/// let str = format!("{}", committer);
/// let committer2 = Committer::parse_bytes(str.as_bytes()).unwrap();
/// assert_eq!(committer, committer2);
/// assert_eq!(committer.provide_size(), str.len());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Committer {
    committer: Person
}
impl Committer {
    pub fn new(p: Person) -> Self { Committer { committer: p } }
    pub fn new_str(n: &str, e: &str, d: Date) -> Self {
        Committer { committer: Person::new_str(n, e, d) }
    }
}
impl fmt::Display for Committer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { self.serialise(f) }
}
named!( nom_parse_committer<Committer>
      , chain!( tag!("committer ")
              ~ committer: call!(Person::nom_parse)
              , || Committer::new(committer)
              )
      );
impl Writable for Committer {
    fn provide_size(&self) -> usize { self.committer.provide_size() + 10 }
    fn serialise(&self, f: &mut fmt::Formatter) -> fmt::Result {
        serialise!(f, "committer ", self.committer)
    }
}
impl Readable for Committer {
    fn nom_parse(b: &[u8]) -> nom::IResult<&[u8], Self> { nom_parse_committer(b) }
}
