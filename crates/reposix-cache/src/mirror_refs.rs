//! Mirror-lag refs — observability for the SoT-vs-mirror gap.
//!
//! Two refs per SoT-host (`<sot-host>` is the `state.backend_name` slug
//! `"sim" | "github" | "confluence" | "jira"`):
//!
//! - `refs/mirrors/<sot-host>-head` — direct ref pointing at the cache's
//!   post-write synthesis-commit OID (the SHA the cache's bare repo
//!   presents to vanilla `git fetch` after a successful push to the `SoT`).
//! - `refs/mirrors/<sot-host>-synced-at` — annotated tag whose message
//!   body's first line is `mirror synced at <RFC3339>` for plain
//!   `git log` rendering.
//!
//! ## Cache vs working tree
//!
//! These refs live in **the cache's bare repo** (`Cache::repo` —
//! `<cache-root>/reposix/<backend>-<project>.git/`), NOT in the working
//! tree's `.git/`. The working tree receives them via the helper's
//! `stateless-connect` advertisement; vanilla `git fetch` from the
//! working tree pulls them across naturally.
//!
//! ## Best-effort vs unconditional
//!
//! - Ref writes are **best-effort**: callers should `tracing::warn!`
//!   on failure and continue. The push still acks `ok` to git.
//! - The companion audit row (`audit::log_mirror_sync_written`) is
//!   **unconditional** per OP-3 — it records the attempt whether or
//!   not the ref writes succeeded.
//!
//! ## Donor pattern
//!
//! This module is a copy-and-adapt of [`crate::sync_tag`]. The
//! `RefEdit` shape, the audit-write call site, and the error-mapping
//! idiom transfer 1:1. The only new wrinkle is the annotated-tag
//! object (vs. `sync_tag`'s direct ref). gix 0.83's `Repository::tag(...)`
//! hardcodes the `refs/tags/` prefix in the ref name it writes, so
//! it CANNOT be used for our `refs/mirrors/<sot>-synced-at` namespace.
//! Instead we write the tag object via [`gix::Repository::write_object`]
//! and create the ref pointing at the tag OID via
//! [`gix::Repository::reference`] (which accepts any `FullName`).
//!
//! ## Reflog growth (v0.14.0 deferral)
//!
//! Every push appends two reflog entries (one per ref edit). For
//! long-lived caches this grows unboundedly; gix doesn't auto-prune.
//! Filed as a v0.14.0 operational concern per
//! `.planning/research/v0.14.0-observability-and-multi-repo/vision-and-mental-model.md`
//! § "Operational maturity" — not P80 scope.
//!
//! ## Design intent
//! `.planning/research/v0.13.0-dvcs/architecture-sketch.md` § 2 +
//! `.planning/research/v0.13.0-dvcs/decisions.md` § "Phase-N+1
//! (mirror-lag refs) decisions" Q2.1, Q2.2, Q2.3.

use chrono::{DateTime, SecondsFormat, Utc};
use gix::refs::transaction::{Change, LogChange, PreviousValue, RefEdit, RefLog};
use gix::refs::Target;

use crate::audit;
use crate::cache::Cache;
use crate::error::{Error, Result};

/// Ref-namespace prefix for mirror-head refs.
pub const MIRROR_REFS_HEAD_PREFIX: &str = "refs/mirrors/";
/// Ref-namespace prefix for mirror-synced-at refs (same root as head;
/// the differentiator is the per-host suffix `-synced-at`).
pub const MIRROR_REFS_SYNCED_AT_PREFIX: &str = "refs/mirrors/";

/// Annotated-tag message-body prefix. The full first line is
/// `mirror synced at <RFC3339>`.
pub const SYNCED_AT_MESSAGE_PREFIX: &str = "mirror synced at ";

/// Format the full ref name for the head ref.
#[must_use]
pub fn format_mirror_head_ref_name(sot_host: &str) -> String {
    format!("{MIRROR_REFS_HEAD_PREFIX}{sot_host}-head")
}

