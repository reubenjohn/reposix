//! Helper for resolving sibling binary paths.
//!
//! The CLI spawns `reposix-sim` and `reposix-fuse` as child processes. We
//! want to resolve them relative to `current_exe()` (i.e. the
//! `target/{debug,release}/` sibling of `reposix`) rather than via `$PATH`,
//! so a malicious binary earlier on `$PATH` cannot spoof either child
//! (T-03-08 spoofing mitigation).
//!
//! Fallback: if the sibling is missing (dev-loop environments where only
//! `reposix` got built), return a `cargo run -q -p <crate> --` command.
//! This is slower on the first invocation but keeps the demo
//! reproducible from a fresh clone.

use std::path::{Path, PathBuf};
use std::process::Command;

/// Build a [`Command`] that invokes `name` (one of `reposix-sim` or
/// `reposix-fuse`). Prefers the sibling of `current_exe`; falls back to
/// `cargo run -q -p <crate> --`.
pub fn resolve_bin(name: &str) -> Command {
    // Case 1: production binary next to `reposix` itself
    //   (e.g. `target/debug/reposix` → `target/debug/<name>`).
    // Case 2: test harness binary one level below
    //   (e.g. `target/debug/deps/cli-<hash>` → `target/debug/<name>`).
    // Case 3: `CARGO_TARGET_DIR` explicitly set.
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            for candidate in candidates(parent, name) {
                if candidate.exists() {
                    return Command::new(candidate);
                }
            }
        }
    }
    if let Ok(dir) = std::env::var("CARGO_TARGET_DIR") {
        for subdir in ["debug", "release"] {
            let candidate = PathBuf::from(&dir).join(subdir).join(name);
            if candidate.exists() {
                return Command::new(candidate);
            }
        }
    }
    // Fallback: cargo run. The binary names (`reposix-sim` /
    // `reposix-fuse`) are also the crate package names, so we pass `name`
    // through directly.
    let mut cmd = Command::new("cargo");
    cmd.args(["run", "-q", "-p", name, "--"]);
    cmd
}

fn candidates(parent: &Path, name: &str) -> Vec<PathBuf> {
    let mut out = vec![parent.join(name)];
    // `target/debug/deps/` → `target/debug/<name>`.
    if let Some(grandparent) = parent.parent() {
        out.push(grandparent.join(name));
    }
    out
}
