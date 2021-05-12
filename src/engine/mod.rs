use anyhow::{Result, bail};

mod javascript;

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
    let mut path = std::env::current_dir()?;

    path.push("package.json");
    if path.exists() {
        return javascript::select(path);
    }

    bail!("could not select engine, please specify it via --engine")
}

/// Select engine by name
pub fn by_name(name: &str) -> Result<Box<dyn Engine>> {
    match name {
        "npm" => javascript::JavaScript::npm(),
        "yarn" => javascript::JavaScript::yarn(),
        "yarn2" | "yarn-2" | "yarn-berry" => javascript::JavaScript::yarn2(),
        _ => bail!("no engine named {}", name),
    }
}
