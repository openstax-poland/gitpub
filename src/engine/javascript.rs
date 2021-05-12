use anyhow::{Context, Result, bail};
use json::{JsonValue, object::Object};
use std::{fs, path::PathBuf, process::Command};

use crate::util::OutputEx;

use super::Engine;

/// Select appropriate engine for a JavaScript project
pub fn select(path: PathBuf) -> Result<Box<dyn Engine>> {
    let mut lock = path.parent().unwrap().join("yarn.lock");

    let has_yarn_lock = lock.exists();

    lock.pop();
    lock.push("package-lock.json");

    let has_npm_lock = lock.exists();

    // yarn.lock exists, use yarn
    if has_yarn_lock || !has_npm_lock {
        let ver = Command::new("yarn").arg("--version").output()?;

        if ver.stdout.starts_with(b"1.") {
            return JavaScript::new_in(Yarn, path);
        }

        if ver.stdout.starts_with(b"2.") {
            return JavaScript::new_in(Yarn2, path);
        }

        if has_yarn_lock {
            bail!("found unsupported version of Yarn")
        }
    }

    JavaScript::new_in(Npm, path)
}

/// Engine for JavaScript projects
pub struct JavaScript<C> {
    /// Path to `package.json`
    path: PathBuf,
    /// Contents of `package.json`
    pkg: Object,
    /// Package name
    name: String,
    /// Package version
    version: String,
    /// Client selected
    _client: C,
}

impl<C: Client + 'static> JavaScript<C> {
    fn new(_client: C) -> Result<Box<dyn Engine>> {
        Self::new_in(_client, std::env::current_dir()?.join("package.json"))
    }

    fn new_in(_client: C, path: PathBuf) -> Result<Box<dyn Engine>> {
        let mut engine = JavaScript {
            path,
            pkg: Object::new(),
            name: String::new(),
            version: String::new(),
            _client,
        };

        engine.read_pkg()?;

        Ok(Box::new(engine))
    }
}

impl<C> JavaScript<C> {
    fn read_pkg(&mut self) -> Result<()> {
        let data = fs::read_to_string(&self.path).context("could not read package.json")?;

        self.pkg = match json::parse(&data).context("invalid package.json")? {
            JsonValue::Object(pkg) => pkg,
            _ => bail!("package.json is not a JSON object"),
        };

        self.name = self.pkg.get("name").context("package has no name")?
            .as_str().context("package name is not a string")?
            .to_string();
        self.version = self.pkg.get("version").context("package has no version")?
            .as_str().context("package version is not a string")?
            .to_string();

        Ok(())
    }
}

impl JavaScript<Npm> {
    pub fn npm() -> Result<Box<dyn Engine>> {
        JavaScript::new(Npm)
    }
}

impl JavaScript<Yarn> {
    pub fn yarn() -> Result<Box<dyn Engine>> {
        JavaScript::new(Yarn)
    }
}

impl JavaScript<Yarn2> {
    pub fn yarn2() -> Result<Box<dyn Engine>> {
        JavaScript::new(Yarn2)
    }
}

impl<C: Client> Engine for JavaScript<C> {
    fn name(&self) -> String {
        format!("JavaScript / {}", C::NAME)
    }

    fn pkg_name(&self) -> String {
        self.name.clone()
    }

    fn pkg_version(&self) -> String {
        self.version.clone()
    }

    fn prepare(&mut self) -> Result<()> {
        C::prepare()?;
        self.read_pkg()?;
        Ok(())
    }
}

/// npm client
pub trait Client {
    const NAME: &'static str;

    fn prepare() -> Result<()>;
}

pub struct Npm;

impl Client for Npm {
    const NAME: &'static str = "Npm";

    fn prepare() -> Result<()> {
        Command::new("npm").arg("prepublish").output()?.exit_on_fail()?;
        Command::new("npm").arg("prepublishOnly").output()?.exit_on_fail()?;
        Command::new("npm").arg("prepare").output()?.exit_on_fail()?;
        Ok(())
    }
}

pub struct Yarn;

impl Client for Yarn {
    const NAME: &'static str = "Yarn";

    fn prepare() -> Result<()> {
        Command::new("yarn").arg("prepublish").output()?.exit_on_fail()?;
        Command::new("yarn").arg("prepublishOnly").output()?.exit_on_fail()?;
        Command::new("yarn").arg("prepare").output()?.exit_on_fail()?;
        Ok(())
    }
}

pub struct Yarn2;

impl Client for Yarn2 {
    const NAME: &'static str = "Yarn 2";

    fn prepare() -> Result<()> {
        Command::new("yarn").arg("pack").output()?.exit_on_fail()
    }
}
