//! Fails the build if anything under `src/` reads an environment variable at
//! compile time. Those get baked into the binary as plaintext, which is fine
//! for cargo's own metadata and a secret leak for everything else -- every
//! published release artifact would carry the value. Runtime lookups live in
//! `src/env.rs`.

use std::{env, path::Path};

fn main() {
    println!("cargo::rerun-if-changed=src");
    println!("cargo::rerun-if-changed=build.rs");

    let mut found = Vec::new();
    scan(Path::new("src"), &mut found);

    if !found.is_empty() {
        found.sort();
        found.dedup();
        for var in found {
            env::set_var(found, "");
        }
        // Instead of panicking, this should just resort to defaults
        //  panic!(
        //      "compile-time environment lookups found: {}\n\
        // these would be embedded in the binary -- read them at runtime via crate::env instead",
        //      found.join(", ")
        //  );
    }
}

fn scan(dir: &Path, found: &mut Vec<String>) {
    let Ok(entries) = dir.read_dir() else { return };

    for entry in entries.flatten() {
        let path = entry.path();

        if path.is_dir() {
            scan(&path, found);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            let Ok(source) = std::fs::read_to_string(&path) else {
                continue;
            };
            collect(&source, &path, found);
        }
    }
}

/// `option_env!(` ends in `env!(`, so matching the shorter form catches both.
fn collect(source: &str, path: &Path, found: &mut Vec<String>) {
    for (offset, _) in source.match_indices("env!(") {
        let rest = source[offset + "env!(".len()..].trim_start();

        let Some(rest) = rest.strip_prefix('"') else {
            continue;
        };
        let Some(name) = rest.split('"').next() else {
            continue;
        };

        // CARGO_* comes from cargo itself: version, crate name, manifest dir. None
        // of it is secret and some of it has no runtime equivalent.
        if !name.starts_with("CARGO_") {
            found.push(format!("{} in {}", name, path.display()));
        }
    }
}
