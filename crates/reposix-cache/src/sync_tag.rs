//! Time-travel via git tags — sync-point refs in the cache's bare repo.
//!
//! Every successful [`crate::Cache::sync`] writes a deterministic ref of the
//! form `refs/reposix/sync/<ISO8601-no-colons>` pointing at the synthesis
//! commit produced by that sync. Combined with the `last_fetched_at` audit
//! trail, this gives a fully replayable history of what reposix observed
//! from the backend — `git checkout refs/reposix/sync/<ts>` (in the cache's
//! bare repo) prints "what did this issue look like at <ts>".
//!
//! The namespace `refs/reposix/sync/...` is private to the cache. The
//! helper's refspec is `refs/heads/*:refs/reposix/*`; `git upload-pack`
//! does NOT advertise refs outside `refs/heads/`, `refs/tags/`,
//! `refs/notes/` by default, so these tags are NEVER exposed to the
//! agent's working tree. The export refspec also doesn't propagate them
//! (only `refs/heads/main` is in the helper's `list` advertisement).
//!
//! ISO8601 colons are illegal inside git ref names; we substitute `-` for
//! `:`. The tag string `2026-04-25T01-13-00Z` round-trips to a `DateTime<Utc>`
//! via [`parse_sync_tag_timestamp`].
//!
//! Design intent: `.planning/research/v0.11.0-vision-and-innovations.md` §3b.

use chrono::{DateTime, SecondsFormat, Utc};
use gix::refs::transaction::{Change, LogChange, PreviousValue, RefEdit, RefLog};
use gix::refs::Target;

use crate::audit;
use crate::cache::Cache;
use crate::error::{Error, Result};

/// Ref-namespace prefix for sync tags. Private to the cache; not in the
/// helper's `list` advertisement nor in the export refspec.
pub const SYNC_TAG_PREFIX: &str = "refs/reposix/sync/";

/// One sync tag — a `(name, timestamp, commit)` triple.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncTag {
    /// Full ref name, e.g. `refs/reposix/sync/2026-04-25T01-13-00Z`.
    pub name: String,
    /// Parsed UTC timestamp (round-trip from the ref name).
    pub timestamp: DateTime<Utc>,
    /// Synthesis commit the tag points at.
    pub commit: gix::ObjectId,
}

/// List sync tags from a bare-repo path WITHOUT going through `Cache::open`.
///
/// `Cache::open` requires a `BackendConnector` instance; introspection
/// commands (`reposix history`, `reposix at`) only need read access to the
/// refs and don't have a backend handle. This free function is the
/// dependency-free path.
///
/// Tags are returned sorted by timestamp ascending. Refs that don't
/// round-trip via [`parse_sync_tag_timestamp`] are silently skipped.
///
/// # Errors
/// - [`Error::Git`] if the bare repo cannot be opened or its ref store
///   cannot be iterated.
pub fn list_sync_tags_at(cache_path: &std::path::Path) -> Result<Vec<SyncTag>> {
    let repo = gix::open(cache_path)
        .map_err(|e| Error::Git(format!("open bare repo at {}: {e}", cache_path.display())))?;
    let platform = repo
        .references()
        .map_err(|e| Error::Git(format!("open ref iter: {e}")))?;
    let iter = platform
        .prefixed("refs/reposix/sync/")
        .map_err(|e| Error::Git(format!("iter sync tags: {e}")))?;
    let mut tags: Vec<SyncTag> = Vec::new();
    for r in iter {
        let mut reference = r.map_err(|e| Error::Git(format!("read ref: {e}")))?;
        let name = reference.name().as_bstr().to_string();
        let Some(slug) = name.strip_prefix(SYNC_TAG_PREFIX) else {
            continue;
        };
        let Some(ts) = parse_sync_tag_timestamp(slug) else {
            continue;
        };
        let oid = reference
            .peel_to_id()
            .map_err(|e| Error::Git(format!("peel sync tag {name}: {e}")))?
            .detach();
        tags.push(SyncTag {
            name,
            timestamp: ts,
            commit: oid,
        });
    }
    tags.sort_by_key(|t| t.timestamp);
    Ok(tags)
}

/// Format a `DateTime<Utc>` as an ISO8601 ref-safe slug (colons replaced
/// with `-`). The reverse direction is [`parse_sync_tag_timestamp`].
#[must_use]
pub fn format_sync_tag_slug(ts: DateTime<Utc>) -> String {
    // RFC-3339 with second precision and trailing Z; replace `:` with `-`
    // to satisfy git ref-name validation (`gix_validate::reference::name`
    // rejects `:`).
    ts.to_rfc3339_opts(SecondsFormat::Secs, true)
        .replace(':', "-")
}

