use anyhow::{Result, bail};

use crate::package::Package;

mod javascript;

pub trait Engine {
    /// Engine name
    fn name(&self) -> String;

    /// Package name
    fn pkg_name(&self) -> String;

    /// Package version
    fn pkg_version(&self) -> String;

    /// Prepare the package for publishing
    fn prepare(&mut self) -> Result<()>;

    /// Add files to the package
    fn pack(&mut self, pkg: &mut Package) -> Result<()>;

    /// Clean project after successful publish
    fn clean(&mut self) -> Result<()>;
}

#[derive(Default)]
#[non_exhaustive]
pub struct Options {
    /// Keep build artefacts
    pub keep_artefacts: bool,
}

/// Select the most appropriate engine
pub fn select(options: Options) -> Result<Box<dyn Engine>> {
    let mut path = std::env::current_dir()?;

    path.push("package.json");
    if path.exists() {
        return javascript::select(path, options);
    }

    bail!("could not select engine, please specify it via --engine")
}

/// Select engine by name
pub fn by_name(name: &str, options: Options) -> Result<Box<dyn Engine>> {
    match name {
        "npm" => javascript::JavaScript::npm(options),
        "yarn" => javascript::JavaScript::yarn(options),
        "yarn2" | "yarn-2" | "yarn3" | "yarn-3" | "yarn4" | "yarn-4" | "yarn-berry" =>
            javascript::JavaScript::yarn_modern(options),
        _ => bail!("no engine named {}", name),
    }
}
