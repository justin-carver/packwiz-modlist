# Security Policy

## Supported versions

`sculkr` is pre-1.0. Only the most recent release on
[crates.io](https://crates.io/crates/sculkr) receives fixes; there are no
maintenance branches for earlier versions.

| Version | Supported |
| ------- | --------- |
| Latest release | Yes |
| Anything older | No |

## Reporting a vulnerability

Report privately through GitHub, using **Security → Report a vulnerability** on
[this repository](https://github.com/justin-carver/sculkr/security/advisories/new).

Please do not open a public issue for a vulnerability. Ordinary bugs belong in
[Issues](https://github.com/justin-carver/sculkr/issues).

Useful things to include: the version, the platform, whether the pack is
Modrinth or CurseForge backed, and the smallest pack or `.sculk` that reproduces
the problem. Redact any CurseForge key before pasting logs.

## Scope

In scope:

- The `sculkr` crate and the binaries published on the
  [releases page](https://github.com/justin-carver/sculkr/releases).
- Handling of `CF_API_KEY`, whether it arrives from the environment, a `.env`,
  or a `[secrets]` table in a `.sculk`.
- Anything written to disk: the cache, `--output` files, and the `--json` export.

Out of scope:

- Vulnerabilities in Modrinth or CurseForge themselves. Report those upstream.
- Vulnerabilities in `packwiz`. Report those to
  [packwiz/packwiz](https://github.com/packwiz/packwiz).
- A CurseForge key committed to a pack's own `.sculk`. `sculkr` warns about this
  on every run; see the README.

## What the project already does

- **No secret is compiled in.** `build.rs` fails the build if anything under
  `src/` reads a variable at compile time, so a key cannot be baked into a
  published binary.
- **Secrets are redacted in output.** `Secret` has no `Serialize`
  implementation, and the `--json` export names each exported field by hand
  rather than serializing the whole config, so a `[secrets]` key cannot reach
  the document.
- **Releases carry provenance.** Every archive is attested by the release
  workflow and can be verified before use:

  ```sh
  gh attestation verify sculkr-<version>-<target>.tar.gz --repo justin-carver/sculkr
  ```

- **Release tooling is pinned.** Every GitHub Action is pinned to a commit SHA
  rather than a mutable tag, and `dependabot` watches both the actions and the
  cargo dependencies.
