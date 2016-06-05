use std::collections::BTreeSet;

use error::*;
use hash::{SHA1, HashRef};
use refs::{SpecRef, Ref};
use object::Object;

pub trait Repo {
    fn is_valid(&self) -> Result<()>;

    fn get_description(&self) -> Result<String>;
    fn get_ref(&self, r: SpecRef) -> Result<Ref<SHA1>>;
    fn get_ref_follow_links(&self, r: SpecRef) -> Result<HashRef<SHA1>> {
        match try!(self.get_ref(r)) {
            Ref::Link(r) => self.get_ref_follow_links(r),
            Ref::Hash(h) => Ok(h)
        }
    }
    fn get_object(&self, r: Ref<SHA1>) -> Result<Object<SHA1>>;

    fn get_head(&self) -> Result<Ref<SHA1>> { self.get_ref(SpecRef::Head) }
    fn list_branches(&self) -> Result<BTreeSet<SpecRef>>;
    fn list_remotes(&self) -> Result<BTreeSet<SpecRef>>;
    fn list_tags(&self) -> Result<BTreeSet<SpecRef>>;
}
