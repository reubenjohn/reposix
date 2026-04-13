//! Project (top-level container of issues) types.

use serde::{Deserialize, Serialize};

/// A URL- and path-safe project identifier (e.g. `"demo"`, `"PROJ-A"`).
///
/// Validated to contain only `[A-Za-z0-9._-]` and to be 1-64 chars long. This guarantees we can
/// safely render it as a directory name in the FUSE mount without path-traversal risk.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ProjectSlug(String);

impl ProjectSlug {
    /// Parse a slug, returning `None` if it contains disallowed characters or is a path
    /// traversal sentinel (`.`, `..`).
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        if s.is_empty() || s.len() > 64 || s == "." || s == ".." {
            return None;
        }
        if s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.'))
        {
            Some(Self(s.to_owned()))
        } else {
            None
        }
    }

    /// Borrow the slug as a `&str`.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ProjectSlug {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// A project: a named container of issues, with a configured workflow and permissions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    /// Stable URL-safe identifier.
    pub slug: ProjectSlug,
    /// Human-readable display name.
    pub name: String,
    /// Free-form description shown to agents in `index.md`.
    #[serde(default)]
    pub description: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slug_accepts_safe_chars() {
        assert!(ProjectSlug::parse("demo").is_some());
        assert!(ProjectSlug::parse("PROJ-123").is_some());
        assert!(ProjectSlug::parse("my.project_v2").is_some());
    }

    #[test]
    fn slug_rejects_path_traversal() {
        assert!(ProjectSlug::parse("..").is_none());
        assert!(ProjectSlug::parse("a/b").is_none());
        assert!(ProjectSlug::parse("a\0b").is_none());
        assert!(ProjectSlug::parse("").is_none());
    }
}
