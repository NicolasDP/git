use std::collections::BTreeSet;

use error::*;
use hash::SHA1;
use ::object::elements::hash::{HashRef, HasHashRef};
use refs::{SpecRef, Ref};
use object::Object;

pub trait Repo {
    /// common function to validate the given Git Repository
    /// is valid. See GitFS.
    fn is_valid(&self) -> Result<()>;

    /// get the description of the given Git Repository
    fn get_description(&self) -> Result<String>;

    /// standard function to get the Hash associated to a given
    /// reference (HEAD, master... remote/master...)
    fn get_ref(&self, r: SpecRef) -> Result<Ref<SHA1>>;

    /// follow the links for a given Ref until a HashRef.
    /// HEAD -> master -> abcdef012345678..
    ///
    /// This function is a combination of get_ref and patten match on the Ref
    /// enumeration.
    fn get_ref_follow_links(&self, r: SpecRef) -> Result<HashRef<SHA1>> {
        match try!(self.get_ref(r)) {
            Ref::Link(r) => self.get_ref_follow_links(r),
            Ref::Hash(h) => Ok(h)
        }
    }

    /// get object from a given hash ref
    fn get_object<T: HasHashRef<SHA1>>(&self, r: &T) -> Result<Object<SHA1>>;
    /// default implementation to read an object (a commit if Ref is a SpecRef)
    /// from a given Ref.
    ///
    /// This default implementation is a combination of get_ref_follow_links,
    /// pattern match and get_object.
    fn get_object_ref(&self, r: Ref<SHA1>) -> Result<Object<SHA1>> {
        let hr : Result<HashRef<SHA1>> = match r {
            Ref::Link(sr) => self.get_ref_follow_links(sr),
            Ref::Hash(hr) => Ok(hr)
        };
        hr.and_then(|r| self.get_object(&r))
    }

    fn get_head(&self) -> Result<Ref<SHA1>> { self.get_ref(SpecRef::Head) }
    fn list_branches(&self) -> Result<BTreeSet<SpecRef>>;
    fn list_remotes(&self) -> Result<BTreeSet<SpecRef>>;
    fn list_tags(&self) -> Result<BTreeSet<SpecRef>>;
}
