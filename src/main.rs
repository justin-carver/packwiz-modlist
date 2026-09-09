#![allow(unused)]

use std::path::PathBuf;

use clap::Parser;

use crate::{
    app::App,
    args::{Cli, Command},
    cache::Cache,
    error::Error,
    format::Formatter,
    parser::{packwiz::PackwizParser, text::TextParser},
    request::{Mod, curseforge::get_curseforge_mods, modrinth::get_modrinth_mods},
};

mod app;
mod args;
mod cache;
mod consts;
mod env;
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

const CACHE_PATH: &str = ".packwiz-modlist.cache.json";

fn run(cli: Cli) -> Result<(), Error> {
    match std::env::current_dir() {
        Ok(cwd) => log::debug!("working directory: \"{}\"", cwd.display()),
        Err(err) => log::debug!("could not determine working directory: {err}"),
    }

    // Let's parse a few flags rights here, to make sure we catch them in time
    let pack_root = match cli.path.as_ref() {
        Some(path) => path.clone(),
        None => PathBuf::from(".")
            .canonicalize()
            .unwrap_or(PathBuf::from(".")),
    };

    log::debug!("pack root: \"{}\"", pack_root.display());

    let cache = Cache::load(CACHE_PATH)?;

    let pw_parser = PackwizParser::load_from(pack_root)?;
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

    crate::env::load_dotenv();

    // Commands / Subcommands
    // Result of the most recently run subcommand
    let result: anyhow::Result<()> = match cli.command {
        Some(Command::Config { path }) => args::config(path),
        Some(Command::About) => args::about(&mut std::io::stdout().lock()),
        None => run(cli).map_err(anyhow::Error::from),
    };

    if let Err(err) = result {
        log::error!("{err}");
    }
}
