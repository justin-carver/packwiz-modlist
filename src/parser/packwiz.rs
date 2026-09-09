use std::{fs::OpenOptions, io::Read, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::{
    Error,
    error::IoContext,
    parser::{ParsedCurseForgeId, ParsedModrinthId, Parser},
};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct PackwizModUpdate {
    pub modrinth: Option<PackwizModUpdateModrinth>,
    pub curseforge: Option<PackwizModUpdateCurseforge>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct PackwizModUpdateModrinth {
    pub mod_id: String,
    pub version: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct PackwizModUpdateCurseforge {
    pub file_id: u32,
    pub project_id: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct PackwizMod {
    pub name: String,
    pub filename: String,
    pub update: PackwizModUpdate,
}

#[derive(Debug, Clone)]
pub struct PackwizParser {
    pub modrinth_mods: Vec<ParsedModrinthId>,
    pub curseforge_mods: Vec<ParsedCurseForgeId>,
}

impl PackwizParser {
    pub fn load_from<T>(directory: T) -> Result<Self, Error>
    where
        T: Into<PathBuf>,
    {
        let directory = directory.into();

        let resolved = crate::util::resolve_for_display(&directory);

        log::debug!("scanning for *.pw.toml in \"{}\"", resolved.display());

        let entries = directory.read_dir().path_ctx(&resolved, "read directory")?;
        let mut parsed_mods = Vec::new();
        let mut skipped = 0usize;

        for entry in entries {
            let entry = entry.path_ctx(&resolved, "read directory entry")?;
            let path = entry.path();

            if !entry.file_name().to_string_lossy().ends_with(".pw.toml") {
                skipped += 1;
                log::trace!("skipping non-pw.toml entry \"{}\"", path.display());
                continue;
            }

            let data = std::fs::read_to_string(&path).path_ctx(&path, "read file")?;
            let parsed = toml::from_str::<PackwizMod>(&data)
                .map_err(|err| Error::TomlFile(path.clone(), err))?;

            log::debug!("parsed \"{}\" as \"{}\"", path.display(), parsed.name);
            parsed_mods.push(parsed);
        }

        log::info!(
            "found {} mod(s) in \"{}\" ({} entries skipped)",
            parsed_mods.len(),
            resolved.display(),
            skipped
        );

        if parsed_mods.is_empty() {
            log::warn!(
                "no *.pw.toml files found in \"{}\" -- is PACK_ROOT pointing at your packwiz mods directory?",
                resolved.display()
            );
        }

        let modrinth_mods = parsed_mods
            .clone()
            .into_iter()
            .filter_map(|m| m.update.modrinth)
            .map(|data| ParsedModrinthId {
                cache_id: data.version,
                id: data.mod_id,
            })
            .collect();

        let curseforge_mods = parsed_mods
            .into_iter()
            .filter_map(|m| m.update.curseforge)
            .map(|data| ParsedCurseForgeId {
                cache_id: data.file_id.to_string(),
                id: data.project_id,
            })
            .collect();

        Ok(Self {
            modrinth_mods,
            curseforge_mods,
        })
    }
}

impl Parser for PackwizParser {
    fn get_mods_owned(self) -> (Vec<ParsedModrinthId>, Vec<ParsedCurseForgeId>) {
        (self.modrinth_mods, self.curseforge_mods)
    }

    fn get_modrinth_mods(&self) -> Vec<ParsedModrinthId> {
        self.modrinth_mods.clone()
    }

    fn get_curseforge_mods(&self) -> Vec<ParsedCurseForgeId> {
        self.curseforge_mods.clone()
    }
}
