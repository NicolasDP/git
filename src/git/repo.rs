use std::path::*;
use std::result;

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum GitError {
    MissingDirectory(PathBuf),
    MissingFile(PathBuf),
    NotValidRepo
}

pub type Result<T> = result::Result<T, GitError>;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Git {
    path: PathBuf,
}

impl Git {
    pub fn new(p: &Path) -> Result<Self> {
        let mut git = Git { path: PathBuf::new() };
        git.path.push(p);
        match git.check_repo() {
            Err(err) => Err(err),
            Ok(_)    => Ok(git)
        }
    }

    fn check_repo(&self) -> Result<()> {
        let dirs = ["refs", "info", "objects", "hooks"];
        let files = ["config", "description", "HEAD"];
        for c in dirs.iter() {
            let mut dir = PathBuf::new();
            dir.push(&self.path);
            dir.push(c);
            if ! dir.exists() || ! dir.is_dir() {
                return Err(GitError::MissingDirectory(dir.clone()))
            }
        };
        for c in files.iter() {
            let mut f = PathBuf::new();
            f.push(&self.path);
            f.push(c);
            if ! f.exists() || ! f.is_file() {
                return Err(GitError::MissingFile(f.clone()))
            }
        };
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use ::git::repo::*;
    use std::path::*;

    #[test]
    fn new() {
        let mut path = PathBuf::new();
        path.push(".");
        path.push(".git");
        assert_eq!(Git::new(&path), Ok(Git { path: path.clone()}))
    }
    #[test]
    fn new_fail() {
        let mut path = PathBuf::new();
        let mut missing = PathBuf::new();
        path.push(".");
        path.push("src");
        missing.push(".");
        missing.push("src");
        missing.push("refs");
        assert_eq!(Git::new(&path), Err(GitError::MissingDirectory(missing)))
    }
}
