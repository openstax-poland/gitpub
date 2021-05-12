use anyhow::Result;
use argh::FromArgs;

/// Publish package as a GIT tag
#[derive(FromArgs)]
struct Args {
}

fn main() -> Result<()> {
    let args: Args = argh::from_env();

    Ok(())
}
