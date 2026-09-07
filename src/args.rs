use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

/**
 *  All Code below relies on new clap:: 4.0+ API! (Didn't pin it... for now...)
 *  Perhaps consider moving this into a more centralized "command" file,
 *  depending on future usage.
 */
use clap::{
  builder::{styling::AnsiColor, Styles},
  ArgAction, Args, ColorChoice, CommandFactory, Parser, Subcommand, ValueEnum,
};

const HELP_STYLES: Styles = Styles::styled()
  .header(AnsiColor::Yellow.on_default().bold().underline())
  .usage(AnsiColor::Yellow.on_default().bold())
  .literal(AnsiColor::Green.on_default().bold())
  .placeholder(AnsiColor::Cyan.on_default());

#[derive(Debug, Parser)]
#[command(
  name = "packwizml",
  color = ColorChoice::Auto,
  styles = HELP_STYLES,
  version,
  about = "Creates a modlist from packwiz",
  long_about = "Utilizes the Modrinth and Curseforge API to output modlist information created via the packwiz CLI tool.",
  propagate_version = true,
  arg_required_else_help = true
)]

pub(crate) struct Cli {
  /// Increase logging verbosity (-v, -vv, -vvv)
  #[arg(short, long, action = ArgAction::Count, global = true)]
  pub(crate) verbose: u8,

  /// Suppress all non-error output
  #[arg(short, long, global = true)]
  pub(crate) quiet: bool,

  #[command(subcommand)]
  pub(crate) command: Command,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Command {
  /// Prints information about this program via Cargo.toml
  About,
  /// Print the packwizml configuration to stdout
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
      (_, _) => Verbosity::Debug,
    }
  }
}

/**
 * Commands for clap arg parsing. Can be moved into it's own src/command.rs file,
 * but given how many CLI options there are now, not sure if there's a point.
 * Perhaps if command logic grows exceedingly complicated.
*/

pub(crate) fn config(path: Option<PathBuf>) -> Result<(), String> {
  match path {
    Some(path) => println!("Config would go here, pulled from: {}", path.display()),
    None => println!("Output resolving to None"),
  }
  Ok(())
}

pub(crate) fn about() -> Result<(), String> {
  let cargo_toml = include_str!("../Cargo.toml");
  if (cargo_toml != "") {
    let about_lines = cargo_toml.lines().skip(1).take(10);
    for line in about_lines {
      println!("{}", line)
    }
  }
  std::process::exit(1);
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
