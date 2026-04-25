//! Deterministic cache path resolution. No hidden state (OP-4).

use std::path::PathBuf;

use crate::error::{Error, Result};

/// Environment variable that overrides the default cache directory root.
pub const CACHE_DIR_ENV: &str = "REPOSIX_CACHE_DIR";

/// Resolve the on-disk bare-repo path for `(backend, project)`.
///
/// Precedence:
/// 1. `REPOSIX_CACHE_DIR` env var, if set and non-empty.
/// 2. [`dirs::cache_dir`] (XDG on Linux, `~/Library/Caches` on macOS,
///    `%LOCALAPPDATA%` on Windows).
/// 3. Error if neither is available (no `$HOME`, no `XDG_CACHE_HOME`).
///
/// The returned path is `<root>/reposix/<backend>-<project>.git`.
///
/// # Errors
/// Returns [`Error::Io`] if no cache root is discoverable.
pub fn resolve_cache_path(backend: &str, project: &str) -> Result<PathBuf> {
    let root = std::env::var(CACHE_DIR_ENV)
        .ok()
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .or_else(dirs::cache_dir)
        .ok_or_else(|| {
            Error::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "no cache dir: set REPOSIX_CACHE_DIR or ensure XDG_CACHE_HOME/HOME is set",
            ))
        })?;
    // Safe filename: callers pass already-validated slugs; we do NOT re-validate here.
    Ok(root
        .join("reposix")
        .join(format!("{backend}-{project}.git")))
}

#[cfg(test)]
mod tests {
    use super::{resolve_cache_path, CACHE_DIR_ENV};

    #[test]
    fn env_var_wins() {
        let tmp = tempfile::tempdir().unwrap();
        let prev = std::env::var(CACHE_DIR_ENV).ok();
        // SAFETY: Rust 1.82 std::env::set_var is safe on POSIX hosts; we
        // restore the previous value at the end of the test.
        std::env::set_var(CACHE_DIR_ENV, tmp.path());
        let p = resolve_cache_path("sim", "proj-1").unwrap();
        assert_eq!(p, tmp.path().join("reposix").join("sim-proj-1.git"));
        match prev {
            Some(v) => std::env::set_var(CACHE_DIR_ENV, v),
            None => std::env::remove_var(CACHE_DIR_ENV),
        }
    }
}
