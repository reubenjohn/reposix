//! Shared types for the reposix workspace.
//!
//! This crate is the seam between the simulator (in-process REST API), the FUSE daemon, the git
//! remote helper, and the CLI orchestrator. Every other crate depends on it; it depends on no
//! other internal crate. Keep it small and stable.

#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

mod error;
pub mod http;
mod issue;
pub mod path;
mod project;
mod remote;
mod taint;

pub use error::{Error, Result};
pub use issue::{frontmatter, Issue, IssueId, IssueStatus};
pub use project::{Project, ProjectSlug};
pub use remote::{parse_remote_url, RemoteSpec};
pub use taint::{sanitize, ServerMetadata, Tainted, Untainted};
