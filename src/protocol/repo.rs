use std::collections::BTreeSet;

use error::*;
//use ::hash::SHA1;
//use ::object::elements::hash::{HashRef, HasHashRef};
use refs::{SpecRef, Ref};
use object::{Obj, Object, Commit, CommitRef};
use super::Hash;

pub trait Repo {
    /// common function to validate the given Git Repository
    /// is valid. See GitFS.
    fn is_valid(&self) -> Result<()>;

    /// get the description of the given Git Repository
    fn get_description(&self) -> Result<String>;

    /// standard function to get the Hash associated to a given
    /// reference (HEAD, master... remote/master...)
    fn get_ref<H>(&self, r: SpecRef) -> Result<Ref<H>>
        where H: Hash;

    /// follow the links for a given Ref until a HashRef.
    /// HEAD -> master -> abcdef012345678..
    ///
    /// This function is a combination of get_ref and patten match on the Ref
    /// enumeration.
    fn get_ref_follow_links<H>(&self, r: SpecRef)
        -> Result<H>
        where H: Hash
    {
        match try!(self.get_ref(r)) {
            Ref::Link(r) => self.get_ref_follow_links(r),
            Ref::Hash(h) => Ok(h)
        }
    }

    /// get object from a given hash ref
    fn get_object<H, O>(&self, r: O::Id) -> Result<O>
        where H: Hash
            , O: Object<H>
            , O::Id: Hash;
    fn get_object_<H>(&self, r: H) -> Result<Obj<H>> where H:Hash;

    /// default implementation to read an object (a commit if Ref is a SpecRef)
    /// from a given Ref.
    ///
    /// This default implementation is a combination of get_ref_follow_links,
    /// pattern match and get_object.
    fn get_object_ref<H>(&self, r: Ref<CommitRef<H>>) -> Result<Commit<H>>
        where H: Hash
    {
        let hr = match r {
            Ref::Link(sr) => CommitRef::new(try!(self.get_ref_follow_links(sr))),
            Ref::Hash(hr) => hr
        };
        self.get_object(hr)
    }

    fn get_head<H: Hash>(&self) -> Result<Ref<H>> { self.get_ref(SpecRef::Head) }
    fn list_branches(&self) -> Result<BTreeSet<SpecRef>>;
    fn list_remotes(&self) -> Result<BTreeSet<SpecRef>>;
    fn list_tags(&self) -> Result<BTreeSet<SpecRef>>;
}
