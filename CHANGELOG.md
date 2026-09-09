# Changelog

All notable changes to this project are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- next-header -->

## [Unreleased] - ReleaseDate

## [0.1.3] - 2026-09-09

### Added

- Added `-o, --output` flag and verified arg chaining is working! Defaults to stdout for displaying mod lists, but can flush contents to a custom file path.
  = Added a proper `-p, --path`command to prefer relative paths and runtime args, instead of relying on hardcoded or `.env` values.

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
