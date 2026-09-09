default: fmt lint test

fmt:
    cargo +nightly fmt --all --check

lint:
    cargo clippy --locked --all-targets --all-features -- -D warnings

test:
    cargo nextest run --locked --all-features

msrv:
    RUSTFLAGS="" cargo +1.88.0 check --locked --all-targets --all-features

pre-push: default msrv
