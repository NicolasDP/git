use std::path::*;
use std::fs::{File};
use std::collections::VecDeque;

use refs::{SpecRef};
use error::{Result, GitError};

/// convenient function to open a file or wrap up the error into a
/// the Git Error.
pub fn open_file(path: &PathBuf) -> Result<File> {
    File::open(path)
        .map_err(|err| GitError::ioerror(err))
}

pub fn append_dir_to_queue<P>(queue: &mut VecDeque<PathBuf>, path: P)
    -> Result<()>
    where P: AsRef<Path>
{
    path.as_ref().read_dir()
        .map_err(|err| GitError::ioerror(err))
        .map(|l| {
            l.fold(queue, |queue, d| {
                // TODO: the error is ignored... this is not what we want
                // we need to propagate the error if something wrong happened.
                let _ = d.map_err(|err| GitError::ioerror(err))
                         .map(|dir| queue.push_back(dir.path()));
                queue
            });
        })
}

/// helper to list all files present in a directories and its subdirectories
pub fn get_all_files_in<T, P>( parent_path: T
                             , make_specref: & Fn(&Path) -> Result<Option<P>>
                             )
    -> Result<Vec<P>>
    where T: AsRef<Path>
{
    let mut queue = VecDeque::with_capacity(100);
    let mut array = Vec::new();
    let full_path = parent_path.as_ref();
    try!(append_dir_to_queue(&mut queue, &full_path));
    while let Some(dir) = queue.pop_front() {
        if dir.is_file() {
            let b = match dir.strip_prefix(&parent_path) {
                Err(err) => return Err(GitError::Other(format!("{:?}", err))),
                Ok(b) => b
            };
            if let Some(data) = try!(make_specref(b)) {
                array.push(data);
            }
        } else if dir.is_dir() {
            try!(append_dir_to_queue(&mut queue, &dir));
        }
    }
    Ok(array)
}
