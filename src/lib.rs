#![feature(associated_consts)]
#![feature(test)]
extern crate test;

use std::path::*;
use std::fs::{File};
use std::io::Read;
use std::str::FromStr;
use std::collections::BTreeSet;

pub use hash::*;
pub use repo::Repo;
pub use object::*;
pub use error::*;
pub use refs::{Ref, SpecRef};

mod error;
pub mod hash;
pub mod repo;
pub mod refs;
pub mod object;

/// default structure used to contain some information regarding the git repository
/// some information such as the file path.
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct GitFS {
    path: PathBuf,
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
        match git.check_repo() {
            Err(err) => Err(err),
            Ok(_)    => Ok(git)
        }
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

    pub fn get_all_files_in<T: AsRef<Path>, P: AsRef<Path>>(&self, po: T, b: P)
        -> Result<BTreeSet<SpecRef>>
    {
        let parent_path = self.path.to_path_buf().join(po);
        let full_path = parent_path.clone().join(b);
        let mut btree = BTreeSet::new();
        let subdirs = match full_path.read_dir() {
            Err(err) => return Err(GitError::IoError(format!("{:?}", err))),
            Ok(l)    => l
        };
        for d_ in subdirs {
            if let Ok(d) = d_ {
                let p = d.path();
                let b = match p.strip_prefix(&parent_path) {
                    Err(err) => return Err(GitError::IoError(format!("{:?}", err))),
                    Ok(b)    => b
                };
                if p.exists() && p.is_file() {
                    btree.insert(SpecRef::branch(b));
                } else if p.exists() && p.is_dir() {
                    btree.extend(try!(self.get_all_files_in(po, b)))
                }
            }
        };
        Ok(btree)

    }

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
    match File::open(path) {
        Err(err) => return Err(GitError::IoError(format!("{}", err))),
        Ok(file) => Ok(file)
    }
}

impl Repo for GitFS {
    fn is_valid(&self) -> Result<()> { self.check_repo() }
    fn get_description(&self) -> Result<String> {
        let filepath = self.description_file();
        let mut file = try!(open_file(&filepath));
        let mut s = String::new();
        file.read_to_string(&mut s).unwrap();
        Ok(s)
    }
    fn get_ref(&self, r: &SpecRef) -> Result<Ref<SHA1>> {
        let filepath = self.path.to_path_buf().join(PathBuf::from(r));
        let mut file = try!(open_file(&filepath));
        let mut s = String::new();
        file.read_to_string(&mut s).unwrap();
        Ref::from_str(&s)
    }
    fn list_branches(&self) -> Result<BTreeSet<SpecRef>> {
        let dir = self.refs_dir().join("heads");
        let mut btree = BTreeSet::new();
        let subdirs = match dir.read_dir() {
            Err(err) => return Err(GitError::IoError(format!("{:?}", err))),
            Ok(l)    => l
        };
        for d_ in subdirs {
            if let Ok(d) = d_ {
                let p = d.path();
                let b = match p.strip_prefix(&dir) {
                    Err(err) => return Err(GitError::IoError(format!("{:?}", err))),
                    Ok(b)    => b
                };
                if p.exists() && p.is_file() {
                    // This is a branch
                    btree.insert(SpecRef::branch(b));
                } else if p.exists() && p.is_dir() {
                    // we are going to need to build the branch name exploring
                    // the tree
                    unimplemented!();
                }
            }
        };
        Ok(btree)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::*;

    #[test]
    fn new() {
        let path = PathBuf::new().join(".").join(".git");
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
        let path = PathBuf::new().join(".").join(".git");
        let git = GitFS::new(&path).unwrap();
        let desc = git.get_description();
        assert!(desc.is_ok())
    }

    #[test]
    fn git_fs_get_ref() {
        let path = PathBuf::new().join(".").join(".git");
        let git = GitFS::new(&path).unwrap();
        let r = SpecRef::branch("master");
        assert!(git.get_ref(&r).is_ok())
    }
    #[test]
    fn git_fs_get_ref_follow_link() {
        let path = PathBuf::new().join(".").join(".git");
        let git = GitFS::new(&path).unwrap();
        assert!(git.get_ref_follow_links(&SpecRef::Head).is_ok())
    }
    #[test]
    fn git_fs_get_head() {
        let path = PathBuf::new().join(".").join(".git");
        let git = GitFS::new(&path).unwrap();
        assert_eq!( git.get_head()
                  , Ok(Ref::Link(SpecRef::branch("master")))
                  )
    }
}
