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

use std::path::PathBuf;
use std::process::Command;

/// Build a [`Command`] that invokes `name` (one of `reposix-sim` or
/// `reposix-fuse`). Prefers the sibling of `current_exe`; falls back to
/// `cargo run -q -p <crate> --`.
pub fn resolve_bin(name: &str) -> Command {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let sibling: PathBuf = parent.join(name);
            if sibling.exists() {
                return Command::new(sibling);
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
