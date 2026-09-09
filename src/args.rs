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

    #[clap(
        short,
        long,
        global = true,
        value_name = "PATH",
        help = format!("The path to the packwiz root directory. [default: {:?}]", PathBuf::from(".").canonicalize().unwrap_or(PathBuf::from("."))),
    )]
    pub(crate) path: Option<PathBuf>,

    #[clap(short, long, global = true, value_name = "PATH")]
    /// Sets a custom output path for the modlist [default: stdout]
    pub(crate) output: Option<PathBuf>,

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
    /// Prints information about this program, including version, authors, and description
    About,
    /// Print the sculkr configuration to stdout
    Config,
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

fn write_fancy_header(out: &mut dyn std::io::Write, subtitle: &str) -> anyhow::Result<()> {
    writeln!(out)?;
    writeln!(
        out,
        "  {} {} {}",
        "⣿".cyan(),
        "sculkr".bold().cyan(),
        "⣿".cyan()
    )?;
    writeln!(out, "  {}", subtitle.dimmed())?;
    writeln!(out)?;
    Ok(())
}

/// What `config` reports, read once before any of it is rendered.
///
/// Every field that can fail holds that failure instead of returning it. A
/// missing key or an unreadable pack directory is the most useful thing this
/// command has to say, so it is a line in the output, not a reason to bail.
struct Runtime {
    pack_root: PathBuf,
    /// Modrinth and CurseForge counts, or `None` when there is no pack to read.
    mods: Option<(usize, usize)>,
    cache: PathBuf,
    cache_entries: Option<usize>,
    api_key: Option<String>,
    output: Option<PathBuf>,
    format: String,
    verbosity: Verbosity,
}

impl Runtime {
    fn gather(cli: &Cli) -> Self {
        let pack_root = crate::util::resolve_for_display(
            cli.path.clone().unwrap_or_else(|| PathBuf::from(".")),
        );

        let cache = crate::util::resolve_for_display(crate::CACHE_PATH);

        // An absent cache file and an unreadable one both read as "nothing to
        // report", and `Cache::load` already logs which of the two it was.
        let cache_entries = crate::cache::Cache::load(&cache)
            .ok()
            .filter(|_| cache.exists())
            .map(|cache| cache.get_data().len());

        Self {
            mods: crate::parser::packwiz::PackwizParser::load_from(&pack_root)
                .ok()
                .map(|pack| (pack.modrinth_mods.len(), pack.curseforge_mods.len()))
                // A directory we cannot read and one with nothing in it have the
                // same answer, and it is not a green zero.
                .filter(|(modrinth, curseforge)| modrinth + curseforge > 0),
            pack_root,
            cache,
            cache_entries,
            api_key: crate::env::curseforge_api_key()
                .ok()
                .map(|key| key.fingerprint()),
            output: cli.output.clone(),
            format: cli.format.clone(),
            verbosity: Verbosity::resolve(cli.verbose, cli.quiet),
        }
    }
}

/// Outputs the current runtime configuration of the program.
pub(crate) fn config(out: &mut dyn std::io::Write, cli: &Cli) -> anyhow::Result<()> {
    render_config(out, &Runtime::gather(cli))
}

