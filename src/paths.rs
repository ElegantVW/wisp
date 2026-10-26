//! Config lives under the invoking user's home. Secrets never in the tree.

use std::path::PathBuf;

pub fn home() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

pub fn siren_candidates() -> Vec<PathBuf> {
    let h = home();
    vec![
        std::env::var_os("SIREN_BIN")
            .map(PathBuf::from)
            .unwrap_or_default(),
        h.join(".local/lib/faeos/siren"),
        PathBuf::from("/usr/local/lib/faeos/siren"),
        h.join("siren/target/release/siren"),
        h.join("bin/siren"),
    ]
}
