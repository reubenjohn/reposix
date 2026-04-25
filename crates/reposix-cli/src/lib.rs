//! Library interface for `reposix-cli`.
//!
//! Exposes the core modules so integration tests (which are compiled as a
//! separate crate) can call internal functions directly — in particular
//! `refresh::run_refresh_inner` — without going through a subprocess.

#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]
#![allow(clippy::needless_pass_by_value)]

/// Re-export shim: `cache_db` module moved to
/// [`reposix_cache::cli_compat`] in Phase 31 Plan 02. The alias keeps
/// `reposix_cli::cache_db::{...}` working for any external caller (and
/// for the in-crate `refresh` subcommand) without re-writing imports.
pub use reposix_cache::cli_compat as cache_db;

pub mod list;
pub mod refresh;
pub mod spaces;
