//! Remote URL parsing.
//!
//! Reposix git remote URLs take the form `reposix::<scheme>://<host>[:port]/projects/<slug>` —
//! e.g. `reposix::http://localhost:7777/projects/demo`. The `reposix::` prefix is stripped by
//! git before invoking the helper, but we accept either form for ergonomics in CLI flows.

use serde::{Deserialize, Serialize};

use crate::{error::Result, Error, ProjectSlug};

/// A parsed remote URL pointing at a specific project on a reposix-compatible backend.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteSpec {
    /// Origin (scheme + host + optional port), e.g. `http://localhost:7777`.
    pub origin: String,
    /// Project slug, e.g. `demo`.
    pub project: ProjectSlug,
}

/// Parse a reposix remote URL.
///
/// # Errors
/// Returns [`Error::InvalidRemote`] if the URL does not contain a `/projects/<slug>` path or if
/// the slug fails [`ProjectSlug::parse`] validation.
pub fn parse_remote_url(url: &str) -> Result<RemoteSpec> {
    let stripped = url.strip_prefix("reposix::").unwrap_or(url);

    // Find `/projects/` segment.
    let Some(idx) = stripped.find("/projects/") else {
        return Err(Error::InvalidRemote(format!(
            "expected `/projects/<slug>` in `{stripped}`"
        )));
    };
    let origin = stripped[..idx].trim_end_matches('/').to_owned();
    if origin.is_empty() {
        return Err(Error::InvalidRemote("empty origin".into()));
    }
    let tail = &stripped[idx + "/projects/".len()..];
    let slug_str = tail.trim_end_matches('/').split('/').next().unwrap_or("");
    let project = ProjectSlug::parse(slug_str)
        .ok_or_else(|| Error::InvalidRemote(format!("invalid project slug: `{slug_str}`")))?;
    Ok(RemoteSpec { origin, project })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_with_prefix() {
        let r = parse_remote_url("reposix::http://localhost:7777/projects/demo").unwrap();
        assert_eq!(r.origin, "http://localhost:7777");
        assert_eq!(r.project.as_str(), "demo");
    }

    #[test]
    fn parses_without_prefix() {
        let r = parse_remote_url("https://api.example.com/projects/PROJ-A").unwrap();
        assert_eq!(r.origin, "https://api.example.com");
        assert_eq!(r.project.as_str(), "PROJ-A");
    }

    #[test]
    fn rejects_path_traversal_slug() {
        assert!(parse_remote_url("http://x/projects/..").is_err());
    }
}
