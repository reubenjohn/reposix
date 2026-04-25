//! Library interface for `reposix-cli`.
//!
//! Exposes every subcommand module so integration tests (which compile as
//! a separate crate) can call internal functions directly — in particular
//! `refresh::run_refresh_inner`, `doctor::run`, `gc::run`, etc. — without
//! going through a subprocess.
//!
//! All subcommand modules live here; `main.rs` is intentionally thin and
//! contains only the clap-derive dispatch shim.

#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]
#![allow(clippy::needless_pass_by_value)]

pub mod binpath;
pub mod cache_db;
pub mod doctor;
pub mod gc;
pub mod history;
pub mod init;
pub mod list;
pub mod refresh;
pub mod sim;
pub mod spaces;
pub mod tokens;
pub mod worktree_helpers;