/// Format the full ref name for the synced-at tag.
#[must_use]
pub fn format_mirror_synced_at_ref_name(sot_host: &str) -> String {
    format!("{MIRROR_REFS_SYNCED_AT_PREFIX}{sot_host}-synced-at")
}

/// Parse the annotated-tag message body to recover the RFC3339 timestamp
/// from the first line. Returns `None` if the body's first line does
/// not match `mirror synced at <RFC3339>` or the timestamp does not
/// parse cleanly.
#[must_use]
pub fn parse_synced_at_message(body: &str) -> Option<DateTime<Utc>> {
    let first_line = body.lines().next()?;
    let ts_str = first_line.strip_prefix(SYNCED_AT_MESSAGE_PREFIX)?;
    DateTime::parse_from_rfc3339(ts_str)
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
}

impl Cache {
    /// Write a direct ref at `refs/mirrors/<sot_host>-head` pointing at
    /// `sot_sha`. Returns the full ref name on success. Idempotent
    /// across identical `(sot_host, sot_sha)` pairs (`PreviousValue::Any`
    /// makes re-writes a no-op when the target is unchanged).
    ///
    /// # Errors
    /// - [`Error::Git`] if the ref name fails gix validation
    ///   (`gix::refs::FullName::try_from` rejects `..`, `:`, control
    ///   bytes) — should not occur for the controlled `state.backend_name`
    ///   enum input.
    /// - [`Error::Git`] if `edit_reference` fails (ref-store I/O,
    ///   lock contention).
    pub fn write_mirror_head(&self, sot_host: &str, sot_sha: gix::ObjectId) -> Result<String> {
        let ref_name = format_mirror_head_ref_name(sot_host);
        let full_name: gix::refs::FullName = ref_name
            .as_str()
            .try_into()
            .map_err(|e| Error::Git(format!("invalid mirror-head ref name {ref_name}: {e}")))?;
        let edit = RefEdit {
            change: Change::Update {
                log: LogChange {
                    mode: RefLog::AndReference,
                    force_create_reflog: false,
                    message: format!("reposix: mirror head sync at {}", Utc::now().to_rfc3339())
                        .into(),
                },
                expected: PreviousValue::Any,
                new: Target::Object(sot_sha),
            },
            name: full_name,
            deref: false,
        };
        self.repo
            .edit_reference(edit)
            .map_err(|e| Error::Git(format!("write mirror-head ref {ref_name}: {e}")))?;
        Ok(ref_name)
    }

