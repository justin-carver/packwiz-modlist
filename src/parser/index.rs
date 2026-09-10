//! packwiz's `index.toml`, the manifest of every file in a pack.
//!
//! This makes discovery folder-agnostic. Each entry carries its path
//! relative to the pack root, so `mods/`, `resourcepacks/`, `shaderpacks/`,
//! `datapacks/` and anything a future pack invents all arrive the same way,
//! without this crate naming any of them.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::{Error, error::IoContext};

pub const INDEX_FILE_NAME: &str = "index.toml";

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all(deserialize = "kebab-case", serialize = "snake_case"))]
pub struct Index {
    pub hash_format: Option<String>,

    #[serde(default)]
    pub files: Vec<IndexFile>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all(deserialize = "kebab-case", serialize = "snake_case"))]
pub struct IndexFile {
    /// Relative to the pack root, e.g. `mods/sodium.pw.toml`.
    pub file: PathBuf,
    pub hash: String,
    /// True for a `*.pw.toml` describing something to download, false for a
    /// file the pack ships as-is: configs, a LICENSE, a cache.
    #[serde(default)]
    pub metafile: bool,
}

impl IndexFile {
    /// The folder this belongs to, relative to the pack root: `mods`,
    /// `resourcepacks`, `datapacks`, and so on.
    ///
    /// [None] for a file sitting at the root itself.
    pub fn category(&self) -> Option<&str> {
        let parent = self.file.parent()?;

        if parent.as_os_str().is_empty() {
            return None;
        }

        parent.components().next()?.as_os_str().to_str()
    }
}

impl Index {
    pub fn read<P>(path: P) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let data = std::fs::read_to_string(path).path_ctx(path, "read file")?;

        toml::from_str(&data).map_err(|err| Error::TomlFile(path.to_owned(), err))
    }

    /// Only the entries that describe a download, in index order.
    pub fn metafiles(&self) -> impl Iterator<Item = &IndexFile> {
        self.files.iter().filter(|entry| entry.metafile)
    }

    /// Every folder the index mentions, deduplicated, in first-seen order.
    pub fn categories(&self) -> Vec<&str> {
        let mut seen = Vec::new();

        for category in self.files.iter().filter_map(IndexFile::category) {
            if !seen.contains(&category) {
                seen.push(category);
            }
        }

        seen
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"
hash-format = "sha256"

[[files]]
file = "LICENSE"
hash = "aaa"

[[files]]
file = "mods/sodium.pw.toml"
hash = "bbb"
metafile = true

[[files]]
file = "resourcepacks/faithful.pw.toml"
hash = "ccc"
metafile = true

[[files]]
file = "config/sodium.json"
hash = "ddd"
"#;

    #[test]
    fn separates_metafiles_from_shipped_files() {
        let index: Index = toml::from_str(SAMPLE).expect("sample should parse");

        assert_eq!(index.files.len(), 4);
        assert_eq!(index.metafiles().count(), 2);
    }

    /// The folder is read off the entry rather than being a name this crate
    /// knows, which is what lets a new pack folder work without a code change.
    #[test]
    fn reads_the_folder_off_each_entry() {
        let index: Index = toml::from_str(SAMPLE).expect("sample should parse");

        let categories: Vec<Option<&str>> = index.files.iter().map(IndexFile::category).collect();

        assert_eq!(categories, [
            None,
            Some("mods"),
            Some("resourcepacks"),
            Some("config")
        ]);
        assert_eq!(index.categories(), ["mods", "resourcepacks", "config"]);
    }
}
