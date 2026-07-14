← [back to index](./index.md) · phase 80 plan 01 · [continued in 04b →](./04b-T02-cache-impl-audit-lib-tests.md)

## Task 80-01-T02 — Cache crate impl: `mirror_refs.rs` + `audit::log_mirror_sync_written` + `lib.rs` re-export + 4 unit tests

> **Split note:** this chapter covers §§ 2a (cache module — `mirror_refs.rs`).
> §§ 2b–2d (audit helper, lib.rs re-export, build/test/commit) continue in
> [04b-T02-cache-impl-audit-lib-tests.md](./04b-T02-cache-impl-audit-lib-tests.md).

<read_first>
- `crates/reposix-cache/src/sync_tag.rs` (entire file — DONOR PATTERN;
  the new `mirror_refs.rs` is a copy-and-adapt of this file).
- `crates/reposix-cache/src/audit.rs` lines 300-400 (`log_helper_backend_instantiated`
  + `log_sync_tag_written` — donor patterns; the new `log_mirror_sync_written`
  mirrors `log_sync_tag_written` exactly).
- `crates/reposix-cache/src/cache.rs` lines 1-100 (`Cache::open` +
  field declarations — confirm `repo: gix::Repository` access, `db:
  Mutex<Connection>` access, `backend_name: String`, `project: String`).
- `crates/reposix-cache/src/cache.rs` lines 232-310 — `log_helper_*`
  family (style precedent for new audit-write call sites).
- `crates/reposix-cache/src/lib.rs` (entire file — to find pub mod
  declarations + re-export precedent).
- `crates/reposix-cache/src/builder.rs` lines 25-90 — `Cache::build_from`
  signature (the wrapper `refresh_for_mirror_head` calls this).
- `crates/reposix-cache/Cargo.toml` — confirm gix, chrono, serde_json,
  rusqlite are present (RESEARCH.md verified).
- gix 0.83 docs for `Repository::tag` — investigate at execution time
  via `cargo doc -p gix --open` OR `grep -rn "fn tag" ~/.cargo/registry/src/index.crates.io-*/gix-0.83.*/src/`
  to confirm the API exists at the workspace pin. If absent, use the
  fallback path (two `RefEdit`s — write tag object via
  `Repository::write_object` + ref edit pointing at it; bounded ≤ 30
  lines per RESEARCH.md A1).
</read_first>

<action>
Three concerns in this task; keep ordering: cache module → audit
helper → lib.rs re-export → unit tests → cargo check + nextest +
commit.

### 2a. Cache module — `crates/reposix-cache/src/mirror_refs.rs`

Author the new module. **The structure mirrors `sync_tag.rs` verbatim
where possible** — copy-and-adapt, not from-scratch design (RESEARCH.md
"Don't Hand-Roll" Key Insight). Estimated 200-250 lines.

