# Changelog

All notable changes to this project are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- next-header -->

## [Unreleased] - ReleaseDate

## Added

- Greatly expanded the `#[test]` suite of the main command/arg processing core in `src/args.rs`. Ensures that all listed args and commands have appropriate surface coverage for any parsable context the app can provide.
- Implemented `insta` and `cargo-insta` into the build pipeline, for local development and testing related to snapshot comparisons. These will soon be integrating into the GH Actions/CI workflow.

### Changed

- Modified `.gitignore` to properly untrack pending `insta` snapshots.
- Updated the way `sculkr config` operates. Originally planned as a drop-in/path enabled way to configure sculkr on the fly, this instead now provides a overview of the local modpack environment with a single command. As this app grows, more content will be added to it's output.
- Updated `README.md` with a more in-depth `Usage` section, detailing how various arguments should be used.
- Decided that going forward, args/commands/subcommands, etc., will implement a writer pattern when flushing data to disk or stdout/err, which will _hopefully_ prevent IO lock issues, if the writes are massive.

### Removed

- Removed various mentions of argument/flag workarounds in `README.md`, due to the include of the `Usage` section.

### Security

- Extended the `Secret()` implement to show the first 4 and last 4 characters `(e.g. $2a$...e345)` of a secret string, to determine if any token/API issues are causing problems.

## [0.1.3] - 2026-09-09

### Added

- Added `-o, --output` flag and verified arg chaining is working! Defaults to stdout for displaying mod lists, but can flush contents to a custom file path.
- Added a proper `-p, --path`command to prefer relative paths and runtime args, instead of relying on hardcoded or `.env` values.

### Fixed

- Adjusted context, layout, description of `about` command when running `help`, added color, and some neat formatting.
- Updated `README.md` and `.env.example` to remove `PACK_ROOT` references.

### Removed

- Removed `PACK_ROOT` from `src/env.rs`

## [0.1.2] - 2026-09-09

### Added

- Added instructions on how to install via `cargo`, build from source, or locate prebuilt binaries once they are introduced (hopefully very soon!)

### Fixed

- Updated `src/args.rs` to include new description information relating to the new branding of sculkr.
- Updated information in README.md to make more sense to new users, added project icon image, README badges, fixed overall layout of README.

## [0.1.1] - 2026-09-09

<!-- next-url -->

[Unreleased]: https://github.com/justin-carver/sculkr/compare/v0.1.3...HEAD
[0.1.3]: https://github.com/justin-carver/sculkr/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/justin-carver/sculkr/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/justin-carver/sculkr/compare/v0.1.0...v0.1.1
