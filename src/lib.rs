/*! Git library in pure rust

[![Build Status](https://travis-ci.org/NicolasDP/git.png?branch=master)](https://travis-ci.org/NicolasDP/git)
*/

#[cfg(test)]
extern crate rustc_serialize;

#[macro_use]
extern crate nom;

pub mod protocol;
pub mod object;
pub mod error;
pub mod refs;
pub mod fs;

/*
pub use object::elements::hash::{SHA1, Property, HashRef, HasHashRef};

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
*/
