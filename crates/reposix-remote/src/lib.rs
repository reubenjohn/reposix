//! Library surface for the `reposix-remote` package.
//!
//! The package ships the `git-remote-reposix` binary (`src/main.rs`), but
//! it also exposes [`backend_dispatch`] as a library module so sibling
//! crates — notably `reposix-cli`'s `attach` and `sync --reconcile`
//! subcommands — can reuse the one mature URL→connector factory instead
//! of re-deriving credential handling and the OP-3 `.with_audit(…)`
//! wiring a third and fourth time (D91-03).
//!
//! Only [`backend_dispatch`] is published: the binary's protocol,
//! fast-import, and bus modules stay private to `main.rs`.
#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod backend_dispatch;
