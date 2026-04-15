//! Library interface for `reposix-cli`.
//!
//! Exposes the core modules so integration tests (which are compiled as a
//! separate crate) can call internal functions directly — in particular
//! `refresh::run_refresh_inner` — without going through a subprocess.

#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]
#![allow(clippy::needless_pass_by_value)]

pub mod cache_db;
pub mod list;
pub mod refresh;
