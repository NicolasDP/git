#[macro_use]
pub mod objectable;
pub mod elements;

use hash;
use hash::{SHA1, HashRef};
use std::collections::{BTreeMap, BTreeSet, btree_set};
use std::fmt;
use std::fmt::{Display};
use std::str;
use std::ops::{Deref, DerefMut};
use std::iter::{FromIterator, IntoIterator};
use std::slice;
use std::path::PathBuf;
use std::cmp::Ordering;
use std::borrow::Borrow;
use std::ops::{Sub, BitOr, BitXor, BitAnd};

use nom;

pub use objectable::{Readable, Writable, Objectable};
use elements::person::{Author, Committer};

/// contains the HashRef to a git tree
///
/// # Example
///
/// ```
/// use git::object::{TreeRef, Writable, Readable};
///
/// let bytes = &b"tree 3351570ee30575ccfc99b2ef17348515c54289e8"[..];
/// let tree_ref = TreeRef::parse_bytes(bytes).unwrap();
///
/// let bytes2 = format!("{}", tree_ref);
/// assert_eq!(bytes, bytes2.as_bytes());
/// assert_eq!(tree_ref.provide_size(), bytes.len());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TreeRef<Hash: hash::Property> {
    tree_ref: hash::HashRef<Hash>
}
impl<Hash: hash::Property> TreeRef<Hash> {
    pub fn new(tr: hash::HashRef<Hash>) -> Self { TreeRef { tree_ref: tr } }
}
impl<Hash: hash::Property> hash::HasHashRef<Hash> for TreeRef<Hash> {
    fn hash_ref(&self) -> HashRef<Hash> { self.tree_ref.hash_ref() }
}
impl<Hash: hash::Property> Display for TreeRef<Hash> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { self.serialise(f) }
}
named!( nom_parse_tree_ref<TreeRef<SHA1> >
      , chain!( tag!("tree ") ~ r: call!(Readable::nom_parse)
              , || TreeRef::new(r)
              )
      );
impl Readable for TreeRef<SHA1> {
    fn nom_parse(b: &[u8]) -> nom::IResult<&[u8], Self> { nom_parse_tree_ref(b) }
}
impl<Hash: hash::Property> Writable for TreeRef<Hash> {
    fn serialise(&self, f: &mut fmt::Formatter) -> fmt::Result {
        serialise!(f, "tree ", self.tree_ref)
    }
    fn provide_size(&self) -> usize { self.tree_ref.provide_size() + 5 }
}

/// parent commit ref
///
/// # Example
///
/// ```
/// use git::object::{Parent, Readable, Writable};
///
/// let bytes = &b"parent 3351570ee30575ccfc99b2ef17348515c54289e8"[..];
/// let parent = Parent::parse_bytes(bytes).unwrap();
///
/// let bytes2 = format!("{}", parent);
/// assert_eq!(bytes, bytes2.as_bytes());
/// assert_eq!(parent.provide_size(), bytes.len());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Parent<Hash: hash::Property> {
    parent_ref: hash::HashRef<Hash>
}
impl<Hash: hash::Property> Parent<Hash> {
    pub fn new(pr: hash::HashRef<Hash>) -> Self { Parent { parent_ref: pr } }
}
impl<Hash: hash::Property> Display for Parent<Hash> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { self.serialise(f) }
}
named!( nom_parse_commit_parent<Parent<SHA1> >
      , chain!( tag!("parent ")
              ~ r: call!(HashRef::nom_parse)
              , || Parent::new(r)
              )
      );
impl Readable for Parent<SHA1> {
    fn nom_parse(b: &[u8]) -> nom::IResult<&[u8], Self> { nom_parse_commit_parent(b) }
}
impl<Hash: hash::Property> Writable for Parent<Hash> {
    fn serialise(&self, f: &mut fmt::Formatter) -> fmt::Result {
        serialise!(f, "parent ", self.parent_ref)
    }
    fn provide_size(&self) -> usize { self.parent_ref.provide_size() + 7 }
}

