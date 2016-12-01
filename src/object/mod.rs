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