/// Parse an ISO8601 ref-safe slug (e.g. `2026-04-25T01-13-00Z`) back to a
/// `DateTime<Utc>`. Returns `None` if the slug doesn't round-trip cleanly.
#[must_use]
pub fn parse_sync_tag_timestamp(slug: &str) -> Option<DateTime<Utc>> {
    // Reverse the colon substitution: position [13] and [16] are the only
    // colons in an RFC-3339 second-precision UTC string `YYYY-MM-DDTHH-MM-SSZ`.
    // Length check first to avoid panics on short input.
    if slug.len() < 20 {
        return None;
    }
    let bytes = slug.as_bytes();
    if bytes[13] != b'-' || bytes[16] != b'-' {
        return None;
    }
    // Substitute `-` back to `:` at indices 13 and 16. We work in a `String`
    // by char index — both positions are single-byte ASCII so substitution
    // by character is well-defined.
    let restored: String = slug
        .chars()
        .enumerate()
        .map(|(i, c)| {
            if (i == 13 || i == 16) && c == '-' {
                ':'
            } else {
                c
            }
        })
        .collect();
    DateTime::parse_from_rfc3339(&restored)
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
}

impl Cache {
    /// Write a sync tag at `refs/reposix/sync/<ts>` pointing at `commit`.
    /// Returns the full ref name. Also writes a `op='sync_tag_written'`
    /// audit row (best-effort).
    ///
    /// Idempotent across identical `(commit, ts)` pairs: the underlying
    /// `RefEdit` uses `PreviousValue::Any`, so re-tagging the same
    /// timestamp is a no-op (same target). A different target at the
    /// same timestamp overwrites — sync-point timestamps are the
    /// authoritative key.
    ///
    /// # Errors
    /// - [`Error::Git`] if `edit_reference` fails (validation error,
    ///   lock contention, etc.).
    ///
    /// # Panics
    /// Panics if the cache's `cache.db` mutex is poisoned.
    pub fn tag_sync(&self, commit: gix::ObjectId, ts: DateTime<Utc>) -> Result<String> {
        let slug = format_sync_tag_slug(ts);
        let ref_name = format!("{SYNC_TAG_PREFIX}{slug}");
        let full_name: gix::refs::FullName = ref_name
            .as_str()
            .try_into()
            .map_err(|e| Error::Git(format!("invalid sync tag name {ref_name}: {e}")))?;
        let edit = RefEdit {
            change: Change::Update {
                log: LogChange {
                    mode: RefLog::AndReference,
                    force_create_reflog: false,
                    message: format!("reposix: sync tag at {slug}").into(),
                },
                expected: PreviousValue::Any,
                new: Target::Object(commit),
            },
            name: full_name,
            deref: false,
        };
        self.repo
            .edit_reference(edit)
            .map_err(|e| Error::Git(format!("write sync tag {ref_name}: {e}")))?;

        // Audit (best-effort).
        {
            let conn = self.db.lock().expect("cache.db mutex poisoned");
            audit::log_sync_tag_written(
                &conn,
                &self.backend_name,
                &self.project,
                &ref_name,
                &commit.to_hex().to_string(),
            );
        }

        Ok(ref_name)
    }

    /// List all sync tags in the cache's bare repo, sorted by timestamp
    /// ascending.
    ///
    /// Refs whose name doesn't round-trip via [`parse_sync_tag_timestamp`]
    /// are silently skipped — defensive against stray refs in the namespace.
    ///
    /// Equivalent to [`list_sync_tags_at`] applied to `self.repo_path()`.
    ///
    /// # Errors
    /// - [`Error::Git`] if iteration over the ref store fails.
    pub fn list_sync_tags(&self) -> Result<Vec<SyncTag>> {
        list_sync_tags_at(self.repo_path())
    }

    /// Find the latest sync tag whose timestamp is `<= target`.
    /// Returns `None` if no such tag exists (target predates every tag).
    ///
    /// # Errors
    /// Mirrors [`Cache::list_sync_tags`].
    pub fn sync_tag_at(&self, target: DateTime<Utc>) -> Result<Option<SyncTag>> {
        let mut tags = self.list_sync_tags()?;
        // Reverse-iterate (latest first), return first that satisfies <=.
        tags.sort_by_key(|t| t.timestamp);
        Ok(tags.into_iter().rev().find(|t| t.timestamp <= target))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slug_round_trip() {
        let ts: DateTime<Utc> = "2026-04-25T01:13:00Z".parse().unwrap();
        let slug = format_sync_tag_slug(ts);
        assert_eq!(slug, "2026-04-25T01-13-00Z");
        let back = parse_sync_tag_timestamp(&slug).unwrap();
        assert_eq!(back, ts);
    }

    #[test]
    fn parse_rejects_short_or_malformed() {
        assert!(parse_sync_tag_timestamp("nope").is_none());
        assert!(parse_sync_tag_timestamp("2026-04-25T01:13:00Z").is_none());
        assert!(parse_sync_tag_timestamp("2026-04-25Tzz-13-00Z").is_none());
    }
}