/// collection of commit parents
///
/// # Examples
///
/// ```
/// use git::object::{Parent, Parents, Readable, Writable};
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
pub struct Parents<Hash: hash::Property> {
    parents: Vec<Parent<Hash>>
}
impl<Hash: hash::Property> Parents<Hash> {
    pub fn new() -> Self { Self::new_with(Vec::new()) }
    pub fn new_with(v: Vec<Parent<Hash>>) -> Self { Parents { parents: v } }
    pub fn push(&mut self, p: Parent<Hash>) { self.parents.push(p) }
}
impl<Hash: hash::Property> Deref for Parents<Hash> {
    type Target = [Parent<Hash>];
    fn deref(&self) -> &[Parent<Hash>] { self.parents.deref() }
}
impl<Hash: hash::Property> DerefMut for Parents<Hash> {
    fn deref_mut(&mut self) -> &mut [Parent<Hash>] { self.parents.deref_mut() }
}
impl<Hash: hash::Property> FromIterator<Parent<Hash>> for Parents<Hash> {
    fn from_iter<I: IntoIterator<Item=Parent<Hash>>>(iter: I) -> Self {
        Self::new_with(iter.into_iter().collect())
    }
}
impl<Hash: hash::Property> IntoIterator for Parents<Hash> {
    type Item = Parent<Hash>;
    type IntoIter = ::std::vec::IntoIter<Parent<Hash>>;
    fn into_iter(self) -> Self::IntoIter { self.parents.into_iter() }
}
impl<'a, Hash: hash::Property> IntoIterator for &'a Parents<Hash> {
    type Item = &'a Parent<Hash>;
    type IntoIter = slice::Iter<'a, Parent<Hash>>;
    fn into_iter(self) -> slice::Iter<'a, Parent<Hash>> { self.parents.iter() }
}
impl<'a, Hash: hash::Property> IntoIterator for &'a mut Parents<Hash> {
    type Item = &'a mut Parent<Hash>;
    type IntoIter = slice::IterMut<'a, Parent<Hash>>;
    fn into_iter(mut self) -> slice::IterMut<'a, Parent<Hash>> { self.parents.iter_mut() }
}

impl<Hash: hash::Property> Display for Parents<Hash> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { self.serialise(f) }
}

named!( nom_parse_commit_parents<Parents<SHA1> >
      , chain!( v: many0!(chain!(parent: call!(Parent::nom_parse) ~ char!('\n'), || parent))
              , || Parents::new_with(v)
              )
      );
