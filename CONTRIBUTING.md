# Contributing to sculkr

Thanks for taking an interest! Bug reports, ideas, and pull requests are all
welcome. This is a small project, so nothing here is heavy process — it's mostly
a description of what CI checks so you can run the same things locally before
opening a PR.

## Reporting bugs and ideas

Open an [issue](https://github.com/justin-carver/sculkr/issues). For a bug, the
most useful report includes:

- what you ran (the full `sculkr` invocation, including any `--format` string),
- what you expected and what happened instead,
- the output of the same command with `-vv`, which turns logging all the way up to trace.

Please redact `CF_API_KEY` if it ever shows up in output — it shouldn't, since
the key is wrapped in a `Secret` that redacts itself in both `Display` and
`Debug`, but check anyway.

Feature ideas are welcome too; the [Todo](README.md#todo) list in the README is
a good sense of where things are headed.

## Development setup

You need two toolchains: **stable** for building and testing, **nightly** for
formatting only.

```sh
rustup toolchain install stable nightly
rustup component add rustfmt clippy --toolchain nightly

git clone https://github.com/justin-carver/sculkr.git
cd sculkr
cargo build
```

The minimum supported Rust version is **1.85** (edition 2024), declared as
`rust-version` in `Cargo.toml`.

To run against a real pack, copy `.env.example` to `.env` and point `PACK_ROOT`
at a directory of `*.pw.toml` files. See
[Configuration](README.md#configuration) for the variables and the
single-quotes-around-`CF_API_KEY` trap.

## Before you open a PR

CI runs these on every push and pull request, so running them locally saves a
round trip:

```sh
cargo +nightly fmt --all                                       # format
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo nextest run --locked --all-features                      # tests
cargo test --locked --all-features --doc                       # doctests
RUSTDOCFLAGS="-D warnings" cargo doc --locked --no-deps --all-features
```

`cargo nextest` comes from [cargo-nextest](https://nexte.st/); plain
`cargo test` works fine locally if you'd rather not install it.

Formatting is nightly-only on purpose. `.rustfmt.toml` sets a handful of
nightly options (`group_imports`, `imports_granularity`, `wrap_comments`, and
friends); stable rustfmt warns about each and then ignores them, so `cargo fmt`
on stable will leave the file in a state CI rejects. Use `cargo +nightly fmt`.

## Things the codebase cares about

A few conventions that aren't obvious from reading a single file:

- **No compile-time environment lookups.** `build.rs` scans `src/` and fails the
  build if it finds one. Anything baked in at compile time ends up as a
  plaintext string inside every published release artifact, which is exactly how
  API keys leak. Read values at run time through `crate::env` instead.
- **Secrets go through `Secret`.** It redacts itself in `Display` and `Debug`,
  so it stays hidden even when some future error path wraps it in a message.
  Call `.expose()` at the point of use and never earlier.
- **API failures degrade where they reasonably can.** The Modrinth `/v3`
  organization lookup is the standing example: `/v3` is documented as unstable,
  so a failure there logs a warning and leaves authors empty rather than killing
  the whole run.
- **`--help` is generated from the same source as the README's placeholder
  table.** If you add a placeholder, add it in `src/format.rs` and both stay in
  sync; don't hand-write it into the help text.
- **Output line breaks come only from the format string.** Values are collapsed
  to single spaces before substitution, because a description containing a
  newline would otherwise split a table row across lines.

## Commits and changelog

Commit messages follow [Conventional Commits](https://www.conventionalcommits.org/)
(`feat:`, `fix:`, `chore:`, `docs:`, …). Keep them in the imperative mood.

`CHANGELOG.md` is generated from these commits by
[git-cliff](https://git-cliff.org/), so the commit *is* the changelog entry.
Nothing is written into the file by hand any more.

Two things follow from that:

- **The subject line becomes the bullet.** Write it so it reads well on a
  release page, not just in `git log`. A `:gitmoji:` shortcode after the type is
  fine; it gets stripped on the way in.
- **The commit body becomes the prose under that bullet.** This is where the
  detail that used to go into `CHANGELOG.md` belongs. It is optional, but a
  release note is only as good as the body you wrote a month earlier.

```
feat(export): :sparkles: add JSON export for modlists

Adds a --format json option that emits a machine-readable modlist.
Useful for piping into jq or feeding a pack dashboard.
```

Types map onto [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)
sections: `feat` to Added, `fix` to Fixed, `refactor`/`perf`/`style`/`docs` to
Changed, `deprecate` to Deprecated, `remove` to Removed, `sec` to Security.
`chore`, `ci`, `build` and `test` are dropped, as are merge and release commits.
The mapping lives in [`cliff.toml`](cliff.toml).

To see what the next release would look like at any point:

```sh
git cliff --unreleased
```

Entries for 0.2.0 and earlier were written by hand and are left frozen; only new
sections are generated.

## Releases

Maintainer-only, recorded here so the process isn't folklore.

Releases go through [cargo-release](https://github.com/crate-ci/cargo-release),
configured in `[package.metadata.release]`:

```sh
cargo release <patch|minor|major>
```

That runs [`scripts/changelog.sh`](scripts/changelog.sh) as a pre-release hook,
which asks git-cliff for the commits since the last tag, splices the new section
into `CHANGELOG.md` below the `<!-- next-header -->` marker, and rebuilds the
compare links at the bottom. Then it commits as `release vX.Y.Z`, tags `vX.Y.Z`,
and pushes. Requires `cargo install git-cliff`.

Dates are stamped in `America/Chicago`, set in `cliff.toml`. git-cliff's own
`{{ date }}` is UTC, which is what put 0.2.0 on the wrong day.

Add `--dry-run` to print the section that would be written without touching the
file.

Pushing the tag triggers [`tagged_release.yml`](.github/workflows/tagged_release.yml),
which re-runs lint and tests, builds all five targets, packages archives with
`SHA256SUMS.txt`, attaches a build provenance attestation, publishes the GitHub
release with this version's `CHANGELOG.md` section as the release body (GitHub
appends its own generated commit list underneath), and then publishes to
crates.io via OIDC — no long-lived registry token
lives in the repo. `cargo-release` itself is configured with `publish = false`
precisely so that the workflow owns that step.

A tag whose version doesn't match `Cargo.toml` fails the workflow's first job on
purpose. Running the workflow manually (`workflow_dispatch`) builds and verifies
everything but publishes nothing, which is a good way to test changes to it.

## License

By contributing, you agree that your contributions are licensed under the
[Apache License 2.0](LICENSE), the same as the rest of the project. sculkr began
as a fork of [packwiz-modlist](https://github.com/Ricky12Awesome/packwiz-modlist);
see [NOTICE](NOTICE) for the attribution that carries with it.
