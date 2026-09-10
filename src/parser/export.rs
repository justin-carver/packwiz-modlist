//! One exportable document describing a pack -- its `pack.toml` metadata, the
//! settings the run resolved, and every mod that came back -- serialized as
//! JSON.
//!
//! These structs are a deliberate *view* rather than `Serialize` on the real
//! types. [`Config`] owns a [`Secret`](crate::env::Secret), which has no
//! `Serialize` on purpose; naming each exported field by hand is what keeps a
//! CurseForge key out of a file that tends to get committed. Widen this by
//! adding a field here, never by deriving `Serialize` on `Config`.
//!
//! There is deliberately no `generatedAt` field. A timestamp would make the
//! output differ on every run, which breaks `insta` snapshots and makes the
//! file churn in git for no information.

use std::path::Path;

use serde::Serialize;

use crate::{Error, config::Config, parser::pack::Pack, request::Mod};

/// Bump when a field is removed or changes meaning, so consumers can branch.
pub const SCHEMA_VERSION: u32 = 1;

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Document<'a> {
    pub schema_version: u32,
    pub pack: PackInfo<'a>,
    pub settings: Settings<'a>,
    /// Borrowed, so a few hundred mods are not cloned to be printed once.
    pub mods: &'a [Mod],
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PackInfo<'a> {
    pub name: &'a str,
    pub author: Option<&'a str>,
    pub version: Option<&'a str>,
    pub minecraft: Option<&'a str>,
    pub loader: Option<Loader<'a>>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Loader<'a> {
    pub name: &'a str,
    pub version: &'a str,
}

/// The resolved global flags. Nothing from `[secrets]` belongs here.
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Settings<'a> {
    pub format: Option<&'a str>,
    pub output: Option<&'a Path>,
    pub path: Option<&'a Path>,
}

// Specifically left "Document" naming ambiguous. Will first be targeting JSON output
// to interface with local scripts, but open to TOML/YAML later upon request (or PR!).
impl<'a> Document<'a> {
    pub fn new(pack: &'a Pack, config: &'a Config, mods: &'a [Mod]) -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            pack: PackInfo {
                name: &pack.name,
                author: pack.author.as_deref(),
                version: pack.version.as_deref(),
                minecraft: pack.versions.minecraft.as_deref(),
                loader: pack
                    .versions
                    .loader()
                    .map(|(name, version)| Loader { name, version }),
            },
            settings: Settings {
                // config.secrets is intentionally absent. See the module docs.
                format: config.format.as_deref(),
                output: config.output.as_deref(),
                path: config.path.as_deref(),
            },
            mods,
        }
    }

    // Will extend specific document functions down the line

    pub fn to_json_pretty(&self) -> Result<String, Error> {
        Ok(serde_json::to_string_pretty(self)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::request::{Author, License};

    fn sample_pack() -> Pack {
        toml::from_str(
            r#"
name = "Example Pack"
author = "minecraft_steve"
version = "1.2.0"

[versions]
minecraft = "1.20.1"
fabric = "0.14.21"
"#,
        )
        .expect("sample should parse")
    }

    fn sample_mod() -> Mod {
        Mod {
            id: "AANobbMI".into(),
            slug: "sodium".into(),
            title: "Sodium".into(),
            description: "A modern rendering engine".into(),
            mod_url: "https://modrinth.com/mod/sodium".into(),
            license: Some(License {
                id: "LGPL-3.0-only".into(),
                name: "LGPL-3.0-only".into(),
                url: None,
            }),
            authors: vec![Author {
                name: "jellysquid3".into(),
                url: "https://modrinth.com/user/jellysquid3".into(),
            }],
            icon_url: None,
            source_url: None,
            issues_url: None,
            wiki_url: None,
        }
    }

    /// Parsed back rather than matched as a string, so pretty-printing
    /// whitespace and field order cannot break the test.
    #[test]
    fn renders_the_document_shape() {
        let pack = sample_pack();
        let config = Config::default();
        let mods = vec![sample_mod()];

        let json = Document::new(&pack, &config, &mods)
            .to_json_pretty()
            .expect("should serialize");

        let doc: serde_json::Value =
            serde_json::from_str(&json).expect("the export should be valid json");

        assert_eq!(doc["schemaVersion"], 1);
        assert_eq!(doc["pack"]["name"], "Example Pack");
        assert_eq!(doc["pack"]["author"], "minecraft_steve");
        assert_eq!(doc["pack"]["minecraft"], "1.20.1");
        assert_eq!(doc["pack"]["loader"]["name"], "fabric");
        assert_eq!(doc["pack"]["loader"]["version"], "0.14.21");

        // The payload, carrying Mod's own camelCase through untouched.
        assert_eq!(doc["mods"].as_array().map(Vec::len), Some(1));
        assert_eq!(doc["mods"][0]["slug"], "sodium");
        assert_eq!(doc["mods"][0]["modUrl"], "https://modrinth.com/mod/sodium");
        assert_eq!(doc["mods"][0]["authors"][0]["name"], "jellysquid3");
    }

    /// A pack that names no loader still exports, with a null rather than a
    /// missing key.
    #[test]
    fn a_pack_without_a_loader_still_exports() {
        let pack: Pack = toml::from_str(r#"name = "Bare""#).expect("should parse");

        let json = Document::new(&pack, &Config::default(), &[])
            .to_json_pretty()
            .expect("should serialize");

        let doc: serde_json::Value = serde_json::from_str(&json).expect("valid json");

        assert!(doc["pack"]["loader"].is_null());
        assert!(doc["pack"]["minecraft"].is_null());
    }

    /// The whole reason these structs exist rather than `Serialize` on `Config`.
    #[test]
    fn a_configured_api_key_never_reaches_the_export() {
        let config: Config = toml::from_str(
            r#"
format = "{NAME}"

[secrets]
cf-api-key = "super-secret-value"
"#,
        )
        .expect("config should parse");

        // Without this the test would pass just as happily on a config that
        // never loaded a key at all, which proves nothing.
        let key = config
            .secrets
            .cf_api_key
            .as_ref()
            .expect("the sample key should have been loaded");

        let json = Document::new(&sample_pack(), &config, &[])
            .to_json_pretty()
            .expect("should serialize");

        assert!(
            !json.contains(key.expose()),
            "the raw key reached the export"
        );
        assert!(
            !json.contains(&key.fingerprint()),
            "a fingerprint of the key reached the export"
        );

        // Settings is an allowlist. A new field has to be added deliberately,
        // and this fails until it is added here too.
        let doc: serde_json::Value = serde_json::from_str(&json).expect("valid json");
        let fields: Vec<&str> = doc["settings"]
            .as_object()
            .expect("settings should be an object")
            .keys()
            .map(String::as_str)
            .collect();

        assert_eq!(fields, ["format", "output", "path"]);
    }
}
