use anyhow::Result;
use argh::FromArgs;
use std::fmt;

mod engine;

/// Publish package as a GIT tag
#[derive(FromArgs)]
struct Args {
    /// select engine
    #[argh(option)]
    engine: Option<String>,
}

fn main() -> Result<()> {
    let args: Args = argh::from_env();

    let engine = match args.engine {
        Some(ref name) => engine::by_name(name)?,
        None => engine::select()?,
    };

    println!("{}       Using{} {}", S, R, engine.name());
    println!("{}   Packaging{} {} {}", S, R, engine.pkg_name(), engine.pkg_version());

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
