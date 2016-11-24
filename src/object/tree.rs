use protocol::hash::Hasher;
use protocol::encoder::Encoder;
use protocol::decoder::Decoder;
use super::blob::BlobRef;
use std::{io, fmt, str, collections, path, cmp, borrow, iter, ops, convert};
use nom;

/// Tree reference
///
/// This is simply a wrapper on top of the given type `H` implementing
/// `git::protocol::hash::Hasher`.
///
/// All the implementation is redirected to the Hasher traits
///
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
pub struct TreeRef<H: Hasher>(H);
impl<H: Hasher> TreeRef<H> {
    /// simply create a TreeRef from a given `Hasher`, taking ownership
    pub fn new(h: H) -> Self { TreeRef(h) }
}
impl<H: Hasher + fmt::Display> fmt::Display for TreeRef<H> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { fmt::Display::fmt(&self.0, f) }
}
impl<H: Hasher> Hasher for TreeRef<H> {
    fn hash<R: io::BufRead>(data: &mut R) -> io::Result<Self> {
        H::hash(data).map(|h| TreeRef(h))
    }

    fn from_bytes(v: Vec<u8>) -> Option<Self> {
        H::from_bytes(v).map(|h| TreeRef(h))
    }

    #[inline]
    fn digest_size() -> usize { H::digest_size() }

    #[inline]
    fn as_bytes(&self) -> &[u8] { self.0.as_bytes() }
}
impl<H: Hasher> convert::AsRef<H> for TreeRef<H> {
    fn as_ref(&self) -> &H { &self.0 }

}

/// Tree Permission
///
/// * Read: grant read access;
/// * Write: grant modification access;
/// * Executable: grant executable access.
///
/// By itself the type is not really meanining full, you need to
/// know the group it is associated to. See `PermissionSet` and `Permissions`
///
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Permission {Read, Write, Executable}

/// set of permissions
///
/// This type will allow us to compose `Permission` together:
///
/// * Read only
/// * Read + Write
/// * ...
///
/// This type is not completely meaningful by its own. See `Permissions`
/// to know to which group the `PermissionSet` applies.
///
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PermissionSet {
    set: collections::BTreeSet<Permission>
}
impl PermissionSet {
    /// create a new, empty persmissions
    pub fn new() -> Self { PermissionSet { set: collections::BTreeSet::new() } }
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
    /// create a new persmission form the given octal char
    ///
    /// The operation will create an empty PermissionSet if invalid octal
    pub fn new_from(c: char) -> Self { Self::new_from_byte(c as u8) }

    /// get the octal value
    pub fn to_char(&self) -> char {
        let mut c : u8 = b'0';
        if self.contains(&Permission::Read) { c = c + 1 }
        if self.contains(&Permission::Write) { c = c + 2 }
        if self.contains(&Permission::Executable) { c = c + 4 }
        c as char
    }

    /// set a permission to the set
    pub fn set(&mut self, v: Permission) -> bool { self.set.insert(v) }
    /// check the set contains the given permission
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

/// Permissions for a given entity
///
/// Configuration of the Permissions per group of users:
///
/// * user:  the set of `Permission` only applies to the user;
/// * group: the set of `Permission` applies to the group;
/// * other: the set of `Permission` applies to the other.
///
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Permissions {
    // TODO: do we need to set the extras 3 bits as well?
    pub user: PermissionSet,
    pub group: PermissionSet,
    pub other: PermissionSet
}

impl Permissions {
    /// new empty Permissions
    pub fn new() -> Self {
        Permissions {
            user: PermissionSet::new(),
            group: PermissionSet::new(),
            other: PermissionSet::new()
        }
    }
}
named!( tree_ent_parse_permissions<Permissions>
      , chain!( tag!("0")
              ~ user: nom_parse_permission_set
              ~ group: nom_parse_permission_set
              ~ other: nom_parse_permission_set
              , || Permissions { user:user, group:group, other:other }
              )
      );
impl fmt::Display for Permissions {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{extras}{user}{group}{other}"
              , extras = 0
              , user = permission_write(&self.user)
              , group = permission_write(&self.group)
              , other = permission_write(&self.other)
              )
    }
}

