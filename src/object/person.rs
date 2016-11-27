//! Git's Person (author and committer)

use super::Date;
use protocol::{Encoder, Decoder};
use nom;
use std::{str, io, fmt};

/// Git's Person data type
///
/// Base type for committer and author (embedding a date along):
///
/// * a full name;
/// * an email address;
/// * a date (the date associated to when the person did something on the current object).
///
/// # Enhancement
///
/// ## Date and Person
///
/// Semantically, it does not make sense to define a `Person`
/// with its actions logged along. A Person's is not defined
/// by when it actually did something.
/// However, in the context of git, a `Committer` and an `Author`
/// will have the same data and will be tagged along a date.
///
/// ## Data Validation
///
/// There is not validation of the username or the email address.
/// This is customer's responsabilities to store valide data (or not).
///
/// # Example
///
/// ```
/// use git::object::Person;
///
/// let me_now = Person::now( "Nicolas".to_string()
///                         , "my@email.address".to_string()
///                         );
/// println!("Hello! Right now, I am: {}", me_now);
/// ```
///
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Person {
    name:  String,
    email: String,
    date:  Date
}

impl Person {
    /// convenient function to create a `Person` with the current Date
    ///
    /// ```
    /// use git::object::Person;
    ///
    /// let me_now = Person::now("nicolas".to_string(), "my@email.address".to_string());
    /// println!("{}", me_now);
    /// ```
    pub fn now(name: String, email: String) -> Self {
        Person::new(name, email, Date::now())
    }
    /// create a new `Person`
    ///
    /// ```
    /// use git::object::Person;
    /// use git::object::Date;
    ///
    /// let now = Date::now();
    /// let me_now = Person::new("nicolas".to_string(), "my@email.address".to_string(), now);
    /// println!("{}", me_now);
    /// ```
    pub fn new(name: String, email: String, date: Date) -> Self {
        Person {
            name: name,
            email: email,
            date: date
        }
    }

    /// access the `Person`'s name
    ///
    /// ```
    /// use git::object::Person;
    ///
    /// let name = r"nicolas";
    /// let email = "my@email.address".to_string();
    /// let me_now = Person::now(name.to_string(), email);
    /// assert_eq!(name, me_now.name());
    /// ```
    pub fn name(&self) -> &str { self.name.as_str() }

    /// access the `Person`'s email address
    ///
    /// ```
    /// use git::object::Person;
    ///
    /// let name = "nicolas".to_string();
    /// let email = r"my@email.address";
    /// let me_now = Person::now(name, email.to_string());
    /// assert_eq!(email, me_now.email());
    /// ```
    pub fn email(&self) -> &str { self.email.as_str() }

    /// access the `Person`'s date
    ///
    /// ```
    /// use git::object::Person;
    /// use git::object::Date;
    ///
    /// let name = "nicolas".to_string();
    /// let email = "my@email.address".to_string();
    /// let date = Date::now();
    /// let me_now = Person::new(name, email, date.clone());
    /// assert_eq!(&date, me_now.date());
    /// ```
    pub fn date(&self) -> &Date { &self.date }

    // not public
    fn new_str(name: &str, email: &str, date: Date) -> Self {
        Person::new(name.to_string(),email.to_string(), date)
    }
    /// not public
    fn encode_name_email(&self) -> String {
        format!("{} <{}> ", self.name, self.email).to_string()
    }
}
impl fmt::Display for Person {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} <{}> {}", self.name, self.email, self.date.encode_for_obj())
    }
}
impl Decoder for Person {
    fn decode(b: &[u8]) -> nom::IResult<&[u8], Self> { nom_parse_person(b) }
}
impl Encoder for Person {
    fn required_size(&self) -> usize {
        self.encode_name_email().len() + self.date.required_size()
    }
    fn encode<W: io::Write>(&self, writer: &mut W) -> io::Result<usize> {
        let ne = self.encode_name_email();
        try!(writer.write_all(ne.as_bytes()));
        let d_len = try!(self.date.encode(writer));
        Ok(ne.len() + d_len)
    }
}

named!( nom_parse_person<Person>
      , chain!( name:  map_res!(take_until_and_consume!(" <"), str::from_utf8)
              ~ email: map_res!(take_until_and_consume!("> "), str::from_utf8)
              ~ date:  call!(Date::decode)
              , || Person::new_str(name, email, date)
              )
      );


// -- --------------------------------------------------------------------- --
// --                                 Tests                                 --
// -- --------------------------------------------------------------------- --

#[cfg(test)]
mod test {
    use super::*;
    use ::protocol::test_encoder_decoder;

    #[test]
    fn serialisable() {
        let p = Person::now("Nicolas".to_string(), "my@email.address".to_string());
        test_encoder_decoder(p);
    }
}
