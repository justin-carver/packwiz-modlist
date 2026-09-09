use std::{env, path::PathBuf, process::ExitCode};

///  All Code below relies on new clap:: 4.0+ API! (Didn't pin it... for now...)
///  Perhaps consider moving this into a more centralized "command" file,
///  depending on future usage.
use clap::{
    ArgAction, Args, ColorChoice, CommandFactory, Parser, Subcommand, ValueEnum,
    builder::{Styles, styling::AnsiColor},
};
use colored::Colorize;

/// Built from the placeholder table so `--help` can never drift from what the
/// formatter actually accepts.
fn format_long_help() -> String {
    format!(
        "Sets a custom output format for the modlist.\n\n\
     A string literal with {{PLACEHOLDER}} holes in it, one per field the cache holds.\n\
     Backslash escapes (\\n, \\t, \\\\, \\{{, \\}}) are resolved by sculkr rather than by\n\
     the shell, so quote the template and write \\n where you want a line break.\n\n\
     Placeholders with no value for a given mod (CurseForge sends no license,\n\
     Modrinth sends no authors) render as an empty string.\n\n\
     Available placeholders:\n{}\n\n\
     [default: {}]",
        crate::format::placeholder_help(),
        crate::format::DEFAULT_FORMAT
    )
}

const HELP_STYLES: Styles = Styles::styled()
    .header(AnsiColor::Yellow.on_default().bold().underline())
    .usage(AnsiColor::Yellow.on_default().bold())
    .literal(AnsiColor::Green.on_default().bold())
    .placeholder(AnsiColor::Cyan.on_default());

#[derive(Debug, Parser)]
#[command(
  name = "sculkr",
  color = ColorChoice::Auto,
  styles = HELP_STYLES,
  version,
  about = "Companion CLI for packwiz - generate modlists and track Minecraft modpack changes",
  long_about = "A companion CLI application for packwiz that parses its output data to deliver advanced utility commands and extended features for Minecraft modpack development.",
  propagate_version = true,
  // DEBUG TESTING
  // arg_required_else_help = true
)]
pub(crate) struct Cli {
    /// Increase logging verbosity (-v, -vv, -vvv)
    #[arg(short, long, action = ArgAction::Count, global = true)]
    pub(crate) verbose: u8,

    /// Suppress all non-error output
    #[arg(short, long, global = true)]
    pub(crate) quiet: bool,

    /// Sets a custom output format for the modlist
    ///
    /// A string literal with {PLACEHOLDER} holes in it, one per field the cache
    /// holds. Backslash escapes (\n, \t, \\, \{, \}) are resolved here rather
    /// than by the shell, so quote the template and write \n for a line break.
    #[clap(
    long,
    short = 'f',
    allow_hyphen_values = true,
    default_value = crate::format::DEFAULT_FORMAT,
    // clap debug-prints defaults, which would show the literal \n as \\n; the
    // long help states it plainly instead.
    hide_default_value = true,
    long_help = format_long_help()
  )]
    pub(crate) format: String,

    #[command(subcommand)]
    pub(crate) command: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Command {
    // TODO: This needs to be done differently, probably hardcoded.
    /// Prints information about this program via Cargo.toml
    About,
    /// Print the sculkr configuration to stdout
    Config {
        #[arg(short, long, value_name = "PATH")]
        /// The config file path [default: ../]
        path: Option<PathBuf>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Verbosity {
    Quiet,
    Normal,
    Info,
    Debug,
}

impl Verbosity {
    pub fn resolve(verbose: u8, quiet: bool) -> Self {
        match (quiet, verbose) {
            (true, _) => Verbosity::Quiet,
            (_, 0) => Verbosity::Normal,
            (_, 1) => Verbosity::Info,
            (..) => Verbosity::Debug,
        }
    }

    /// Converts the verbosity level to a log::LevelFilter for use with the log crate.
    pub fn to_level_filter(self) -> log::LevelFilter {
        match self {
            Verbosity::Quiet => log::LevelFilter::Error,
            Verbosity::Normal => log::LevelFilter::Warn,
            Verbosity::Info => log::LevelFilter::Info,
            Verbosity::Debug => log::LevelFilter::Trace,
        }
    }
}

// Commands for clap arg parsing. Could move into its own src/command.rs, but with
// this few options there is not much point yet.
pub(crate) fn config(path: Option<PathBuf>) -> Result<(), String> {
    match path {
        Some(path) => println!("Config would go here, pulled from: {}", path.display()),
        None => println!("Output resolving to None"),
    }
    Ok(())
}

pub(crate) fn about() -> Result<(), String> {
    println!();
    println!("  {} {} {}", "⣿".cyan(), "sculkr".bold().cyan(), "⣿".cyan());
    println!("  {}", "Companion CLI for packwiz".dimmed());
    println!();
    println!("  {:<12} {}", "Version:".bold(), env!("CARGO_PKG_VERSION"));
    println!("  {:<12} {}", "Authors:".bold(), env!("CARGO_PKG_AUTHORS"));
    println!(
        "  {:<12} {}",
        "Description:".bold(),
        env!("CARGO_PKG_DESCRIPTION")
    );
    println!(
        "  {:<12} {}",
        "Repository:".bold(),
        env!("CARGO_PKG_REPOSITORY")
    );
    println!("  {:<12} {}", "License:".bold(), env!("CARGO_PKG_LICENSE"));
    println!();

    Ok(())
}

// Tests
#[cfg(test)]
mod tests {
    use super::*;
    // TODO: Try to keep these sections up to date. Pretty important.

    /// Catches conflicting short flags, bad `conflicts_with` names, and other
    /// definition mistakes at test time instead of at the user's first run.
    #[test]
    fn cli_definition_is_valid() {
        Cli::command().debug_assert();
    }
}
