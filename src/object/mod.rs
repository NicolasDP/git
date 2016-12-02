mod date;
mod person;
mod blob;
mod tree;
mod commit;

pub use self::date::Date;
pub use self::person::Person;
pub use self::blob::{BlobRef, Blob};
pub use self::tree::{TreeRef, Permission, Permissions, PermissionSet, Tree, TreeEnt};
pub use self::commit::{CommitRef, Parents, Commit, Encoding, Extras};

use nom;
use std::fmt;
use protocol::{Hash, Decoder};

pub trait Object<H: Hash> : Decoder{
    type Id;
}
impl<H: Hash> Object<H> for Commit<H> {
    type Id = CommitRef<H>;
}
impl<H: Hash> Object<H> for Tree<H> {
    type Id = TreeRef<H>;
}
impl<H: Hash> Object<H> for Blob {
    type Id = BlobRef<H>;
}

pub enum Obj<H: Hash> {
    Commit(Commit<H>),
    Tree(Tree<H>),
    Blob(Blob)
}
impl<H: Hash> Decoder for Obj<H> {
    fn decode(b: &[u8]) -> nom::IResult<&[u8], Self> {
        use nom::{IResult, Needed};
        if b.len() < 1 {
            return IResult::Incomplete(Needed::Size(1))
        }
        let c : char = b[0] as char;
        match c {
            'c' => Commit::<H>::decode(b).map(|com| Obj::Commit(com)),
            't' => Tree::<H>::decode(b).map(|t| Obj::Tree(t)),
            'b' => Blob::decode(b).map(|bl| Obj::Blob(bl)),
            _   => panic!()
        }
    }
}
impl<H: Hash+fmt::Display> fmt::Display for Obj<H> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Obj::Commit(ref c) => write!(f, "{}", c),
            &Obj::Tree(ref t)   => write!(f, "{}", t),
            &Obj::Blob(ref b)   => write!(f, "{}", b)
        }
    }
}
impl<H: Hash> Object<H> for Obj<H> {
    type Id = H;
}
