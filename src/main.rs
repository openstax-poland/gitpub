#![feature(map_first_last)]

use anyhow::Result;
use argh::FromArgs;
use git2::{ObjectType, Repository, Signature};
use std::{fmt, process::Command};

use crate::util::OutputEx;

mod engine;
mod package;
mod util;

/// Publish package as a GIT tag
#[derive(FromArgs)]
struct Args {
    /// select engine
    #[argh(option)]
    engine: Option<String>,
}

fn main() -> Result<()> {
    let args: Args = argh::from_env();

    let mut engine = match args.engine {
        Some(ref name) => engine::by_name(name)?,
        None => engine::select()?,
    };

    let repo = Repository::open_from_env()?;

    println!("{}       Using{} {}", S, R, engine.name());
    let name = engine.pkg_name();
    let version = engine.pkg_version();
    println!("{}   Packaging{} {} {}", S, R, name, version);

    println!("{}   Preparing{}", S, R);
    engine.prepare()?;

    println!("{}     Packing{}", S, R);
    let mut package = package::Package::new(&repo)?;
    engine.pack(&mut package)?;
    let tree = package.finish()?;

    println!("{}  Committing{} release", S, R);
    let author = repo.signature()?;
    let committer = Signature::now("gitpub", "gitpub")?;
    let message = format!("Publish {} {}", name, version);
    let tag_name = format!("gitpub/{}@{}", name, version);
    let tree = repo.find_tree(tree)?;
    let commit = repo.commit(None, &author, &committer, &message, &tree, &[])?;
    let commit = repo.find_object(commit, Some(ObjectType::Commit))?;
    repo.tag(&tag_name, &commit, &committer, &message, false)?;

    println!("{}   Uploading{} {}", S, R, tag_name);
    Command::new("git").args(&["push", "origin", &tag_name]).output()?.exit_on_fail()?;
    repo.tag_delete(&tag_name)?;

    Ok(())
}

struct S;
struct R;

impl fmt::Display for S {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", termion::color::LightGreen.fg_str(), termion::style::Bold)
    }
}

impl fmt::Display for R {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", termion::color::Reset.fg_str(), termion::style::Reset)
    }
}
