
use std::path::{PathBuf, Path, Component};
use std::str::FromStr;
use std::fmt;
use error::{GitError, Result};
use hash::{Property, Hasher, HashRef, HasHashRef};

pub type RefName = PathBuf;

/// Special reference
///
/// # Examples
///
/// ```
/// use git::SpecRef;
/// use std::str::FromStr;
/// use std::path::PathBuf;
/// let master_branch = SpecRef::Branch(PathBuf::from("master"));
/// let master_ref = SpecRef::from_str("refs/heads/master").unwrap();
/// assert_eq!(master_branch, master_ref);
/// ```
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub enum SpecRef {
    Tag(RefName),
    Branch(RefName),
    Remote(RefName, RefName),
    Patch(RefName),
    Stash,
    Head,
    OriginHead,
    FetchHead
}

impl SpecRef {
    pub fn tag<T: AsRef<Path>>(bn: T)    -> Self { SpecRef::Tag(PathBuf::new().join(bn)) }
    pub fn branch<T: AsRef<Path>>(bn: T) -> Self { SpecRef::Branch(PathBuf::new().join(bn)) }
    pub fn remote<R, T>(r: R, bn: T) -> Self
        where R: AsRef<Path>, T: AsRef<Path>
    {
        SpecRef::Remote( PathBuf::new().join(r)
                       , PathBuf::new().join(bn)
                       )
    }
    pub fn patch<T: AsRef<Path>>(bn: T)  -> Self { SpecRef::Patch(PathBuf::new().join(bn)) }
    pub fn stash()                       -> Self { SpecRef::Stash }
    pub fn head()                        -> Self { SpecRef::Head }
    pub fn origin_head()                 -> Self { SpecRef::OriginHead }
    pub fn fetch_head()                  -> Self { SpecRef::FetchHead }
}

impl FromStr for SpecRef {
    type Err = GitError;
    fn from_str(s: &str) -> Result<Self> {
        let refnstr = PathBuf::from(s.trim_right());
        let mut components = refnstr.components();
        if let Some(Component::Normal(r)) = components.next() {
            if r == "refs" {
                if let Some(Component::Normal(t)) = components.next() {
                    if t == "tags"    { return Ok(SpecRef::tag(components.as_path())) }
                    if t == "heads"   { return Ok(SpecRef::branch(components.as_path())) }
                    if t == "patches" { return Ok(SpecRef::patch(components.as_path())) }
                    if t == "stash"   { return Ok(SpecRef::stash()) }
                    if t == "remotes" {
                        if let Some(Component::Normal(rem)) = components.next() {
                            return Ok(SpecRef::remote(rem, components.as_path()))
                        }
                    }
                }
            }
            if r == "HEAD"       { return Ok (SpecRef::head()) }
            if r == "ORIG_HEAD"  { return Ok (SpecRef::origin_head()) }
            if r == "FETCH_HEAD" { return Ok (SpecRef::fetch_head()) }
        };
        return Err(GitError::InvalidRef(refnstr.clone()))
    }
}
impl fmt::Display for SpecRef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &SpecRef::Tag(ref t)    => write!(f, "refs/tags/{}", t.display()),
            &SpecRef::Branch(ref b) => write!(f, "refs/heads/{}", b.display()),
            &SpecRef::Remote(ref r, ref b) => write!(f, "refs/remotes/{}/{}", r.display(), b.display()),
            &SpecRef::Patch(ref p)  => write!(f, "refs/patches/{}", p.display()),
            &SpecRef::Stash         => write!(f, "refs/stash"),
            &SpecRef::Head          => write!(f, "HEAD"),
            &SpecRef::OriginHead    => write!(f, "ORIG_HEAD"),
            &SpecRef::FetchHead     => write!(f, "FETCH_HEAD"),
        }
    }
}
impl From<SpecRef> for PathBuf {
    fn from(sr: SpecRef) -> Self {
        let mut pb = PathBuf::new();
        pb.push(format!("{}", &sr));
        pb
    }
}
impl<'a> From<&'a SpecRef> for PathBuf {
    fn from(sr: &'a SpecRef) -> Self {
        let mut pb = PathBuf::new();
        pb.push(format!("{}", sr));
        pb
    }
}


