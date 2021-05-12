use anyhow::Result;
use argh::FromArgs;

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

    Ok(())
}
