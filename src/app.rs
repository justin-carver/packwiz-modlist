use crate::cache::CacheId;
use crate::parser::{ParsedCurseForgeId, ParsedModrinthId, Parser};
use crate::request::{CurseForgeId, ModrinthId};
use crate::{get_curseforge_mods, get_modrinth_projects, Cache, Error, Mod};
use std::cell::{RefCell, RefMut};
use std::collections::{HashMap, HashSet};

/// Warns about ids that were requested but never came back.
///
/// A deleted or renamed project will just vanishes from the list, leaving
/// an incorrect mod count, compared to what packwiz found on disk.
fn warn_missing(source: &str, requested: &HashMap<String, CacheId>, returned: &HashSet<String>) {
  for id in requested.keys() {
    if !returned.contains(id) {
      log::warn!("{source} returned no result for \"{id}\" -- it will be missing from the list");
    }
  }
}

pub struct App {
  cache: RefCell<Cache>,
  modrinth_mods: Vec<ParsedModrinthId>,
  curseforge_mods: Vec<ParsedCurseForgeId>,
}

impl App {
  pub fn new<P>(cache: Cache, parser: P) -> Self
  where
    P: Parser,
  {
    let (modrinth_mods, curseforge_mods) = parser.get_mods_owned();

    Self {
      cache: RefCell::new(cache),
      modrinth_mods,
      curseforge_mods,
    }
  }

  fn get_mods(&self) -> Result<Vec<Mod>, Error> {
    let mut cache = self.cache.borrow_mut();
    let mut mods = Vec::<Mod>::with_capacity(self.modrinth_mods.len() + self.curseforge_mods.len());
    let mut mr_mods_ids = Vec::<ModrinthId>::with_capacity(self.modrinth_mods.len());
    let mut cf_mods_ids = Vec::<CurseForgeId>::with_capacity(self.curseforge_mods.len());
    let mut mr_id_map = HashMap::<String, CacheId>::with_capacity(mr_mods_ids.capacity());
    let mut cf_id_map = HashMap::<String, CacheId>::with_capacity(cf_mods_ids.capacity());

    for id in self.modrinth_mods.iter().cloned() {
      match cache.get_mod(id.clone()).cloned() {
        None => {
          mr_id_map.insert(id.id.clone(), id.clone().into());
          mr_mods_ids.push(id.into());
        }
        Some(m) => mods.push(m),
      }
    }

    for id in self.curseforge_mods.iter().cloned() {
      match cache.get_mod(id.clone()).cloned() {
        None => {
          cf_id_map.insert(id.id.to_string(), id.clone().into());
          cf_mods_ids.push(id.into());
        }
        Some(m) => mods.push(m),
      }
    }

    if !mr_mods_ids.is_empty() {
      let fetched = get_modrinth_projects(mr_mods_ids)?;
      let mut returned = HashSet::<String>::with_capacity(fetched.len());

      for m in fetched.into_iter().map(Mod::from) {
        returned.insert(m.id.clone());

        match mr_id_map.get(&m.id).cloned() {
          Some(id) => {
            cache.set_mod(id, m.clone());
            mods.push(m);
          }
          // An id we never asked for should not take the whole run down.
          None => log::warn!(
            "Modrinth returned unrequested project \"{}\"; ignoring",
            m.id
          ),
        }
      }

      warn_missing("Modrinth", &mr_id_map, &returned);
    }

    if !cf_mods_ids.is_empty() {
      let fetched = get_curseforge_mods(cf_mods_ids)?;
      let mut returned = HashSet::<String>::with_capacity(fetched.len());

      for m in fetched.into_iter().map(Mod::from) {
        returned.insert(m.id.clone());

        match cf_id_map.get(&m.id).cloned() {
          Some(id) => {
            cache.set_mod(id, m.clone());
            mods.push(m);
          }
          None => log::warn!(
            "CurseForge returned unrequested project \"{}\"; ignoring",
            m.id
          ),
        }
      }

      warn_missing("CurseForge", &cf_id_map, &returned);
    }

    Ok(mods)
  }

  /// Every mod in the pack in alphabetical order by title, then id.
  pub fn sorted_mods(&self) -> Result<Vec<Mod>, Error> {
    let mut mods = self.get_mods()?;

    // Tie-break on id so mods sharing a title still land in a fixed order.
    mods.sort_by(|a, b| {
      a.title
        .to_lowercase()
        .cmp(&b.title.to_lowercase())
        .then_with(|| a.id.cmp(&b.id))
    });

    Ok(mods)
  }

  pub fn run(&self) -> Result<(), Error> {
    let mods = self.sorted_mods()?;

    for m in &mods {
      println!("{}", m.title);
    }

    log::info!("listed {} mod(s)", mods.len());
    Ok(())
  }

  /// Persists the cache, pruning anything no longer in the pack first.
  pub fn close(&self) -> Result<(), Error> {
    let mut cache = self.cache.borrow_mut();

    let installed: HashSet<String> = self
      .modrinth_mods
      .iter()
      .map(|m| m.id.clone())
      .chain(self.curseforge_mods.iter().map(|m| m.id.to_string()))
      .collect();

    let pruned = cache.retain_only(&installed);

    if pruned > 0 {
      log::debug!("pruned {pruned} cached mod(s) no longer in the pack");
    }

    cache.save()
  }
}
