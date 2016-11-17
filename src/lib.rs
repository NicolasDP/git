#![feature(associated_consts)]
#![feature(try_from)]
#![feature(test)]
extern crate test;

#[macro_use]
extern crate nom;

extern crate flate2;

use std::path::*;
use std::fs::{File};
use std::io::Read;
use std::str::FromStr;
use std::collections::BTreeSet;
use std::collections::VecDeque;
use flate2::read::ZlibDecoder;

pub use repo::Repo;
pub use object::*;
pub use error::*;
pub use refs::{Ref, SpecRef};

mod error;
pub mod hash;
pub mod repo;
pub mod refs;
pub mod object;

pub use object::elements::hash::{SHA1, Property, HashRef, HasHashRef};

/// default structure used to contain some information regarding the git repository
/// some information such as the file path.
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct GitFS {
    path: PathBuf,
}

fn append_dir_to_queue<P>(queue: &mut VecDeque<PathBuf>, path: P)
    -> Result<()>
    where P: AsRef<Path>
{
    path.as_ref().read_dir()
        .map_err(|err| GitError::IoError(format!("{:?}", err)))
        .map(|l| {
            l.fold(queue, |queue, d| {
                let _ = d.map_err(|err| GitError::IoError(format!("{:?}", err)))
                         .map(|dir| queue.push_back(dir.path()));
                queue
            });
        })
}

pub fn get_all_files_in<T>( parent_path: T
                          , make_specref: & Fn(&Path) -> Result<SpecRef>
                          )
    -> Result<BTreeSet<SpecRef>>
    where T: AsRef<Path>
{
    let mut queue = VecDeque::with_capacity(100);
    let mut btree = BTreeSet::new();
    let full_path = parent_path.as_ref();
    try!(append_dir_to_queue(&mut queue, &full_path));
    while let Some(dir) = queue.pop_front() {
        if dir.is_file() {
            let b = match dir.strip_prefix(&parent_path) {
                Err(err) => return Err(GitError::IoError(format!("{:?}", err))),
                Ok(b) => b
            };
            btree.insert(try!(make_specref(b)));
        } else if dir.is_dir() {
            try!(append_dir_to_queue(&mut queue, &dir));
        }
    }
    Ok(btree)
}

impl GitFS {
    /// create a Git struct from a given path
    ///
    /// Will fail if the given directory does not look like a valid git
    /// directory.
    ///
    /// # Example
    ///
    /// ```
    /// use std::path::*;
    /// use git::GitFS;
    /// let git_path = PathBuf::new().join("path").join("to").join("nowhere");
    /// match GitFS::new(&git_path) {
    ///   Ok(_)    => { /* I have a valid git repository */ },
    ///   Err(_) => { /* invalid path or invaild git repository */ }
    /// }
    /// ```
    pub fn new(p: &Path) -> Result<Self> {
        let git = GitFS { path: p.to_path_buf() };
        git.check_repo().map(move |_| git)
    }

    /// return the refs directory (where all the link to the branches and tags are)
    ///
    /// # Example
    ///
    /// ```
    /// use std::path::*;
    /// use git::GitFS;
    /// let git_path = PathBuf::new().join(".").join(".git");
    /// let git = GitFS::new(&git_path).unwrap();
    /// let refs_dir = git.refs_dir();
    /// assert_eq!(refs_dir, PathBuf::new().join(&git_path).join("refs"))
    /// ```
    pub fn refs_dir(&self)  -> PathBuf { self.path.to_path_buf().join("refs") }
    /// return the objects path (contain all the blob and packed git objects)
    pub fn objs_dir(&self)  -> PathBuf { self.path.to_path_buf().join("objects") }
    /// return the info directory path
    pub fn info_dir(&self)  -> PathBuf { self.path.to_path_buf().join("info") }
    /// return the hooks directory path
    pub fn hooks_dir(&self) -> PathBuf { self.path.to_path_buf().join("hooks") }
    /// return the git config file path
    pub fn config_file(&self)      -> PathBuf { self.path.to_path_buf().join("config") }
    /// return the git description file path
    pub fn description_file(&self) -> PathBuf { self.path.to_path_buf().join("description") }
    /// return the git current HEAD file path
    pub fn head_file(&self)        -> PathBuf { self.path.to_path_buf().join("HEAD") }


    fn check_repo(&self) -> Result<()> {
        let dirs = [ self.refs_dir()
                   , self.objs_dir()
                   , self.info_dir()
                   , self.hooks_dir()
                   ];
        let files = [ self.config_file()
                    , self.description_file()
                    , self.head_file()
                    ];
        for dir in dirs.iter() {
            if ! dir.exists() || ! dir.is_dir() {
                return Err(GitError::MissingDirectory(dir.clone()))
            }
        };
        for file in files.iter() {
            if ! file.exists() || ! file.is_file() {
                return Err(GitError::MissingFile(file.clone()))
            }
        };
        Ok(())
    }
}

fn open_file(path: &PathBuf) -> Result<File> {
    File::open(path)
        .map_err(|err| GitError::IoError(format!("{}", err)))
}

