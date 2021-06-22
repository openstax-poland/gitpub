use anyhow::{Context, Result, bail};
use json::{JsonValue, object::Object};
use std::{fs::{self, File}, path::PathBuf, process::Command};

use crate::{package::Package, util::OutputEx};

use super::Engine;

/// Fields to remove from package.json
const REMOVE_FIELDS: &[&str] = &[
    "devDependencies",
    "scripts",
    "workspaces",
];

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

    fn run_script(&self, client: &str, script: &str) -> Result<()> {
        if self.pkg.get("scripts").map_or(false, |scripts| scripts.has_key(script)) {
            Command::new(client).arg(script).output()?.exit_on_fail()?;
        }

        Ok(())
    }

    /// Adjust `package.json` for release
    fn adjust_pkg(&self, data: &mut Vec<u8>) -> Result<()> {
        let mut pkg = json::parse(std::str::from_utf8(&data)?)?;

        for key in REMOVE_FIELDS {
            pkg.remove(key);
        }

        data.clear();
        pkg.write_pretty(data, 2)?;

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
        C::prepare(self)?;
        self.read_pkg()?;
        Ok(())
    }

    fn pack(&mut self, pkg: &mut Package) -> Result<()> {
        let archive = C::archive_name(self);
        let prefix = C::archive_prefix(self);
        let tar = File::open(&archive)?;

        pkg.add_tar(tar, |path, content| {
            let path = match path.strip_prefix(prefix.as_bytes()) {
                Some(path) if !path.is_empty() => path,
                _ => return Ok(None),
            };

            if path == b"package.json" || path == b"/package.json" {
                self.adjust_pkg(content)?;
            }

            Ok(Some(path))
        })?;

        fs::remove_file(self.path.parent().unwrap().join(&archive))?;

        Ok(())
    }

    fn clean(&mut self) -> Result<()> {
        C::postpublish(self)
    }
}

/// npm client
pub trait Client: Sized {
    const NAME: &'static str;

    fn prepare(engine: &JavaScript<Self>) -> Result<()>;

    fn archive_name(engine: &JavaScript<Self>) -> String;

    fn archive_prefix(engine: &JavaScript<Self>) -> String;

    fn postpublish(engine: &JavaScript<Self>) -> Result<()>;
}

pub struct Npm;

impl Client for Npm {
    const NAME: &'static str = "Npm";

    fn prepare(engine: &JavaScript<Self>) -> Result<()> {
        engine.run_script("npm", "prepublish")?;
        engine.run_script("npm", "prepare")?;
        engine.run_script("npm", "prepublishOnly")?;
        Command::new("npm").arg("pack").output()?.exit_on_fail()?;
        Ok(())
    }

    fn archive_name(engine: &JavaScript<Self>) -> String {
        format!("{}-{}.tgz", engine.name, engine.version)
    }

    fn archive_prefix(_: &JavaScript<Self>) -> String {
        String::from("package")
    }

    fn postpublish(engine: &JavaScript<Self>) -> Result<()> {
        engine.run_script("npm", "publish")?;
        engine.run_script("npm", "postpublish")?;
        Ok(())
    }
}

pub struct Yarn;

impl Client for Yarn {
    const NAME: &'static str = "Yarn";

    fn prepare(engine: &JavaScript<Self>) -> Result<()> {
        engine.run_script("yarn", "prepublish")?;
        engine.run_script("yarn", "prepare")?;
        engine.run_script("yarn", "prepublishOnly")?;
        Command::new("yarn").arg("pack").output()?.exit_on_fail()?;
        Ok(())
    }

    fn archive_name(engine: &JavaScript<Self>) -> String {
        format!("{}-v{}.tgz", engine.name, engine.version)
    }

    fn archive_prefix(_: &JavaScript<Self>) -> String {
        String::from("package")
    }

    fn postpublish(engine: &JavaScript<Self>) -> Result<()> {
        engine.run_script("npm", "publish")?;
        engine.run_script("npm", "postpublish")?;
        Ok(())
    }
}

pub struct Yarn2;

impl Client for Yarn2 {
    const NAME: &'static str = "Yarn 2";

    fn prepare(engine: &JavaScript<Self>) -> Result<()> {
        engine.run_script("yarn", "prepublish")?;
        Command::new("yarn").arg("pack").output()?.exit_on_fail()
    }

    fn archive_name(_: &JavaScript<Self>) -> String {
        String::from("package.tgz")
    }

    fn archive_prefix(_: &JavaScript<Self>) -> String {
        String::from("package")
    }

    fn postpublish(engine: &JavaScript<Self>) -> Result<()> {
        Ok(())
    }
}