/// the different type of entity managed by our current implementation
///
/// * Tree: reference with a permission to a sub tree (recursive entry).
///         This is equivalent to a filepath directory.
/// * Blob: reference with a permission to a blob of data.
///         This is equivalent to a file.
///
#[derive(Debug, Clone)]
pub enum TreeEnt<H: Hasher> {
    Tree(Permissions, path::PathBuf, TreeRef<H>),
    Blob(Permissions, path::PathBuf, BlobRef<H>)
    /*
    TODO: add missing:
    SymbolicLink(Permissions, PathBuf, HashRef<SHA1>),
    GitLink(Permissions, PathBuf, HashRef<SHA1>)
    */
}
impl<H: Hasher> TreeEnt<H> {
    fn get_file_path(&self) -> &path::PathBuf {
        match self {
            &TreeEnt::Tree(_, ref pb, _) => pb,
            &TreeEnt::Blob(_, ref pb, _) => pb
        }
    }
    fn get_ent_type_str(&self) -> &'static str {
        match self {
            &TreeEnt::Tree(_, _, _) => "tree",
            &TreeEnt::Blob(_, _, _) => "blob"
        }
    }
    fn get_ent_type(&self) -> &'static str {
        match self {
            &TreeEnt::Tree(_, _, _) => "10",
            &TreeEnt::Blob(_, _, _) => "4"
        }
    }
    fn get_premission(&self) -> &Permissions {
        match self {
            &TreeEnt::Tree(ref p, _, _) => p,
            &TreeEnt::Blob(ref p, _, _) => p
        }
    }
    fn get_hash_hex(&self) -> String { self.get_hash().to_hexadecimal() }
    fn get_hash(&self) -> &H {
        match self {
            &TreeEnt::Tree(_, _, ref pb) => pb.as_ref(),
            &TreeEnt::Blob(_, _, ref pb) => pb.as_ref()
        }
    }
    fn new_from(ty: &str, perm: Permissions, path: path::PathBuf, h: H) -> Self {
        match ty {
            "10" => TreeEnt::Tree(perm, path, TreeRef::new(h)),
            "4" => TreeEnt::Blob(perm, path, BlobRef::new(h)),
            _ => panic!("unexpected type")
        }
    }
}
impl<H: Hasher> fmt::Display for TreeEnt<H> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{type_byte}{perms} {type} {hash} {name:?}"
              , type_byte = self.get_ent_type()
              , perms = self.get_premission()
              , type = self.get_ent_type_str()
              , hash = self.get_hash_hex()
              , name = self.get_file_path()
              )
    }
}
impl<H: Hasher> PartialEq for TreeEnt<H> {
    fn eq(&self, rhs: &Self) -> bool { self.get_file_path() == rhs.get_file_path() }
}
impl<H: Hasher> Eq for TreeEnt<H> {}
impl<H: Hasher> PartialOrd for TreeEnt<H> {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        self.get_file_path().partial_cmp(other.get_file_path())
    }
}
impl<H: Hasher> Ord for TreeEnt<H> {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.get_file_path().cmp(other.get_file_path())
    }
}
impl<H: Hasher> borrow::Borrow<path::PathBuf> for TreeEnt<H> {
    fn borrow(&self) -> &path::PathBuf { self.get_file_path() }
}
impl<H: Hasher> Decoder for TreeEnt<H> {
    fn decode(b: &[u8]) -> nom::IResult<&[u8], Self> {
        let (i, (ty, perm, p)) = match nom_parse_tree_ent_head(b) {
            nom::IResult::Done(i, b) => (i, b),
            nom::IResult::Error(err) => return nom::IResult::Error(err),
            nom::IResult::Incomplete(n) => return nom::IResult::Incomplete(n)
        };
        let (i, h) = match H::decode_bytes(i) {
            nom::IResult::Done(i, b) => (i, b),
            nom::IResult::Error(err) => return nom::IResult::Error(err),
            nom::IResult::Incomplete(n) => return nom::IResult::Incomplete(n)
        };
        nom::IResult::Done(i, TreeEnt::new_from(ty, perm, p, h))
    }
}
impl<H: Hasher> Encoder for TreeEnt<H> {
    fn encode<W: io::Write>(&self, writer: &mut W) -> io::Result<usize> {
        let data = format!( "{} {} {:?}\0"
                          , self.get_ent_type()
                          , self.get_premission()
                          , self.get_file_path()
                          );
        try!(writer.write_all(data.as_bytes()));
        let sz = try!(self.get_hash().encode_bytes(writer));
        Ok(sz + data.len())
    }
}
named!( nom_parse_path<path::PathBuf>
      , chain!( path_str: map_res!(take_until_and_consume!("\0"), str::from_utf8)
              , || path::PathBuf::new().join(path_str)
              )
      );