impl Repo for GitFS {
    fn is_valid(&self) -> Result<()> { self.check_repo() }
    fn get_description(&self) -> Result<String> {
        let filepath = self.description_file();
        let mut file = try!(open_file(&filepath));
        let mut s = String::new();
        file.read_to_string(&mut s)
            .map_err(|err| GitError::IoError(format!("{}", err)))
            .map(move |_| s)
    }
    fn get_ref(&self, r: SpecRef) -> Result<Ref<SHA1>> {
        let filepath = self.path.to_path_buf().join(PathBuf::from(r));
        let mut file = try!(open_file(&filepath));
        let mut s = String::new();
        file.read_to_string(&mut s)
            .map_err(|err| GitError::IoError(format!("{}", err)))
            .and_then(|_| Ref::from_str(&s))
    }

    fn get_object<T: HasHashRef<SHA1>>(&self, hhr: &T) -> Result<Object<SHA1>> {
        let r = hhr.hash_ref();
        let path = self.objs_dir().join(r.path());
        println!("{:?}", &path);
        if ! path.is_file() {
            return Err(GitError::InvalidRef(path))
        }
        let file = try!(open_file(&path));
        let mut zlibr = ZlibDecoder::new(file);
        let mut s = Vec::new();
        zlibr.read_to_end(&mut s)
             .map_err(|err| GitError::IoError(format!("{:?}", err)))
             .and_then(|_| {
                 Object::parse_bytes(s.as_ref())
             })
    }

    fn list_branches(&self) -> Result<BTreeSet<SpecRef>> {
        get_all_files_in( self.refs_dir().join("heads")
                        , &|x| Ok(SpecRef::branch(x))
                        )
    }
    fn list_remotes(&self) -> Result<BTreeSet<SpecRef>> {
        get_all_files_in( self.refs_dir().join("remotes")
                        , &|remote_path| {
            let mut components = remote_path.components();
            components
                .next()
                .map_or(Err(GitError::InvalidRemote(remote_path.to_path_buf())), |p| {
                    match p {
                        Component::Normal(remote) => Ok(SpecRef::remote(remote, components.as_path())),
                        _ => Err(GitError::Unknown("invalid remote name".to_string()))
                    }
                })
            }
        )
    }
    fn list_tags(&self) -> Result<BTreeSet<SpecRef>> {
        get_all_files_in( self.refs_dir().join("tags")
                        , &|x| Ok(SpecRef::tag(x))
                        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::*;

    fn get_test_commit() -> Ref<SHA1> {
        Ref::Link(SpecRef::branch("master"))
    }

    fn get_root_test() -> PathBuf {
        PathBuf::new().join(".").join("test_ref").join(".git")
    }

    #[test]
    fn new() {
        let path = get_root_test();
        assert_eq!(GitFS::new(&path), Ok(GitFS { path: path.clone()}))
    }
    #[test]
    fn new_fail() {
        let path = PathBuf::new().join(".").join("src");
        let missing = PathBuf::new().join(".").join("src").join("refs");
        assert_eq!(GitFS::new(&path), Err(GitError::MissingDirectory(missing)))
    }

    #[test]
    fn git_fs_get_description() {
        let path = get_root_test();
        let git = GitFS::new(&path).unwrap();
        let desc = git.get_description();
        assert!(desc.is_ok())
    }

    #[test]
    fn git_fs_get_ref() {
        let path = get_root_test();
        let git = GitFS::new(&path).unwrap();
        let r = SpecRef::branch("master");
        assert!(git.get_ref(r).is_ok())
    }
    #[test]
    fn git_fs_get_ref_follow_link() {
        let path = get_root_test();
        let git = GitFS::new(&path).unwrap();
        assert!(git.get_ref_follow_links(SpecRef::Head).is_ok())
    }
    #[test]
    fn git_fs_get_head() {
        let path = get_root_test();
        let git = GitFS::new(&path).unwrap();
        assert_eq!( git.get_head()
                  , Ok(Ref::Link(SpecRef::branch("master")))
                  )
    }
    #[test]
    fn git_fs_get_branches() {
        let path = get_root_test();
        let git = GitFS::new(&path).unwrap();
        let branches = git.list_branches().expect("expect to list the branches");
        assert!(branches.contains(&SpecRef::branch("master")));
    }
    #[test]
    fn git_fs_get_remotes() {
        let path = get_root_test();
        let git = GitFS::new(&path).unwrap();
        let branches = git.list_remotes().expect("expect to list the remotes");
        println!("{:?}", branches);
        assert!( branches.contains(&SpecRef::remote("origin", "HEAD"))
                 || branches.contains(&SpecRef::remote("origin", "master")));
    }
    #[test]
    fn git_fs_get_commit() {
        let path = get_root_test();
        let git = GitFS::new(&path).unwrap();
        let commit = git.get_object_ref(get_test_commit()).unwrap();
        let parse_str = format!("{}", commit);
        let commit2 = Object::parse_bytes(parse_str.as_bytes()).unwrap();
        assert_eq!(commit, commit2);
    }

    #[test]
    fn git_fs_get_tree() {
        let path = get_root_test();
        let git = GitFS::new(&path).unwrap();
        // get the head
        let commit =
            git.get_object_ref(get_test_commit())
               .map(|o| match o {
                   Object::Commit(c) => c,
                   _ => panic!("This is not a tree...")
               }).expect("expected to read a commit object");

        let tree = git.get_object(&commit.tree_ref)
                      .expect("expected to read a Tree object");
        println!("++ Tree ++++++++++++++++++++++++++");
        println!("{}", tree.to_string())
    }
}
