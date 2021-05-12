use anyhow::{Result, bail};

pub trait Engine {
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
