use anyhow::Result;
use argh::FromArgs;
use git2::{ObjectType, Repository, Signature};
use std::{fmt, process::Command};

use crate::util::CommandEx;

mod engine;
mod output;
mod package;
mod util;

/// Publish package as a GIT tag
#[derive(FromArgs)]
struct Args {
    /// select engine
    #[argh(option)]
    engine: Option<String>,
    /// do not push tag to repository
    #[argh(switch)]
    no_publish: bool,
    /// keep tag after publishing
    #[argh(switch)]
    keep_tag: bool,
    /// show verbose output
    #[argh(switch, short = 'v')]
    verbose: bool,
}

fn main() -> Result<()> {
    env_logger::init();

    let args: Args = argh::from_env();
    output::init(args.verbose);

    let mut engine = match args.engine {
        Some(ref name) => engine::by_name(name)?,
        None => engine::select()?,
    };

    let repo = Repository::open_from_env()?;

    output::message("Using", engine.name())?;
    let name = engine.pkg_name();
    let version = engine.pkg_version();

    let status = format!("{name} {version}");
    output::message("Preparing", &status)?;
    engine.prepare()?;

    output::update("Packaging", &status)?;
    let mut package = package::Package::new(&repo)?;
    engine.pack(&mut package)?;
    let tree = package.finish()?;

    output::update("Committing", &status)?;
    let author = repo.signature()?;
    let committer = Signature::now("gitpub", "gitpub")?;
    let message = format!("Publish {name} {version}");
    let tag_name = format!("gitpub/{name}@{version}");
    let tree = repo.find_tree(tree)?;
    let commit = repo.commit(None, &author, &committer, &message, &tree, &[])?;
    let commit = repo.find_object(commit, Some(ObjectType::Commit))?;
    repo.tag(&tag_name, &commit, &committer, &message, false)?;

    if !args.no_publish {
        output::update("Uploading", &status)?;
        Command::new("git").args(["push", "origin", &tag_name]).wait_or_fail()?;
    }

    if !args.keep_tag {
        repo.tag_delete(&tag_name)?;
    }

    output::update("Released", format!("{name} {version} as {tag_name}"))?;

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
