use anyhow::Result;
use flate2::read::GzDecoder;
use git2::{Oid, TreeBuilder, Repository};
use tar::Archive;
use std::{collections::BTreeMap, io::Read};

use crate::util;

/// Package builder
pub struct Package<'a> {
    repo: &'a Repository,
    root: TreeBuilder<'a>,
    dirs: BTreeMap<Vec<u8>, TreeBuilder<'a>>,
}

impl<'a> Package<'a> {
    pub fn new(repo: &'a Repository) -> Result<Self> {
        let root = repo.treebuilder(None)?;
        Ok(Package {
            repo,
            root,
            dirs: BTreeMap::new(),
        })
    }

    pub fn finish(mut self) -> Result<Oid> {
        while let Some((path, tree)) = self.dirs.pop_last() {
            let oid = tree.write()?;
            self.add_entry(&path, oid, 0o040000)?;
        }

        self.root.write().map_err(From::from)
    }

    /// Create `TreeBuilder`s for `path` and all of its parent directories if
    /// they don't already exist
    fn ensure_directory(&mut self, mut path: &[u8]) -> Result<()> {
        loop {
            if !self.dirs.contains_key(path) {
                self.dirs.insert(path.to_owned(), self.repo.treebuilder(None)?);
            }

            match split_path(path).0 {
                Some(dir) if !dir.is_empty() => path = dir,
                _ => break Ok(()),
            }
        }
    }

    /// Add contents of a TAR archive
    ///
    /// Files will be passed to `f` before being added to the archive, and can
    /// be modified, or skipped entirely.
    ///
    /// `f` takes file path as it's first
    pub fn add_tar<R, F>(&mut self, tar: R, mut f: F) -> Result<()>
    where
        R: Read,
        F: for<'b> FnMut(&'b [u8], &mut Vec<u8>) -> Result<Option<&'b [u8]>>,
    {
        let tar = GzDecoder::new(tar);
        let mut tar = Archive::new(tar);

        let mut buf = Vec::new();

        for entry in tar.entries()? {
            let mut entry = entry?;
            let perms = entry.header().mode()?;

            buf.clear();
            entry.read_to_end(&mut buf)?;

            let path = entry.path_bytes();

            if let Some(path) = f(&path, &mut buf)? {
                self.add_file(path, &buf, perms as i32)?;
            }
        }

        Ok(())
    }

    /// Add a file
    pub fn add_file(&mut self, path: &[u8], content: &[u8], mode: i32) -> Result<()> {
        let oid = self.repo.blob(content)?;
        self.add_entry(path, oid, 0o100000 | (mode & 0o777))?;
        Ok(())
    }

    /// Add entry at path
    fn add_entry(&mut self, path: &[u8], content: Oid, mode: i32) -> Result<()> {
        log::trace!("add_entry path: {:?} content: {} mode: {:o}",
            util::format_bytes(path), content, mode);
        let (dir, name) = split_path(path);
        let tree = match dir {
            Some(dir) => {
                self.ensure_directory(dir)?;
                self.dirs.get_mut(dir).unwrap()
            }
            None => &mut self.root,
        };
        tree.insert(name, content, mode)?;
        Ok(())
    }
}

fn split_path(path: &[u8]) -> (Option<&[u8]>, &[u8]) {
    let mut parts = path.rsplitn(2, |&b| b == b'/');
    let name = parts.next().unwrap();
    let dir = match parts.next() {
        Some(dir) if !dir.is_empty() => Some(dir),
        _ => None,
    };
    (dir, name)
}