impl<Hash: hash::Property> Writable for Parents<Hash> {
    fn provide_size(&self) -> usize {
        let mut sum = 0;
        for ref parent in self.parents.iter() {
            sum += parent.provide_size() + 1;
        }
        sum
    }
    fn serialise(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for ref parent in &self.parents {
            try!(serialise!(f, parent, "\n"));
        }
        Ok(())
    }
}
impl Readable for Parents<SHA1> {
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
impl Writable for Encoding {
    fn provide_size(&self) -> usize { self.raw.len() + 9 }
    fn serialise(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{}", self) }
}
impl Readable for Encoding {
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
impl Writable for Extras {
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
    fn serialise(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{}", self) }
}
impl Readable for Extras {
    fn nom_parse(b: &[u8]) -> nom::IResult<&[u8], Self> { nom_parse_extras(b) }
}

named!( nom_parse_commit<Commit<SHA1> >
      , chain!( tree_ref: call!(TreeRef::nom_parse) ~ char!('\n')
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
pub struct Commit<Hash: hash::Property> {
    pub tree_ref: TreeRef<Hash>,
    pub parents: Parents<Hash>,
    pub author: Author,
    pub committer: Committer,
    pub encoding: Option<Encoding>,
    pub extras: Extras,
    pub message: String
}

impl<Hash: hash::Property> Writable for Commit<Hash> {
    fn provide_size(&self) -> usize {
        0 + self.tree_ref.provide_size() + 1
          + self.parents.provide_size()
          + self.author.provide_size() + 1
          + self.committer.provide_size() + 1
          + match &self.encoding { &Some(ref e) => e.provide_size() + 1, &None => 0 }
          + self.extras.provide_size()
          + self.message.len()
    }
    fn serialise(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(serialise!( f
                       , self.tree_ref, "\n"
                       , self.parents
                       , self.author, "\n"
                       , self.committer, "\n"
                   ));
        if let &Some(ref e) = &self.encoding {
            try!(serialise!(f, e, "\n"));
        }
        serialise!( f
                  , self.extras, "\n"
                  , self.message
                  )
    }
}
impl Readable for Commit<SHA1> {
    fn nom_parse(b: &[u8]) -> nom::IResult<&[u8], Self> { nom_parse_commit(b) }
}
impl<Hash: hash::Property> Display for Commit<Hash> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "{}\n", self.tree_ref));
        try!(write!(f, "{}", self.parents));
        try!(write!(f, "{}\n", self.author));
        try!(write!(f, "{}\n", self.committer));
        if let &Some(ref e) = &self.encoding {
            try!(write!(f, "{}\n", e));
        }
        try!(write!(f, "{}\n", self.extras));
        write!(f, "{}", self.message)
    }
}

/* ------------------- Tree ------------------------------------------------ */

/// permission associated to a given type
///
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Permission {
    Read,
    Write,
    Executable
}

/// set of permissions associated to a given Tree entity
///
/// # TODO
///
/// This type is not stable yet. We will move to EnumSet when it
/// will be freezed and stable.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PermissionSet {
    set: BTreeSet<Permission>
}
impl PermissionSet {
    pub fn new() -> Self { PermissionSet { set: BTreeSet::new() } }
    fn new_from_byte(byte: u8) -> Self {
        let mut s : Self = Self::new();
        match byte {
            b'1' => s.set(Permission::Read),
            b'2' => s.set(Permission::Write),
            b'3' => {
                 s.set(Permission::Read);
                 s.set(Permission::Write)
            },
            b'4' => s.set(Permission::Executable),
            b'5' => {
                s.set(Permission::Executable);
                s.set(Permission::Read)
            },
            b'6' => {
                 s.set(Permission::Executable);
                 s.set(Permission::Write)
            },
            b'7' => {
                s.set(Permission::Executable);
                s.set(Permission::Read);
                s.set(Permission::Write)
            },
            _    => true
        };
        s
    }
    pub fn new_from(c: char) -> Self { Self::new_from_byte(c as u8) }
    pub fn to_char(&self) -> char {
        let mut c : u8 = b'0';
        if self.contains(&Permission::Read) { c = c + 1 }
        if self.contains(&Permission::Write) { c = c + 2 }
        if self.contains(&Permission::Executable) { c = c + 4 }
        c as char
    }
    pub fn set(&mut self, v: Permission) -> bool { self.set.insert(v) }
    pub fn contains(&self, v: &Permission) -> bool { self.set.contains(&v) }
}
#[inline(always)]
fn permission_write(ps: &PermissionSet) -> usize {
    let mut set = 0;
    if ps.contains(&Permission::Read) { set += 1 }
    if ps.contains(&Permission::Write) { set += 2 }
    if ps.contains(&Permission::Executable) { set += 4 }
    set
}
named!( nom_parse_permission_set<PermissionSet>
      , chain!(b : take!(1), || PermissionSet::new_from_byte(b[0]))
      );

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Permissions {
    // TODO: do we need to set the extras 3 bits as well?
    pub user: PermissionSet,
    pub group: PermissionSet,
    pub other: PermissionSet
}

impl Permissions {
    pub fn new() -> Self {
        Permissions {
            user: PermissionSet::new(),
            group: PermissionSet::new(),
            other: PermissionSet::new()
        }
    }
}

named!( nom_parse_permissions<Permissions>
      , chain!( tag!("0")
              ~ user: nom_parse_permission_set
              ~ group: nom_parse_permission_set
              ~ other: nom_parse_permission_set
              , || Permissions { user:user, group:group, other:other }
              )
      );
impl Writable for Permissions {
    fn serialise(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!( f
              , "{extras}{user}{group}{other}"
              , extras = 0
              , user = permission_write(&self.user)
              , group = permission_write(&self.group)
              , other = permission_write(&self.other)
              )
    }
    fn provide_size(&self) -> usize { 4 }
}
impl Readable for Permissions {
    fn nom_parse(b: &[u8]) -> nom::IResult<&[u8], Self> { nom_parse_permissions(b) }
}
impl Display for Permissions {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { self.serialise(f) }
}

/// the different type of entity managed by our current implementation
///
#[derive(Debug, Clone)]
pub enum TreeEnt {
    Dir(Permissions, PathBuf, HashRef<SHA1>),
    File(Permissions, PathBuf, HashRef<SHA1>)
    /*
    TODO: add missing:
    SymbolicLink(Permissions, PathBuf, HashRef<SHA1>),
    GitLink(Permissions, PathBuf, HashRef<SHA1>)
    */
}
impl Writable for TreeEnt {
    fn serialise(&self, f: &mut fmt::Formatter) -> fmt::Result {
        serialise!( f
                  , self.get_ent_type()
                  , self.get_premission()
                  , " "
                  , format!("{:?}", self.get_file_path())
                  , "\0"
                  , self.get_hash_ref().digest()
                  )
    }
    fn provide_size(&self) -> usize {
        8 + self.get_file_path().as_os_str().len()
    }
}
named!( nom_parse_path<PathBuf>
      , chain!( path_str: map_res!(take_until_and_consume!("\0"), str::from_utf8)
              , || PathBuf::new().join(path_str)
              )
      );
named!( nom_parse_tree_ent<TreeEnt>
      , chain!( t: map_res!( alt!( tag!("4") | tag!("10")) , str::from_utf8)
              ~ perm: call!(Permissions::nom_parse)
              ~ tag!(" ")
              ~ path: nom_parse_path
              ~ hash: take!(<hash::SHA1 as hash::Property>::DIGEST_SIZE)
              , || TreeEnt::new_from(t, perm, path, hash)
              )
      );
impl Readable for TreeEnt {
    fn nom_parse(b: &[u8]) -> nom::IResult<&[u8], Self> { nom_parse_tree_ent(b) }
}
impl Borrow<PathBuf> for TreeEnt {
    fn borrow(&self) -> &PathBuf { self.get_file_path() }
}
impl Borrow<HashRef<SHA1>> for TreeEnt {
    fn borrow(&self) -> &HashRef<SHA1> { self.get_hash_ref() }
}
impl Display for TreeEnt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{type_byte}{perms} {type} {hash} {name:?}"
              , type_byte = self.get_ent_type()
              , perms = self.get_premission()
              , type = self.get_ent_type_str()
              , hash = self.get_hash_ref()
              , name = self.get_file_path()
              )
    }
}
impl TreeEnt {
    fn new_from(ty: &str, perm: Permissions, path: PathBuf, hash_bytes: &[u8]) -> Self {
        if hash_bytes.len() != <hash::SHA1 as hash::Property>::DIGEST_SIZE { panic!("wrong size of Digest"); }
        let hash = hash::HashRef::new_with(hash_bytes);
        match ty {
            "10" => TreeEnt::Dir(perm, path, hash),
            "4" => TreeEnt::File(perm, path, hash),
            _ => panic!("unexpected type")
        }
    }
    pub fn get_ent_type_str(&self) -> &'static str {
        match self {
            &TreeEnt::Dir(_, _, _) => "tree",
            &TreeEnt::File(_, _, _) => "blob"
        }
    }
    fn get_ent_type(&self) -> &'static str {
        match self {
            &TreeEnt::Dir(_, _, _) => "10",
            &TreeEnt::File(_, _, _) => "4"
        }
    }
    pub fn get_premission(&self) -> &Permissions {
        match self {
            &TreeEnt::Dir(ref p, _, _) => p,
            &TreeEnt::File(ref p, _, _) => p
        }
    }
    pub fn get_hash_ref(&self) -> &HashRef<SHA1> {
        match self {
            &TreeEnt::Dir(_, _, ref pb) => pb,
            &TreeEnt::File(_, _, ref pb) => pb
        }
    }
    pub fn get_file_path(&self) -> &PathBuf {
        match self {
            &TreeEnt::Dir(_, ref pb, _) => pb,
            &TreeEnt::File(_, ref pb, _) => pb
        }
    }
}
impl PartialEq for TreeEnt {
    fn eq(&self, rhs: &TreeEnt) -> bool { self.get_file_path() == rhs.get_file_path() }
}
impl Eq for TreeEnt {}
impl PartialOrd for TreeEnt {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.get_file_path().partial_cmp(other.get_file_path())
    }
}
impl Ord for TreeEnt {
    fn cmp(&self, other: &Self) -> Ordering {
        self.get_file_path().cmp(other.get_file_path())
    }
}