named!( nom_parse_tree_ent_head<(&str, Permissions, path::PathBuf)>
      , chain!( t: map_res!( alt!( tag!("4") | tag!("10")) , str::from_utf8)
              ~ perm: tree_ent_parse_permissions
              ~ tag!(" ")
              ~ path: nom_parse_path
              , || (t, perm, path)
              )
      );

///
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
pub struct Tree<H: Hasher>(collections::BTreeSet<TreeEnt<H>>);
impl<H: Hasher> Tree<H> {
    pub fn new_with(bt: collections::BTreeSet<TreeEnt<H>>) -> Self { Tree(bt) }
    pub fn new() -> Self { Tree(collections::BTreeSet::new()) }
    pub fn iter(&self) -> collections::btree_set::Iter<TreeEnt<H>> { self.0.iter() }
    pub fn difference<'a>(&'a self, other: &'a Self)
        -> collections::btree_set::Difference<'a, TreeEnt<H>>
    {
        self.0.difference(&other.0)
    }
    pub fn symmetric_difference<'a>(&'a self, other: &'a Self)
        -> collections::btree_set::SymmetricDifference<'a, TreeEnt<H>>
    {
        self.0.symmetric_difference(&other.0)
    }
    pub fn intersection<'a>(&'a self, other: &'a Self)
        -> collections::btree_set::Intersection<'a, TreeEnt<H>>
    {
        self.0.intersection(&other.0)
    }
    pub fn union<'a>(&'a self, other: &'a Self)
        -> collections::btree_set::Union<'a, TreeEnt<H>>
    {
        self.0.union(&other.0)
    }
    pub fn len(&self) -> usize { self.0.len() }
    pub fn is_empty(&self) -> bool { self.0.is_empty() }
    pub fn clear(&mut self) { self.0.clear() }
    pub fn contains(&self, value: path::PathBuf) -> bool { self.0.contains(&value) }
    pub fn get(&self, value: path::PathBuf) -> Option<&TreeEnt<H>> { self.0.get(&value) }
    pub fn is_disjoint(&self, other: &Self) -> bool { self.0.is_disjoint(&other.0) }
    pub fn is_subset(&self, other: &Self) -> bool { self.0.is_subset(&other.0) }
    pub fn is_superset(&self, other: &Self) -> bool { self.0.is_superset(&other.0) }
    pub fn insert(&mut self, value: TreeEnt<H>) -> bool { self.0.insert(value) }
    pub fn replace(&mut self, value: TreeEnt<H>) -> Option<TreeEnt<H>> { self.0.replace(value) }
    pub fn remove(&mut self, value: &path::PathBuf) -> bool { self.0.remove(value) }
    pub fn take(&mut self, value: &path::PathBuf) -> Option<TreeEnt<H>> { self.0.take(value) }
}
impl<H: Hasher> iter::FromIterator<TreeEnt<H>> for Tree<H> {
    fn from_iter<I: IntoIterator<Item=TreeEnt<H>>>(iter: I) -> Self {
        Tree::new_with(collections::BTreeSet::from_iter(iter))
    }
}
impl<H: Hasher> IntoIterator for Tree<H> {
    type Item = TreeEnt<H>;
    type IntoIter = collections::btree_set::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter { self.0.into_iter() }
}
impl<'a, H: Hasher> IntoIterator for &'a Tree<H> {
    type Item = &'a TreeEnt<H>;
    type IntoIter = collections::btree_set::Iter<'a, TreeEnt<H>>;
    fn into_iter(self) -> Self::IntoIter { self.0.iter() }
}
impl<H: Hasher> Extend<TreeEnt<H>> for Tree<H> {
    fn extend<Iter: IntoIterator<Item=TreeEnt<H>>>(&mut self, iter: Iter) {
        self.0.extend(iter)
    }
}
impl<'a, 'b, H:Hasher+Clone> ops::Sub<&'b Tree<H>> for &'a Tree<H> {
    type Output = Tree<H>;
    fn sub(self, rhs: &'b Tree<H>) -> Tree<H> { Tree::new_with(self.0.sub(&rhs.0)) }
}
impl<'a, 'b, H: Hasher+Clone> ops::BitXor<&'b Tree<H>> for &'a Tree<H> {
    type Output = Tree<H>;
    fn bitxor(self, rhs: &'b Tree<H>) -> Tree<H> { Tree::new_with(self.0.bitxor(&rhs.0)) }
}
impl<'a, 'b, H: Hasher+Clone> ops::BitAnd<&'b Tree<H>> for &'a Tree<H> {
    type Output = Tree<H>;
    fn bitand(self, rhs: &'b Tree<H>) -> Tree<H> { Tree::new_with(self.0.bitand(&rhs.0)) }
}
impl<'a, 'b, H: Hasher+Clone> ops::BitOr<&'b Tree<H>> for &'a Tree<H> {
    type Output = Tree<H>;
    fn bitor(self, rhs: &'b Tree<H>) -> Tree<H> { Tree::new_with(self.0.bitor(&rhs.0)) }
}
impl<H: Hasher> Decoder for Tree<H> {
    fn decode(b: &[u8]) -> nom::IResult<&[u8], Self> { nom_parse_tree(b) }
}

named!(nom_parse_tree_tag, tag!("tree "));
named!(nom_parse_tree_size<usize>
      , map_res!( map_res!( nom::digit, str::from_utf8), str::FromStr::from_str)
      );
named!(nom_parse_tree_head<usize>
      , chain!(nom_parse_tree_tag ~ r: nom_parse_tree_size ~ char!('\0'), || r)
      );
fn nom_parse_tree<H: Hasher>(b: &[u8]) -> nom::IResult<&[u8], Tree<H>> {
    let (mut b, size) = match nom_parse_tree_head(b) {
        nom::IResult::Done(b, o) => (b, o),
        nom::IResult::Error(err) => return nom::IResult::Error(err),
        nom::IResult::Incomplete(n) => return nom::IResult::Incomplete(n)
    };
    if b.len() < size {
        return nom::IResult::Incomplete(nom::Needed::Size(size - b.len()));
    }
    let mut tree = Tree::new();
    while let nom::IResult::Done(i, te) = TreeEnt::<H>::decode(b) {
        tree.insert(te);
        b = i;
    }
    nom::IResult::Done(b, tree)
}

// -- --------------------------------------------------------------------- --
// --                                 Tests                                 --
// -- --------------------------------------------------------------------- --

#[cfg(test)]
mod test {
    #[test]
    fn tree_serialisable() {
    }
}