/// Split from [`config`] so the layout can be tested without a pack, a cache,
/// or a key in the environment. Yay, testing!
fn render_config(out: &mut dyn std::io::Write, rt: &Runtime) -> anyhow::Result<()> {
    write_fancy_header(out, "Runtime Configuration");

    writeln!(
        out,
        "  {:<12} {}",
        "Pack root:".bold(),
        rt.pack_root.display()
    )?;

    match rt.mods {
        Some((modrinth, curseforge)) => writeln!(
            out,
            "  {:<12} {} ({modrinth} Modrinth, {curseforge} CurseForge)",
            "Mods:".bold(),
            (modrinth + curseforge).to_string().green()
        )?,
        None => writeln!(
            out,
            "  {:<12} {}",
            "Mods:".bold(),
            "no *.pw.toml files found here".yellow()
        )?,
    }

    match rt.cache_entries {
        Some(entries) => writeln!(
            out,
            "  {:<12} {} ({entries} cached)",
            "Cache:".bold(),
            rt.cache.display()
        )?,
        None => writeln!(
            out,
            "  {:<12} {} {}",
            "Cache:".bold(),
            rt.cache.display(),
            "(not written yet)".dimmed()
        )?,
    }

    match &rt.api_key {
        Some(fingerprint) => writeln!(
            out,
            "  {:<12} {}",
            format!("{}:", crate::env::CF_API_KEY).bold(),
            fingerprint.green()
        )?,
        None => writeln!(
            out,
            "  {:<12} {} {}",
            format!("{}:", crate::env::CF_API_KEY).bold(),
            "not set".yellow(),
            "(CurseForge mods will fail)".dimmed()
        )?,
    }

    writeln!(out, "  {:<12} {}", "Output:".bold(), match &rt.output {
        Some(path) => path.display().to_string(),
        None => "stdout".to_owned(),
    })?;
    writeln!(out, "  {:<12} {}", "Format:".bold(), rt.format)?;
    writeln!(
        out,
        "  {:<12} {}",
        "Log level:".bold(),
        rt.verbosity.to_level_filter().to_string().to_lowercase()
    )?;
    writeln!(out)?;

    Ok(())
}

pub(crate) fn about(out: &mut dyn std::io::Write) -> anyhow::Result<()> {
    write_fancy_header(out, &format!("Companion CLI for {}", "packwiz".yellow()));

    writeln!(
        out,
        "  {:<12} {}",
        "Version:".bold(),
        env!("CARGO_PKG_VERSION")
    )?;
    writeln!(
        out,
        "  {:<12} {}",
        "Authors:".bold(),
        env!("CARGO_PKG_AUTHORS")
    )?;
    writeln!(
        out,
        "  {:<12} {}",
        "Description:".bold(),
        env!("CARGO_PKG_DESCRIPTION")
    )?;
    writeln!(
        out,
        "  {:<12} {}",
        "Repository:".bold(),
        env!("CARGO_PKG_REPOSITORY")
    )?;
    writeln!(
        out,
        "  {:<12} {}",
        "License:".bold(),
        env!("CARGO_PKG_LICENSE")
    )?;
    writeln!(out)?;

    Ok(())
}

