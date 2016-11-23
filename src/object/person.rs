use super::date::Date;
use protocol::decoder::*;
use protocol::encoder::*;
use nom;
use std::{str, io};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Person {
    pub name:  String,
    pub email: String,
    pub date:  Date
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
impl Decoder for Person {
    fn decode(b: &[u8]) -> nom::IResult<&[u8], Self> { nom_parse_person(b) }
}
impl Encoder for Person {
    fn encode<W: io::Write>(&self, writer: &mut W) -> io::Result<usize> {
        let hex = format!("{} <{}> ", self.name, self.email).to_string();
        try!(writer.write_all(hex.as_bytes()));
        let d_len = try!(self.date.encode(writer));
        Ok(hex.len() + d_len)
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
    //! contract test. It's more to detect changes and make sure
    //! things don't break under our feet without knowing it.

    use super::*;
    use ::object::date::Date;
    use ::protocol::test_encoder_decoder;

    #[test]
    fn serialisable() {
        let p = Person::new_str("Nicolas", "nicolas@di-prima.fr", Date::now());
        test_encoder_decoder(p);
    }
}
