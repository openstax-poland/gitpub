use anyhow::{Result, bail};

pub trait Engine {
    /// Engine name
    fn name(&self) -> String;

    /// Package name
    fn pkg_name(&self) -> String;

    /// Package version
    fn pkg_version(&self) -> String;
}

/// Select the most appropriate engine
pub fn select() -> Result<Box<dyn Engine>> {
    bail!("could not select engine, please specify it via --engine")
}

/// Select engine by name
pub fn by_name(name: &str) -> Result<Box<dyn Engine>> {
    match name {
        _ => bail!("no engine named {}", name),
    }
}
