# Local mirror of .github/workflows/ci.yml.
#
# Toolchains are pinned to the ones CI installs, because a green `just` that
# turns red on push is worse than no local check at all. RUSTFLAGS is cleared
# for the same reason: a personal ~/.cargo/config.toml carrying `-Z` flags
# makes every non-nightly invocation fail outright.

# fmt, lint, docs, test: everything CI's lint and test jobs run.
default: fmt lint docs test

# Nightly, since rustfmt.toml sets options stable warns about and ignores.
fmt:
    cargo +nightly fmt --all --check

# What `fmt` would have you do, done.
fmt-fix:
    cargo +nightly fmt --all

lint:
    RUSTFLAGS="" cargo +stable clippy --locked --all-targets --all-features -- -D warnings

docs:
    RUSTFLAGS="" RUSTDOCFLAGS="-D warnings" cargo +stable doc --locked --no-deps --all-features

test:
    RUSTFLAGS="" cargo +stable nextest run --locked --all-features

msrv:
    RUSTFLAGS="" cargo +1.88.0 check --locked --all-targets --all-features

# Everything CI checks, including the MSRV job.
pre-push: default msrv
