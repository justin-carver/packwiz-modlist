//! After some research, looks like GitHub Action artifacts are a great way
//! to accidentally leak secrets, so let's fix that here.
//!
//! None of these are resolved while compiling. The compiler never sees the
//! values, so they cannot end up as string literals inside a published binary.
//! `build.rs` enforces that: it fails the build if a compile-time variable
//! lookup shows up anywhere under `src/`.

use std::{fmt, path::PathBuf};

use crate::error::Error;

/// Directory holding the packwiz pack.
pub const PACK_ROOT: &str = "PACK_ROOT";
pub const CF_API_KEY: &str = "CF_API_KEY";

/// A value that must not reach logs, errors, or serialized output.
///
/// Custom implements for `Debug` and `Display` both redact, so it stays hidden even when something
/// wraps it in a message on a path nobody thought about.
#[derive(Clone)]
pub struct Secret(String);

impl Secret {
    /// Hands out the real value. Call this at the point of use, never earlier.
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for Secret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Secret(<redacted>)")
    }
}

impl fmt::Display for Secret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("<redacted>")
    }
}

/// Pulls `.env` into the process environment for local runs. Absent file is
/// fine; anything already set in the real environment wins.
pub fn load_dotenv() {
    match dotenvy::dotenv() {
        Ok(path) => log::debug!("loaded environment from \"{}\"", path.display()),
        Err(err) if err.not_found() => {}
        Err(err) => log::warn!("could not read .env: {err}"),
    }
}

/// Defaults to the working directory, which is what a user in their pack
/// folder expects.
pub fn pack_root() -> PathBuf {
    match std::env::var(PACK_ROOT) {
        Ok(value) if !value.trim().is_empty() => PathBuf::from(value),
        _ => PathBuf::from("."),
    }
}

pub fn curseforge_api_key() -> Result<Secret, Error> {
    match std::env::var(CF_API_KEY) {
        Ok(value) if !value.trim().is_empty() => Ok(Secret(value)),
        _ => Err(Error::MissingEnv(CF_API_KEY)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_secret_redacts_itself_in_both_formats() {
        let secret = Secret("hunter2".to_owned());

        assert_eq!(format!("{secret}"), "<redacted>");
        assert_eq!(format!("{secret:?}"), "Secret(<redacted>)");
        assert_eq!(secret.expose(), "hunter2");
    }
}