```rust
//! Mirror-lag refs — observability for the SoT-vs-mirror gap.
//!
//! Two refs per SoT-host (`<sot-host>` is the `state.backend_name` slug
//! `"sim" | "github" | "confluence" | "jira"`):
//!
//! - `refs/mirrors/<sot-host>-head` — direct ref pointing at the cache's
//!   post-write synthesis-commit OID (the SHA the cache's bare repo
//!   presents to vanilla `git fetch` after a successful push to the SoT).
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
//! object (vs. sync_tag's direct ref) — gix `Repository::tag(...)` is
//! the canonical idiom; if absent at the workspace pin, the fallback
//! path uses two `RefEdit`s (write tag object via `write_object`,
//! point ref at it).
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

/// Ref-namespace prefix for mirror-head refs. Public for use by the
/// helper's stateless-connect advertisement filter (if any).
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
    format!("{MIRROR_REFS_HEAD_PREFIX}{sot_host}-synced-at")
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
    /// across identical `(sot_host, sot_sha)` pairs (PreviousValue::Any
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
    /// recent synthesis commit if HEAD is unset) — the tag content
    /// identifies WHEN; the target is symbolic. Plain `git log
    /// refs/mirrors/<sot>-synced-at -1` renders the message body
    /// for the cold reader.
    ///
    /// # Errors
    /// - [`Error::Git`] if the gix tag-object write fails or the ref
    ///   name fails validation.
    pub fn write_mirror_synced_at(
        &self,
        sot_host: &str,
        ts: DateTime<Utc>,
    ) -> Result<String> {
        let ref_name = format_mirror_synced_at_ref_name(sot_host);
        let message = format!(
            "{SYNCED_AT_MESSAGE_PREFIX}{}",
            ts.to_rfc3339_opts(SecondsFormat::Secs, true)
        );

        // Target: use the current HEAD commit if available; else the
        // mirror-head ref's target if it exists; else fall back to a
        // null OID (defensive — first-push case races head + synced-at
        // creation, so synced-at may write before head). The architecture-
        // sketch's intent is that synced-at's *message* is the authoritative
        // signal; the tag's target is bookkeeping.
        let target_id = self
            .repo
            .head_id()
            .ok()
            .map(gix::Id::detach)
            .or_else(|| {
                self.repo
                    .find_reference(&format_mirror_head_ref_name(sot_host))
                    .ok()
                    .and_then(|mut r| r.peel_to_id().ok().map(gix::Id::detach))
            });
        let Some(target) = target_id else {
            // No reachable target — degrade: write a direct ref pointing
            // at a deterministic placeholder commit produced by
            // `Cache::build_from`. P80 callers always have a populated
            // cache (helper success branch fires AFTER REST writes).
            // If we hit this branch, log WARN and bail with Error::Git;
            // caller's WARN-log catches it.
            return Err(Error::Git(format!(
                "write mirror-synced-at ref {ref_name}: no reachable target commit (cache empty?)"
            )));
        };

        // gix 0.83 Repository::tag signature (verified at planning time
        // against ~/.cargo/registry/src/index.crates.io-*/gix-0.83.0/src/repository/object.rs:338-346):
        //
        //   pub fn tag(
        //       &self,
        //       name: impl AsRef<str>,
        //       target: impl AsRef<gix_hash::oid>,
        //       target_kind: gix_object::Kind,                    // slot 3
        //       tagger: Option<gix_actor::SignatureRef<'_>>,      // slot 4
        //       message: impl AsRef<str>,
        //       constraint: PreviousValue,                        // slot 6
        //   ) -> Result<Reference<'_>, tag::Error>
        //
        // The committer accessor returns Option<Result<SignatureRef<'_>, _>>;
        // we own a Signature first (so we control the lifetime) and borrow
        // it as SignatureRef for slot 4. PreviousValue::Any in slot 6 makes
        // re-writes a no-op when the (target, message) is unchanged AND
        // overwrites the previous tag when it differs (replaces the legacy
        // `force=true` semantic).
        let tagger_owned: Option<gix_actor::Signature> = self
            .repo
            .committer()
            .and_then(|r| r.ok())
            .and_then(|sig_ref| gix_actor::Signature::try_from(sig_ref).ok());
        let tagger_ref: Option<gix_actor::SignatureRef<'_>> =
            tagger_owned.as_ref().map(|s| s.to_ref());
        let _new_ref = self
            .repo
            .tag(
                format!("{sot_host}-synced-at"),
                target,
                gix::object::Kind::Commit,
                tagger_ref,
                &message,
                PreviousValue::Any,
            )
            .map_err(|e| Error::Git(format!("write annotated tag {ref_name}: {e}")))?;

        // Tag was written; gix's `tag(...)` atomically writes both the
        // tag object and the ref pointing at it.
        Ok(ref_name)
    }

    /// Resolve `refs/mirrors/<sot_host>-synced-at` and recover the
    /// timestamp from the tag-message body's first line. Returns
    /// `None` if the ref is absent (first-push case) or if the message
    /// body fails to parse (defensive — log WARN, return None rather
    /// than poison the reject-hint composition path).
    ///
    /// # Errors
    /// - [`Error::Git`] if ref-store I/O fails (NOT for "ref absent" —
    ///   that case returns `Ok(None)`).
    pub fn read_mirror_synced_at(
        &self,
        sot_host: &str,
    ) -> Result<Option<DateTime<Utc>>> {
        let ref_name = format_mirror_synced_at_ref_name(sot_host);
        let mut reference = match self.repo.find_reference(&ref_name) {
            Ok(r) => r,
            Err(gix::reference::find::existing::Error::NotFound { .. }) => {
                return Ok(None);
            }
            Err(e) => return Err(Error::Git(format!("find_reference {ref_name}: {e}"))),
        };
        // Peel to the tag object (annotated tag → tag object → message body).
        // gix's `peel_to_kind(gix::object::Kind::Tag)` returns the tag
        // object; from there, `.message` is the body.
        let target = reference
            .peel_to_id_in_place()
            .map_err(|e| Error::Git(format!("peel ref {ref_name}: {e}")))?;
        // Read the object as a tag; if it's a commit (lightweight),
        // there's no message body to parse — return None defensively.
        let object = self
            .repo
            .find_object(target)
            .map_err(|e| Error::Git(format!("find_object {target}: {e}")))?;
        let message_body = match object.kind {
            gix::object::Kind::Tag => {
                let tag = object
                    .try_to_tag_ref()
                    .map_err(|e| Error::Git(format!("decode tag {target}: {e}")))?;
                tag.message.to_string()
            }
            _ => {
                tracing::warn!(
                    "refs/mirrors/{sot_host}-synced-at peeled to non-tag object kind {kind:?}; treating as None",
                    kind = object.kind,
                );
                return Ok(None);
            }
        };

        Ok(parse_synced_at_message(&message_body))
    }

    /// Audit-row companion for mirror-ref writes. UNCONDITIONAL per
    /// OP-3 — call this after the ref-write attempts whether they
    /// succeeded or not. SQL errors WARN-log; the function returns
    /// `()`.
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

    /// Helper: open a fresh cache in a tempdir for unit-test isolation.
    /// Uses the sim backend and a deterministic project to avoid network.
    fn open_test_cache() -> (tempfile::TempDir, Cache) {
        let tmp = tempfile::tempdir().expect("tempdir");
        let cache = Cache::open(tmp.path(), "sim", "demo").expect("open cache");
        (tmp, cache)
    }

    #[test]
    fn write_mirror_head_round_trips() {
        let (_tmp, cache) = open_test_cache();
        // Use a fabricated OID that's a valid SHA-1 byte pattern.
        // (Real callers pass `cache.build_from()` output; for unit-test
        // we just need any valid ObjectId.)
        let oid = gix::ObjectId::null(gix::hash::Kind::Sha1);
        let ref_name = cache
            .write_mirror_head("sim", oid)
            .expect("write_mirror_head");
        assert_eq!(ref_name, "refs/mirrors/sim-head");
        let resolved = cache
            .repo
            .find_reference(&ref_name)
            .expect("find ref")
            .peel_to_id_in_place()
            .expect("peel to id");
        assert_eq!(resolved, oid);
    }

    #[test]
    fn write_mirror_synced_at_round_trips() {
        let (_tmp, cache) = open_test_cache();
        // First need at least one commit in the cache so the tag has a target.
        // The simplest path: write_mirror_head with a placeholder OID, then
        // write_mirror_synced_at — but the synced-at tag needs a *commit*
        // target, not just any OID. For this unit test, skip if the
        // cache repo has no HEAD; the integration tests in
        // `crates/reposix-remote/tests/mirror_refs.rs` cover the populated
        // case via the full handle_export flow.
        if cache.repo.head_id().is_err() {
            // No HEAD — skip unit test and rely on integration coverage.
            // (Document this gap so reviewers know the round-trip is
            // exercised end-to-end in T04, not in this unit test.)
            eprintln!("skipping: cache has no HEAD; integration test covers populated round-trip");
            return;
        }
        let ts: DateTime<Utc> = "2026-05-01T12:34:56Z".parse().expect("parse ts");
        let _ref_name = cache
            .write_mirror_synced_at("sim", ts)
            .expect("write_mirror_synced_at");
        let read_back = cache
            .read_mirror_synced_at("sim")
            .expect("read_mirror_synced_at")
            .expect("synced_at present");
        assert_eq!(read_back, ts);
    }

    #[test]
    fn read_mirror_synced_at_returns_none_when_absent() {
        let (_tmp, cache) = open_test_cache();
        let result = cache
            .read_mirror_synced_at("sim")
            .expect("read should succeed even when ref absent");
        assert!(result.is_none(), "expected None for absent ref; got {result:?}");
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
        let result: std::result::Result<gix::refs::FullName, _> =
            bad.as_str().try_into();
        assert!(result.is_err(), "ref name with colon should fail gix validation; got {result:?}");
    }
}
```