// Tests
#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    use super::*;

    /// Builds the CLI and validates its definition on the way out.
    ///
    /// Test order is not guaranteed, so anything needing a validated `Command`
    /// goes through here.
    fn cli() -> clap::Command {
        let cmd = Cli::command();
        cmd.clone().debug_assert();
        cmd
    }

    /// Catches conflicting short flags, bad `conflicts_with` names, and other
    /// definition mistakes at test time instead of at the user's first run.
    #[test]
    fn cli_definition_is_valid() {
        Cli::command().debug_assert();
    }

    /// The shape of the CLI: which subcommands and flags exist.
    ///
    /// Here so that changing the surface has to be a deliberate act.
    mod cli_surface {
        use std::collections::BTreeSet;

        use super::*;

        /// Walks the command tree and records every subcommand and argument.
        ///
        /// clap creates `help` and `version` arguments on every command, so
        /// those are skipped.
        fn collect_surface(cmd: &clap::Command, prefix: &str, out: &mut BTreeSet<String>) {
            for arg in cmd.get_arguments() {
                if matches!(arg.get_id().as_str(), "help" | "version") {
                    continue;
                }
                let id = arg.get_id().as_str();
                if arg.is_positional() {
                    out.insert(format!("{prefix}<{id}>"));
                } else {
                    out.insert(format!("{prefix}--{id}"));
                }
            }
            for sub in cmd.get_subcommands() {
                out.insert(format!("{prefix}{}", sub.get_name()));
                collect_surface(sub, &format!("{prefix}{} ", sub.get_name()), out);
            }
        }

        /// Adding a variant to [`Command`] breaks this match, which is the
        /// reminder to extend `expected` below.
        /// Don't add a default arm, or we'll fall through the test!
        #[allow(dead_code)]
        fn subcommands_are_exhaustive(c: &Command) {
            match c {
                Command::About | Command::Config => {}
            }
        }

        #[test]
        fn cli_surface_matches_expectations() {
            let mut actual = BTreeSet::new();
            collect_surface(&cli(), "", &mut actual);

            // Do not add "help": clap synthesises that subcommand while building the
            // command, and `cli()` hands back the un-built definition.
            let expected: BTreeSet<String> = [
                "about",
                "config",
                "--verbose",
                "--quiet",
                "--path",
                "--output",
                "--format",
            ]
            .into_iter()
            .map(String::from)
            .collect();

            let missing: Vec<_> = expected.difference(&actual).collect();
            let unexpected: Vec<_> = actual.difference(&expected).collect();

            assert!(
                missing.is_empty() && unexpected.is_empty(),
                "CLI surface changed, update this test.\n  \
                 In the list but not in clap (removed or renamed?): {missing:?}\n  \
                 In clap but not in the list (newly added?): {unexpected:?}",
            );
        }

        /// Pins the whole `--help` render. Review changes with `cargo insta review`
        ///
        /// `term_width` is fixed because clap's `wrap_help` otherwise wraps to the
        /// detected terminal width, which would make this snapshot machine-dependent
        #[test]
        fn help_snapshot() {
            let help = cli().term_width(80).render_long_help().to_string();

            // The --path help text interprets the canonicalised working directory,
            // which differs on every machine. Masking only the quoted default
            // leaves --format's own `[default: ...]`
            insta::with_settings!({filters => vec![
                (r#"\[default: "[^"]*"\]"#, r#"[default: "[CWD]"]"#),
            ]}, {
                insta::assert_snapshot!(help);
            });
        }
    }

    /// Tests for the commands themselves, no arg parsing.
    ///
    /// Arguably some of the most important tests, since they are the core
    /// functionality of the program.
    mod commands {
        use super::*;

        #[test]
        fn about_command_outputs_expected() -> anyhow::Result<()> {
            let mut buf = Vec::new();
            about(&mut buf)?;
            let rendered = String::from_utf8(buf)?;

            // The version changes on release, and `colored` may add ANSI codes
            // depending on the terminal. We strip those and normalize the version
            // so the snapshot stays stable.
            insta::with_settings!({filters => vec![
                (r"\x1b\[[0-9;]*m", ""),
                (r"\d+\.\d+\.\d+", "[VERSION]"),
            ]}, {
                insta::assert_snapshot!(rendered);
            });
            Ok(())
        }

        /// `Runtime` is built by hand so the layout is pinned without a pack, a
        /// cache, or a key on the machine running the test.
        fn render(rt: &Runtime) -> anyhow::Result<String> {
            let mut buf = Vec::new();
            render_config(&mut buf, rt)?;
            Ok(String::from_utf8(buf)?)
        }

        #[test]
        fn config_reports_a_working_setup() -> anyhow::Result<()> {
            let rendered = render(&Runtime {
                pack_root: PathBuf::from("/home/user/modpack"),
                mods: Some((32, 15)),
                cache: PathBuf::from("/home/user/modpack/.packwiz-modlist.cache.json"),
                cache_entries: Some(38),
                api_key: Some("$2a$...e345".to_owned()),
                output: Some(PathBuf::from("modlist.md")),
                format: crate::format::DEFAULT_FORMAT.to_owned(),
                verbosity: Verbosity::Info,
            })?;

            insta::with_settings!({filters => vec![(r"\x1b\[[0-9;]*m", "")]}, {
                insta::assert_snapshot!(rendered);
            });
            Ok(())
        }

        /// Nothing set up yet, which is the run where this command is worth having.
        #[test]
        fn config_reports_what_is_missing() -> anyhow::Result<()> {
            let rendered = render(&Runtime {
                pack_root: PathBuf::from("/home/user"),
                mods: None,
                cache: PathBuf::from("/home/user/.packwiz-modlist.cache.json"),
                cache_entries: None,
                api_key: None,
                output: None,
                format: crate::format::DEFAULT_FORMAT.to_owned(),
                verbosity: Verbosity::Normal,
            })?;

            insta::with_settings!({filters => vec![(r"\x1b\[[0-9;]*m", "")]}, {
                insta::assert_snapshot!(rendered);
            });
            Ok(())
        }
    }
}