/// Ref
///
/// # Examples
///
/// ```
/// use git::{SpecRef, Ref, SHA1};
/// use std::str::FromStr;
/// use std::path::PathBuf;
/// let master_branch : Ref<SHA1> = Ref::Link(SpecRef::Branch(PathBuf::from("master")));
/// let master_ref : Ref<SHA1> = Ref::from_str("ref: refs/heads/master").unwrap();
/// assert_eq!(master_branch, master_ref);
/// ```
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub enum Ref<Hash: Property + Hasher> {
    Hash(HashRef<Hash>),
    Link(SpecRef)
}
impl<Hash: Property + Hasher> Ref<Hash> {
    pub fn hash<T: HasHashRef<Hash> >(t: &T) -> Self { Ref::Hash(t.hash_ref()) }
    pub fn link(sr: SpecRef) -> Self { Ref::Link(sr) }
}
impl<Hash: Property + Hasher> From<Ref<Hash>> for PathBuf {
    fn from(sr: Ref<Hash>) -> Self {
        let pb = PathBuf::new();
        match sr {
            Ref::Hash(hr) => pb.join("objects").join(hr.path()),
            Ref::Link(sr) => PathBuf::from(sr)
        }
    }
}
impl<'a, Hash: Property + Hasher> From<&'a Ref<Hash>> for PathBuf {
    fn from(sr: &'a Ref<Hash>) -> Self {
        let pb = PathBuf::new();
        match sr {
            &Ref::Hash(ref hr) => pb.join("objects").join(hr.path()),
            &Ref::Link(ref sr) => PathBuf::from(sr)
        }
    }
}

impl<Hash: Property+Hasher> fmt::Display for Ref<Hash> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Ref::Hash(ref r)   => write!(f, "{}", r),
            &Ref::Link(ref b)   => write!(f, "ref: {}", b)
        }
    }
}

impl<Hash: Property+Hasher> FromStr for Ref<Hash> {
    type Err = GitError;
    fn from_str(s: &str) -> Result<Self> {
        if s.starts_with("ref: ") {
            let sub :&str = &s[5..];
            return Ok(Ref::Link(try!(SpecRef::from_str(sub))));
        }

        Ok(Ref::Hash(try!(HashRef::from_str(s))))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hash::SHA1;
    use std::str::FromStr;

    fn get_specref() -> [SpecRef; 9] {
        [ SpecRef::tag("v-1.1")
        , SpecRef::branch("master")
        , SpecRef::branch("dev/stage")
        , SpecRef::remote("origin", "master")
        , SpecRef::patch("patch-file")
        , SpecRef::stash()
        , SpecRef::head()
        , SpecRef::origin_head()
        , SpecRef::fetch_head()
        ]
    }
    fn get_ref() -> [Ref<SHA1>; 9] {
        [ Ref::Link(SpecRef::tag("v-1.1"))
        , Ref::Link(SpecRef::branch("master"))
        , Ref::Link(SpecRef::branch("dev/stage"))
        , Ref::Link(SpecRef::remote("origin", "master"))
        , Ref::Link(SpecRef::patch("patch-file"))
        , Ref::Link(SpecRef::stash())
        , Ref::Link(SpecRef::head())
        , Ref::Link(SpecRef::origin_head())
        , Ref::Link(SpecRef::fetch_head())
        ]
    }

    #[test]
    fn encode_decode_specref() {
        for sr in get_specref().iter() {
            let s = format!("{}", &sr);
            match SpecRef::from_str(&s) {
                Err(err) => panic!("{:?}", err),
                Ok(res) => {
                    let s_ = format!("{}", &res);
                    println!("{:?}", &res);
                    assert_eq!(s, s_);
                    assert_eq!(sr, &res)
                }
            }
        }
    }

    #[test]
    fn encode_decode_ref() {
        for sr in get_ref().iter() {
            let s = format!("{}", &sr);
            match Ref::from_str(&s) {
                Err(err) => panic!("{:?}", err),
                Ok(res) => {
                    let s_ = format!("{}", &res);
                    println!("{:?}", &res);
                    assert_eq!(s, s_);
                    assert_eq!(sr, &res)
                }
            }
        }
    }
}
