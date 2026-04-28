//! `reposix-quality` library: shared catalog + hash modules used by the
//! umbrella binary, the standalone `hash_test_fn` binary, and the
//! integration tests.
//!
//! Self-contained — does not depend on any other reposix crate. A future
//! standalone spinoff of this dimension verifier suite is one `cargo init`
//! away. See:
//!   `.planning/research/v0.12.0-docs-alignment-design/02-architecture.md`
//!   `.planning/research/v0.12.0-docs-alignment-design/05-p64-infra-brief.md`

#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]
// Pass-by-value matches clap-derive call sites cleanly.
#![allow(clippy::needless_pass_by_value)]
// Subcommand modules contain documentation strings on the verb fns.
#![allow(clippy::missing_errors_doc)]

pub mod catalog;
pub mod commands;
pub mod coverage;
pub mod hash;

pub use catalog::{Catalog, NextAction, Row, RowState, Source, Summary};