    /// Write an annotated tag at `refs/mirrors/<sot_host>-synced-at` with
    /// a message body `mirror synced at <RFC3339>`. Returns the full
    /// ref name on success.
    ///
    /// The annotated tag's target is the cache's HEAD (or the most
    /// recent mirror-head ref's target if HEAD is unset) — the tag
    /// content identifies WHEN; the target is symbolic. Plain
    /// `git log refs/mirrors/<sot>-synced-at -1` renders the message
    /// body for the cold reader.
    ///
    /// Implementation note: gix 0.83's `Repository::tag(...)` hardcodes
    /// the `refs/tags/` prefix and cannot write to `refs/mirrors/...`,
    /// so we use the two-step pattern: write the tag *object* via
    /// `write_object`, then point the ref at the tag OID via
    /// `Repository::reference`.
    ///
    /// # Errors
    /// - [`Error::Git`] if there is no reachable target commit (cache
    ///   empty), the tag-object write fails, or the ref edit fails.
    pub fn write_mirror_synced_at(&self, sot_host: &str, ts: DateTime<Utc>) -> Result<String> {
        let ref_name = format_mirror_synced_at_ref_name(sot_host);
        let message = format!(
            "{SYNCED_AT_MESSAGE_PREFIX}{}",
            ts.to_rfc3339_opts(SecondsFormat::Secs, true)
        );

        // Resolve a target commit for the tag object. Prefer HEAD; fall
        // back to the mirror-head ref's target if HEAD is unset (e.g.,
        // the head ref was just written but HEAD has not been updated).
        let target_id: gix::ObjectId = if let Ok(id) = self.repo.head_id() {
            id.detach()
        } else if let Ok(mut r) = self
            .repo
            .find_reference(&format_mirror_head_ref_name(sot_host))
        {
            r.peel_to_id()
                .map_err(|e| Error::Git(format!("peel mirror-head ref for {sot_host}: {e}")))?
                .detach()
        } else {
            return Err(Error::Git(format!(
                "write mirror-synced-at ref {ref_name}: no reachable target commit (cache empty?)"
            )));
        };

        // Build the owned tagger Signature. `committer()` returns
        // `Option<Result<SignatureRef<'_>, _>>` borrowed from `&self`;
        // `to_owned()` materializes a `Signature` we can hold without
        // borrow-lifetime drama.
        let tagger: Option<gix::actor::Signature> = self
            .repo
            .committer()
            .and_then(std::result::Result::ok)
            .and_then(|sig_ref| sig_ref.to_owned().ok());

        // Construct the Tag object and write it to the object DB.
        let tag_obj = gix::objs::Tag {
            target: target_id,
            target_kind: gix::object::Kind::Commit,
            name: format!("{sot_host}-synced-at").into(),
            tagger,
            message: message.clone().into(),
            pgp_signature: None,
        };
        let tag_id = self
            .repo
            .write_object(&tag_obj)
            .map_err(|e| Error::Git(format!("write tag object {ref_name}: {e}")))?
            .detach();

        // Point refs/mirrors/<sot>-synced-at at the tag object.
        self.repo
            .reference(
                ref_name.as_str(),
                tag_id,
                PreviousValue::Any,
                format!("reposix: mirror synced-at tag for {sot_host}"),
            )
            .map_err(|e| Error::Git(format!("write tag ref {ref_name}: {e}")))?;

        Ok(ref_name)
    }

    /// Resolve `refs/mirrors/<sot_host>-synced-at` and recover the
    /// timestamp from the tag-message body's first line. Returns
    /// `None` if the ref is absent (first-push case) or if the message
    /// body fails to parse (defensive — log WARN, return `None` rather
    /// than poison the reject-hint composition path).
    ///
    /// # Errors
    /// - [`Error::Git`] if ref-store I/O fails (NOT for "ref absent" —
    ///   that case returns `Ok(None)`).
    pub fn read_mirror_synced_at(&self, sot_host: &str) -> Result<Option<DateTime<Utc>>> {
        let ref_name = format_mirror_synced_at_ref_name(sot_host);
        let Some(reference) = self
            .repo
            .try_find_reference(ref_name.as_str())
            .map_err(|e| Error::Git(format!("try_find_reference {ref_name}: {e}")))?
        else {
            return Ok(None);
        };

        // We want the *tag object* (annotated), not the peeled commit
        // it points at. `Reference::id()` returns the immediate target
        // OID without following annotated tags.
        let target_oid = match reference.target() {
            gix::refs::TargetRef::Object(oid) => oid.to_owned(),
            gix::refs::TargetRef::Symbolic(_) => {
                tracing::warn!("refs/mirrors/{sot_host}-synced-at is symbolic; treating as None");
                return Ok(None);
            }
        };
        let object = self
            .repo
            .find_object(target_oid)
            .map_err(|e| Error::Git(format!("find_object {target_oid}: {e}")))?;
        let message_body = if matches!(object.kind, gix::object::Kind::Tag) {
            let tag = object
                .try_to_tag_ref()
                .map_err(|e| Error::Git(format!("decode tag {target_oid}: {e}")))?;
            tag.message.to_string()
        } else {
            tracing::warn!(
                "refs/mirrors/{sot_host}-synced-at peeled to non-tag object kind {kind:?}; treating as None",
                kind = object.kind,
            );
            return Ok(None);
        };

        Ok(parse_synced_at_message(&message_body))
    }

