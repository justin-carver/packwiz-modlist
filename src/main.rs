#![allow(unused)]

use clap::Parser;

use crate::app::App;
use crate::args::{Cli, Command};
use crate::cache::Cache;
use crate::error::Error;
use crate::format::Formatter;
use crate::parser::packwiz::PackwizParser;
use crate::parser::text::TextParser;
use crate::request::curseforge::get_curseforge_mods;
use crate::request::modrinth::get_modrinth_mods;
use crate::request::Mod;

mod app;
mod args;
mod cache;
mod consts;
mod error;
mod format;
mod parser;
mod request;
mod util;

fn setup_logging(verbosity: args::Verbosity) {
  let level = verbosity.to_level_filter();

  simple_logger::SimpleLogger::new()
    // Dependencies (rustls) dump a wall of TLS handshake traffic at
    // debug/trace, which buries output. Cap them at warn and let the
    // flags raise only this crate's level.
    .with_level(log::LevelFilter::Warn.min(level))
    .with_module_level(env!("CARGO_CRATE_NAME"), level)
    .without_timestamps()
    .env()
    .init()
    .unwrap();

  colored::control::set_override(true);

  #[cfg(windows)]
  colored::control::set_virtual_terminal(true).unwrap();
}

/// Baked in at *build* time by build.rs from `.env`. Editing `.env` has no
/// effect until the crate is rebuilt, and a relative value here resolves
/// against the current working directory at *run* time -- not the crate root.
const MODPACK_TOML_PATH: &str = env!("MODPACK_TOML_PATH");

const CACHE_PATH: &str = ".packwiz-modlist.cache.json";

fn run(cli: Cli) -> Result<(), Error> {
  match std::env::current_dir() {
    Ok(cwd) => log::debug!("working directory: \"{}\"", cwd.display()),
    Err(err) => log::debug!("could not determine working directory: {err}"),
  }
  log::debug!("MODPACK_TOML_PATH (compiled in): \"{MODPACK_TOML_PATH}\"");

  let cache = Cache::load(CACHE_PATH)?;
  // Sodium: AANobbMI (MR)
  // JEI: 238222 (CF)
  // let parser = TextParser::new("mr:AANobbMI:cache_id_here\ncf:238222:cache_id_here")?;
  let pw_parser = PackwizParser::load_from(MODPACK_TOML_PATH)?;
  let app = App::new(cache, pw_parser);

  if let Err(err) = app.run(cli) {
    log::error!("{err}");
  }

  if let Err(err) = app.close() {
    log::error!("{err}");
  }

  Ok(())
}

fn main() {
  // Setting up arg parsing outside of app.rs for now, as these are core commands.
  let cli = args::Cli::parse();

  // Flags
  let verbosity = args::Verbosity::resolve(cli.verbose, cli.quiet);
  // Logging needs to be run after verbosity is resolved, but before any other code that may log.
  setup_logging(verbosity);

  // Commands & Subcommands
  // Result of the most recently run subcommand
  let result = match cli.command {
    Some(Command::Config { path }) => args::config(path),
    Some(Command::About) => args::about(),
    None => run(cli).map_err(|err| err.to_string()),
  };

  if let Err(err) = result {
    log::error!("{err}");
  }
}
