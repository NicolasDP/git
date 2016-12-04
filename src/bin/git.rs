extern crate git;
extern crate clap;
use clap::{Arg, App, SubCommand};
use std::path::Path;
use std::str::FromStr;
use git::refs::SpecRef;
use git::protocol::{SHA1, Hash, Repo};

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
            ).subcommand(SubCommand::with_name("branch")
                .about("show branches")
                .arg(Arg::with_name("all-branch").long("all").short("a").help("show all branches"))
            ).subcommand(SubCommand::with_name("log")
                .about("list commits")
                .arg(
                    Arg::with_name("REF")
                        .help("the hash to print")
                        .required(true)
                        .index(1)
                )
            ).get_matches();

    if let Some(matches) = matches.subcommand_matches("cat-file") {
        cat_file(matches);
    } else if let Some(matches) = matches.subcommand_matches("branch") {
        branch(matches);
    } else if let Some(matches) = matches.subcommand_matches("log") {
        log(matches);
    }
}

fn cat_file(matches: &clap::ArgMatches) {
    let r = matches.value_of("REF").expect("reference to git Object");

    let git = git::fs::GitFS::new(Path::new(".git")).expect("valid git repository");
    let hash = match SHA1::from_hex(r.clone()) {
        Some(e) => e,
        None    => git.get_ref_follow_links(
            SpecRef::from_str(r.clone()).unwrap()
        ).unwrap()
    };


    print!("{}", git.get_object_(hash).unwrap());
}

fn branch(matches: &clap::ArgMatches) {
    let git = git::fs::GitFS::new(Path::new(".git")).expect("valid git repository");
    let mut branches = git.list_branches().unwrap();
    if matches.is_present("all-branch") {
        branches.append(
            &mut git.list_remotes().unwrap()
        );
        branches.append(
            &mut git.list_tags().unwrap()
        );
    }
    for branch in branches.iter() {
        println!("{}", branch);
    }
}

fn log(matches: &clap::ArgMatches) {
    let r = matches.value_of("REF").unwrap_or("HEAD");

    let git = git::fs::GitFS::new(Path::new(".git")).expect("valid git repository");
    let hash = match SHA1::from_hex(r.clone()) {
        Some(e) => e,
        None    => git.get_ref_follow_links(
            SpecRef::from_str(r.clone()).unwrap()
        ).unwrap()
    };
    let mut cmhash = CommitRef::new(hash);

    while let Ok(commit) = git.get_commit(cmhash) {
        print!("{}", commit);
        cmhash = commit.parents.first().unwrap().clone();
    }
}
