extern crate git;
extern crate clap;
use clap::{Arg, App, SubCommand};
use std::path::Path;
use git::object::{CommitRef, Commit, TreeRef, Tree};
use git::protocol::{SHA1, Hash, Repo};
use git::error::Result;

fn main() {
    let matches = App::new("git")
            .version("0.0.1")
            .about("Does awesome things with git")
            .subcommand(SubCommand::with_name("cat-file")
                .about("print information from a given ref/hash")
                .arg(
                    Arg::with_name("pretty").short("p").help("pretty print the content")
                )
                .arg(
                    Arg::with_name("REF")
                        .help("the hash to print")
                        .required(true)
                        .index(1)
                )
            ).get_matches();

    if let Some(matches) = matches.subcommand_matches("cat-file") {
        let r = matches.value_of("REF").unwrap();

        let git = git::fs::GitFS::new(Path::new(".git")).unwrap();
        let hash = SHA1::from_hex(r).unwrap();

        let cmt = CommitRef::new(hash.clone());
        let mcommit : Result<Commit<SHA1>> = git.get_object(cmt);
        if let Ok(commit) = mcommit {
            println!("{}", commit);
        } else {
            let cmt = TreeRef::new(hash.clone());
            let mtree : Result<Tree<SHA1>> = git.get_object(cmt);
            if let Ok(tree) = mtree {
                println!("{}", tree);
            }
        }
    }
}
