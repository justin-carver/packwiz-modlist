//! Reads packwiz's own `pack.toml` -- the pack-level metadata sitting beside
//! the `*.pw.toml` files that [`super::packwiz`] scans.

use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{Error, error::IoContext};

pub const PACK_FILE_NAME: &str = "pack.toml";

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Pack {
    pub name: String,
    pub author: Option<String>,
    pub version: Option<String>,
    /// e.g. `packwiz:1.1.0`.
    pub pack_format: Option<String>,
    #[serde(default)]
    pub versions: PackVersions,
}

/// `[versions]` holds `minecraft` plus one loader key, where the key name *is*
/// the loader (`fabric`, `forge`, `quilt`, `neoforge`). Collecting the rest
/// rather than enumerating them means a new loader does not need a code change.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct PackVersions {
    pub minecraft: Option<String>,

    #[serde(flatten)]
    pub loaders: BTreeMap<String, String>,
}

impl PackVersions {
    /// The loader and its version, if the pack names one.
    pub fn loader(&self) -> Option<(&str, &str)> {
        self.loaders
            .iter()
            .next()
            .map(|(name, version)| (name.as_str(), version.as_str()))
    }
}

impl Pack {
    pub fn load_from<P>(pack_root: P) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        let path: PathBuf = pack_root.as_ref().join(PACK_FILE_NAME);

        log::debug!("reading pack metadata from \"{}\"", path.display());

        let data = std::fs::read_to_string(&path).path_ctx(&path, "read file")?;

        toml::from_str(&data).map_err(|err| Error::TomlFile(path, err))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A representative packwiz `pack.toml`, loader key and all.
    const SAMPLE: &str = r#"
name = "Example Pack"
author = "minecraft_steve"
version = "1.2.0"
pack-format = "packwiz:1.1.0"

[index]
file = "index.toml"
hash-format = "sha256"
hash = "deadbeef"

[versions]
minecraft = "1.20.1"
fabric = "0.14.21"
"#;

    #[test]
    fn parses_a_pack_toml() {
        let pack: Pack = toml::from_str(SAMPLE).expect("sample should parse");

        assert_eq!(pack.name, "Example Pack");
        assert_eq!(pack.author.as_deref(), Some("minecraft_steve"));
        assert_eq!(pack.pack_format.as_deref(), Some("packwiz:1.1.0"));
        assert_eq!(pack.versions.minecraft.as_deref(), Some("1.20.1"));
    }

    /// `minecraft` must not be mistaken for the loader.
    #[test]
    fn finds_the_loader_beside_minecraft() {
        let pack: Pack = toml::from_str(SAMPLE).expect("sample should parse");

        assert_eq!(pack.versions.loader(), Some(("fabric", "0.14.21")));
    }

    #[test]
    fn a_pack_without_versions_is_still_valid() {
        let pack: Pack = toml::from_str(r#"name = "Bare""#).expect("should parse");

        assert_eq!(pack.versions.minecraft, None);
        assert_eq!(pack.versions.loader(), None);
    }
}