/// a tree is a collection of Tree or Blob
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Tree {
    tree: BTreeSet<TreeEnt>
}
named!( nom_parse_tree_line<TreeEnt>
      , chain!(te: call!(TreeEnt::nom_parse), || te)
      );
named!( nom_parse_tree<Tree>
      , chain!( mut ts: value!(Tree::new())
              ~ many0!(chain!(te: nom_parse_tree_line, || ts.insert(te)))
              , || ts
              )
      );
impl Readable for Tree {
    fn nom_parse(b: &[u8]) -> nom::IResult<&[u8], Self> {
        nom_parse_tree(b)
    }
}
impl Display for Tree {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for ref tree_ent in &self.tree {
            try!(write!(f, "{}\n", tree_ent));
        }
        Ok(())
    }
}
impl Writable for Tree {
    fn serialise(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for ref tree_ent in &self.tree {
            try!(serialise!(f, tree_ent));
        }
        Ok(())
    }
    fn provide_size(&self) -> usize {
        let mut sum : usize = 0;
        for ref tree_ent in &self.tree {
            sum += tree_ent.provide_size() + 1;
        }
        sum
    }
}
impl Borrow<BTreeSet<TreeEnt>> for Tree {
    fn borrow(&self) -> &BTreeSet<TreeEnt> { &self.tree }
}
impl Tree {
    fn new_with(bt: BTreeSet<TreeEnt>) -> Self { Tree {tree: bt} }
    pub fn new() -> Self { Tree { tree: BTreeSet::new() }}
    pub fn iter(&self) -> btree_set::Iter<TreeEnt> { self.tree.iter() }
    pub fn difference<'a>(&'a self, other: &'a Tree) -> btree_set::Difference<'a, TreeEnt> {
        self.tree.difference(&other.tree)
    }
    pub fn symmetric_difference<'a>(&'a self, other: &'a Tree)
        -> btree_set::SymmetricDifference<'a, TreeEnt>
    {
        self.tree.symmetric_difference(&other.tree)
    }
    pub fn intersection<'a>(&'a self, other: &'a Tree) -> btree_set::Intersection<'a, TreeEnt> {
        self.tree.intersection(&other.tree)
    }
    pub fn union<'a>(&'a self, other: &'a Tree) -> btree_set::Union<'a, TreeEnt> {
        self.tree.union(&other.tree)
    }
    pub fn len(&self) -> usize { self.tree.len() }
    pub fn is_empty(&self) -> bool { self.tree.is_empty() }
    pub fn clear(&mut self) { self.tree.clear() }
    pub fn contains(&self, value: PathBuf) -> bool { self.tree.contains(&value) }
    pub fn get(&self, value: PathBuf) -> Option<&TreeEnt> { self.tree.get(&value) }
    pub fn is_disjoint(&self, other: &Tree) -> bool { self.tree.is_disjoint(&other.tree) }
    pub fn is_subset(&self, other: &Tree) -> bool { self.tree.is_subset(&other.tree) }
    pub fn is_superset(&self, other: &Tree) -> bool { self.tree.is_superset(&other.tree) }
    pub fn insert(&mut self, value: TreeEnt) -> bool { self.tree.insert(value) }
    pub fn replace(&mut self, value: TreeEnt) -> Option<TreeEnt> { self.tree.replace(value) }
    pub fn remove(&mut self, value: &PathBuf) -> bool { self.tree.remove(value) }
    pub fn take(&mut self, value: &PathBuf) -> Option<TreeEnt> { self.tree.take(value) }
}
impl FromIterator<TreeEnt> for Tree {
    fn from_iter<I: IntoIterator<Item=TreeEnt>>(iter: I) -> Tree {
        Tree::new_with(BTreeSet::from_iter(iter))
    }
}
impl IntoIterator for Tree {
    type Item = TreeEnt;
    type IntoIter = btree_set::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter { self.tree.into_iter() }
}
impl<'a> IntoIterator for &'a Tree {
    type Item = &'a TreeEnt;
    type IntoIter = btree_set::Iter<'a, TreeEnt>;
    fn into_iter(self) -> Self::IntoIter { self.tree.iter() }
}
impl Extend<TreeEnt> for Tree {
    fn extend<Iter: IntoIterator<Item=TreeEnt>>(&mut self, iter: Iter) {
        self.tree.extend(iter)
    }
}
impl<'a, 'b> Sub<&'b Tree> for &'a Tree {
    type Output = Tree;
    fn sub(self, rhs: &'b Tree) -> Tree { Tree::new_with(self.tree.sub(&rhs.tree)) }
}
impl<'a, 'b> BitXor<&'b Tree> for &'a Tree {
    type Output = Tree;
    fn bitxor(self, rhs: &'b Tree) -> Tree { Tree::new_with(self.tree.bitxor(&rhs.tree)) }
}
impl<'a, 'b> BitAnd<&'b Tree> for &'a Tree {
    type Output = Tree;
    fn bitand(self, rhs: &'b Tree) -> Tree { Tree::new_with(self.tree.bitand(&rhs.tree)) }
}
impl<'a, 'b> BitOr<&'b Tree> for &'a Tree {
    type Output = Tree;
    fn bitor(self, rhs: &'b Tree) -> Tree { Tree::new_with(self.tree.bitor(&rhs.tree)) }
}

/* ------------------- Object ---------------------------------------------- */

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ObjectType {
  Commit,
  Tree
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Object<Hash: hash::Property> {
    Commit(Commit<Hash>),
    Tree(Tree)
}
impl<Hash: hash::Property> Object<Hash> {
    pub fn to_string(&self) -> String {
        match self {
            &Object::Commit(ref c) => format!("{}", c),
            &Object::Tree(ref t)   => format!("{}", t)
        }
    }
}
impl Objectable for Object<SHA1> { }
impl<Hash: hash::Property> Display for Object<Hash> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { self.serialise(f) }
}
impl<Hash: hash::Property> Writable for Object<Hash> {
    fn provide_size(&self) -> usize {
        match self {
            &Object::Commit(ref c) => {
                let s = c.provide_size();
                s + 8 + s.provide_size()
            },
            &Object::Tree(ref t) => {
                let s = t.provide_size();
                s + 5 + s.provide_size()
            }
        }
    }
    fn serialise(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Object::Commit(ref c) => serialise!( f, "commit "
                                                , c.provide_size()
                                                , "\0"
                                                , c
                                                ),
            &Object::Tree(ref t) => serialise!( f, "tree "
                                              , t.provide_size()
                                              , "\0"
                                              , t
                                              )
        }
    }
}
named!( nom_parse_object_type_commit<ObjectType>
      , chain!( tag!("commit ") ~ many1!(nom::digit) ~ char!('\0')
              , || ObjectType::Commit
              )
      );
named!( nom_parse_object_type_tree<ObjectType>
      , chain!( tag!("tree ") ~ many1!(nom::digit) ~ char!('\0')
              , || ObjectType::Tree
              )
      );
named!( nom_parse_object_type<ObjectType>
      , alt!( nom_parse_object_type_commit
            | nom_parse_object_type_tree
            )
      );
named!( nom_parse_object_commit<Object<SHA1> >
      , chain!( c: call!(Commit::nom_parse)
              , || Object::Commit(c)
              )
      );
named!( nom_parse_object_tree<Object<SHA1> >
      , chain!( t: call!(Tree::nom_parse)
              , || Object::Tree(t)
              )
      );
impl Readable for Object<SHA1> {
    fn nom_parse(b: &[u8]) -> nom::IResult<&[u8], Self> {
        let (b2, ct) = try_parse!(b, nom_parse_object_type);
        match ct {
          ObjectType::Commit => nom_parse_object_commit(b2),
          ObjectType::Tree => nom_parse_object_tree(b2)
        }
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
