//! Assets are content-hashed into their filename and published only if a template
//! asked for one.
//!
//! nano-web picks caching from the MIME type alone, so CSS, images and fonts are
//! all served `max-age=31536000, immutable`. A stable URL is a promise the build
//! cannot keep; a hashed one makes it true.

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use sha2::{Digest, Sha256};
use walkdir::WalkDir;

/// Their URLs are a convention rather than ours: a crawler asks for /robots.txt
/// and /sitemap.xml at a fixed path, and a browser asks for /favicon.ico when a
/// page links no icon.
const FIXED: &[&str] = &["favicon.ico", "robots.txt", "sitemap.xml"];

pub struct Assets {
    bodies: BTreeMap<String, Vec<u8>>,
    used: RefCell<BTreeSet<String>>,
}

impl Assets {
    pub fn load(dir: &Path) -> Result<Self> {
        let mut bodies = BTreeMap::new();

        for entry in WalkDir::new(dir).into_iter().filter_map(Result::ok) {
            if !entry.file_type().is_file() {
                continue;
            }
            let name = entry
                .path()
                .strip_prefix(dir)?
                .to_string_lossy()
                .into_owned();
            bodies.insert(name, fs::read(entry.path())?);
        }

        Ok(Self {
            bodies,
            used: RefCell::new(FIXED.iter().map(|n| (*n).to_owned()).collect()),
        })
    }

    /// Replaces an asset's bytes before anything asks for its URL. The stylesheet
    /// and the manifest name other assets, so both are rewritten before they can be
    /// hashed themselves.
    pub fn derive(&mut self, name: &str, body: Vec<u8>) {
        self.bodies.insert(name.to_owned(), body);
    }

    /// `logo.png` becomes `logo.a1b2c3d4.png`.
    fn published(name: &str, body: &[u8]) -> String {
        if FIXED.contains(&name) {
            return name.to_owned();
        }

        let hash = format!("{:x}", Sha256::digest(body));
        let hash = &hash[..8];
        match name.rsplit_once('.') {
            Some((stem, ext)) => format!("{stem}.{hash}.{ext}"),
            None => format!("{name}.{hash}"),
        }
    }

    /// The URL for an asset, recording that something wanted it.
    pub fn href(&self, name: &str) -> String {
        let Some(body) = self.bodies.get(name) else {
            let available: Vec<_> = self.bodies.keys().map(String::as_str).collect();
            panic!("no asset `{name}`. Available: {}", available.join(", "));
        };

        self.used.borrow_mut().insert(name.to_owned());
        format!("/{}", Self::published(name, body))
    }

    /// Writes what was referenced, and reports it so a dead link differs from a dead file.
    pub fn publish(&self, dist: &Path) -> Result<BTreeSet<PathBuf>> {
        let mut written = BTreeSet::new();

        for name in self.used.borrow().iter() {
            let Some(body) = self.bodies.get(name) else {
                bail!("referenced but not present: {name}");
            };

            let file = dist.join(Self::published(name, body));
            if let Some(parent) = file.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&file, body).with_context(|| format!("writing {}", file.display()))?;
            written.insert(file);
        }

        Ok(written)
    }
}