    /// Audit-row companion for mirror-ref writes. UNCONDITIONAL per
    /// OP-3 — call this after the ref-write attempts whether they
    /// succeeded or not. SQL errors WARN-log; the function returns
    /// `()`.
    ///
    /// # Panics
    /// Panics if the cache's `cache.db` mutex is poisoned.
    pub fn log_mirror_sync_written(&self, oid_hex: &str, sot_host: &str) {
        let conn = self.db.lock().expect("cache.db mutex poisoned");
        let ref_pair = format!(
            "{},{}",
            format_mirror_head_ref_name(sot_host),
            format_mirror_synced_at_ref_name(sot_host),
        );
        audit::log_mirror_sync_written(
            &conn,
            &self.backend_name,
            &self.project,
            oid_hex,
            &ref_pair,
        );
    }

    /// Wrapper on [`Cache::build_from`] that names the call site for
    /// the helper's mirror-head wiring. Returns the cache's post-write
    /// synthesis-commit OID — the SHA recorded in
    /// `refs/mirrors/<sot>-head`.
    ///
    /// # Errors
    /// Mirrors [`Cache::build_from`].
    pub async fn refresh_for_mirror_head(&self) -> Result<gix::ObjectId> {
        self.build_from().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_synced_at_message_round_trips() {
        let ts: DateTime<Utc> = "2026-05-01T12:34:56Z".parse().expect("parse rfc3339");
        let body = format!(
            "{SYNCED_AT_MESSAGE_PREFIX}{}",
            ts.to_rfc3339_opts(SecondsFormat::Secs, true)
        );
        let recovered = parse_synced_at_message(&body).expect("parse should succeed");
        assert_eq!(recovered, ts);
    }

    #[test]
    fn parse_synced_at_message_returns_none_on_garbage() {
        assert!(parse_synced_at_message("").is_none());
        assert!(parse_synced_at_message("hello world").is_none());
        assert!(parse_synced_at_message("mirror synced at not-an-rfc3339").is_none());
    }

    #[test]
    fn format_mirror_head_ref_name_uses_prefix() {
        assert_eq!(format_mirror_head_ref_name("sim"), "refs/mirrors/sim-head");
        assert_eq!(
            format_mirror_head_ref_name("github"),
            "refs/mirrors/github-head"
        );
    }

    #[test]
    fn format_mirror_synced_at_ref_name_uses_prefix() {
        assert_eq!(
            format_mirror_synced_at_ref_name("sim"),
            "refs/mirrors/sim-synced-at"
        );
        assert_eq!(
            format_mirror_synced_at_ref_name("confluence"),
            "refs/mirrors/confluence-synced-at"
        );
    }

    #[test]
    fn mirror_ref_names_validate_via_gix() {
        // Positive: a controlled sot_host slug produces a valid ref name.
        let head = format_mirror_head_ref_name("sim");
        let _: gix::refs::FullName = head
            .as_str()
            .try_into()
            .expect("sim sot_host produces valid ref name");
        let synced = format_mirror_synced_at_ref_name("github");
        let _: gix::refs::FullName = synced
            .as_str()
            .try_into()
            .expect("github sot_host produces valid ref name");

        // Negative: an injected colon is rejected by gix validation.
        // (sot_host is a controlled enum in production; this test pins
        // the validation contract to gix's enforcement, not our own
        // logic — defensive against a future refactor that exposes
        // sot_host to user input.)
        let bad = format_mirror_head_ref_name("foo:bar");
        let result: std::result::Result<gix::refs::FullName, _> = bad.as_str().try_into();
        assert!(
            result.is_err(),
            "ref name with colon should fail gix validation; got {result:?}"
        );
    }
}
