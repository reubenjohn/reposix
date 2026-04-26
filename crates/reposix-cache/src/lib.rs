//! `reposix-cache` — backing bare-repo cache built from REST responses.
//!
//! This crate is the substrate for the git-native architecture pivot
//! (v0.9.0). It materializes `BackendConnector` responses into a real
//! on-disk bare git repo:
//!
//! - **Tree sync = full.** Every `Cache::build_from` call lists all
//!   issues and writes a tree object with one entry per issue. Tree
//!   metadata is cheap.
//! - **Blob materialization = lazy.** Blobs are NOT written during
//!   `build_from`; only `Cache::read_blob(oid)` persists a blob to
//!   `.git/objects`. This is the whole point — the cache is a partial-
//!   clone promisor, and writing all blobs upfront would defeat the
//!   lazy invariant.
//!
//! Audit log, tainted-byte discipline, and egress allowlist enforcement
//! are wired in via the `audit` module and the shared
//! `reposix_core::http::client()` factory.
//!
//! ## Environment variables
//! - `REPOSIX_CACHE_DIR` — overrides the default cache directory
//!   (`$XDG_CACHE_HOME/reposix/`).
//! - `REPOSIX_ALLOWED_ORIGINS` — egress allowlist, honored transitively
//!   via `reposix_core::http::client()` which backend adapters use.

#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod audit;
pub mod builder;
pub mod cache;
pub mod db;
pub mod error;
pub mod gc;
pub mod meta;
pub mod path;
pub mod sync_tag;

/// Privileged-sink stubs used to lock the Tainted-vs-Untainted
/// discipline at compile time (see `tests/compile-fail/`). The module
/// is `#[doc(hidden)]` so it does not appear in the rendered API.
#[doc(hidden)]
pub mod sink;

pub use builder::SyncReport;
pub use cache::Cache;
pub use error::{Error, Result};
pub use gc::{gc_at, EvictedBlob, GcReport, GcStrategy, DEFAULT_MAX_AGE_DAYS, DEFAULT_MAX_SIZE_MB};
pub use path::{resolve_cache_path, CACHE_DIR_ENV};
pub use sync_tag::{
    format_sync_tag_slug, list_sync_tags_at, parse_sync_tag_timestamp, SyncTag, SYNC_TAG_PREFIX,
};
