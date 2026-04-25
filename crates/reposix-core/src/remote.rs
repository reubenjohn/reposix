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

/// Strip the `reposix::` prefix from a URL — tolerates a double-strip
/// edge case (`reposix::reposix::...`) defensively.
#[must_use]
pub fn strip_reposix_prefix(url: &str) -> &str {
    let mut stripped = url;
    while let Some(rest) = stripped.strip_prefix("reposix::") {
        stripped = rest;
    }
    stripped
}

/// Low-level split of a reposix remote URL into `(origin, project)` string slices.
///
/// This is the shared splitter used by both [`parse_remote_url`] and the
/// `git-remote-reposix` backend-dispatch parser. It strips an optional
/// `reposix::` prefix, locates the `/projects/` separator, and returns
/// the trimmed origin and project segments without any further
/// validation. Callers layer their own slug rules on top.
///
/// # Errors
///
/// Returns [`Error::InvalidRemote`] if the URL has no `/projects/`
/// segment, an empty origin, or an empty project segment.
pub fn split_reposix_url(url: &str) -> Result<(&str, &str)> {
    let stripped = strip_reposix_prefix(url);

    let Some(idx) = stripped.find("/projects/") else {
        return Err(Error::InvalidRemote(format!(
            "expected `/projects/<slug>` in `{stripped}`"
        )));
    };
    let origin = stripped[..idx].trim_end_matches('/');
    if origin.is_empty() {
        return Err(Error::InvalidRemote("empty origin".into()));
    }
    let tail = &stripped[idx + "/projects/".len()..];
    let project = tail.trim_end_matches('/');
    if project.is_empty() {
        return Err(Error::InvalidRemote(format!(
            "empty project segment in `{stripped}`"
        )));
    }
    Ok((origin, project))
}

/// Parse a reposix remote URL.
///
/// # Errors
/// Returns [`Error::InvalidRemote`] if the URL does not contain a `/projects/<slug>` path or if
/// the slug fails [`ProjectSlug::parse`] validation.
pub fn parse_remote_url(url: &str) -> Result<RemoteSpec> {
    let (origin, project_tail) = split_reposix_url(url)?;
    // Older callers expect the project to be a single bare slug; if the
    // tail contains a `/` (e.g. GitHub's `owner/repo`), only the leading
    // segment is taken and validated as a `ProjectSlug`. Backends that
    // need the full path-bearing form must use the lower-level
    // `split_reposix_url` directly.
    let slug_str = project_tail.split('/').next().unwrap_or("");
    let project = ProjectSlug::parse(slug_str)
        .ok_or_else(|| Error::InvalidRemote(format!("invalid project slug: `{slug_str}`")))?;
    Ok(RemoteSpec {
        origin: origin.to_owned(),
        project,
    })
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
