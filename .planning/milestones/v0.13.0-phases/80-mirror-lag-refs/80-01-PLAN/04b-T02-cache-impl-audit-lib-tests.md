← [back to index](./index.md) · phase 80 plan 01 · [← 04a](./04a-T02-cache-impl-module.md)

## Task 80-01-T02 (continued) — Audit helper + lib.rs re-export + build/test/commit

> **Split note:** this chapter covers §§ 2b–2d (audit helper, lib.rs re-export,
> build/test/commit) plus the API verification secondary path and the verify/done
> contract. The module code (§ 2a) is in
> [04a-T02-cache-impl-module.md](./04a-T02-cache-impl-module.md).

**API verification step (post-H2 fix).** The `Repository::tag(...)`
invocation above is the gix 0.83 correct shape (verified at
planning-time against the gix 0.83 source — see PLAN-CHECK.md § H2).
Run:

```bash
cargo check -p reposix-cache 2>&1 | tee /tmp/p80-t02-check.log
```

The build SHOULD succeed on the primary path. If a compile error
fires (e.g., `tagger_ref` lifetime issue tied to the
`committer().and_then(...)` borrow chain — gix exposes
`committer() -> Option<Result<SignatureRef<'_>, _>>` whose borrow is
tied to `&self`, and the `Signature::try_from(sig_ref)` step may
fail to elide cleanly), fall back to the documented **secondary
path**:

**Secondary path (two-step `write_object` + `tag_reference`).** This
is the prescribed fallback when lifetime management on `tagger_ref`
proves awkward at executor-time:

```rust
use gix::objs::Tag;

let tag_obj = Tag {
    target,
    target_kind: gix::object::Kind::Commit,
    name: format!("{sot_host}-synced-at").into(),
    tagger: tagger_owned.clone(),  // owned Signature; cleaner lifetime
    message: message.clone().into(),
    pgp_signature: None,
};
let tag_id = self
    .repo
    .write_object(&tag_obj)
    .map_err(|e| Error::Git(format!("write tag object {ref_name}: {e}")))?;
let _ref = self
    .repo
    .tag_reference(
        format!("{sot_host}-synced-at"),
        tag_id,
        PreviousValue::Any,
    )
    .map_err(|e| Error::Git(format!("write tag ref {ref_name}: {e}")))?;
```

Both paths produce the same observable contract: an annotated tag at
`refs/mirrors/<sot>-synced-at` whose `git log -1 --format=%B` first
line is `mirror synced at <RFC3339>`. The integration tests in T04
verify the contract behaviorally — the API choice doesn't affect them.
Document which path was used in T2's commit message body.

### 2b. Audit helper — `crates/reposix-cache/src/audit.rs`

Edit `crates/reposix-cache/src/audit.rs` to add `log_mirror_sync_written`
mirroring `log_sync_tag_written` (lines 340-363):

```rust
/// Insert `op='mirror_sync_written'` row — one per mirror-refs sync
/// (head + synced-at) attempted by `handle_export`'s success branch.
/// `oid_hex` is the SoT SHA written into `refs/mirrors/<sot>-head`
/// (or empty string if SHA derivation failed and the ref-write was
/// skipped — the audit row records the attempt either way per OP-3).
/// `ref_pair` is the comma-separated pair of full ref names
/// (`refs/mirrors/<sot>-head,refs/mirrors/<sot>-synced-at`).
/// Best-effort: SQL errors WARN-log.
pub fn log_mirror_sync_written(
    conn: &Connection,
    backend: &str,
    project: &str,
    oid_hex: &str,
    ref_pair: &str,
) {
    let res = conn.execute(
        "INSERT INTO audit_events_cache (ts, op, backend, project, oid, reason) \
         VALUES (?1, 'mirror_sync_written', ?2, ?3, ?4, ?5)",
        params![
            Utc::now().to_rfc3339(),
            backend,
            project,
            oid_hex,
            ref_pair,
        ],
    );
    if let Err(e) = res {
        warn!(target: "reposix_cache::audit_failure",
              backend, project, ref_pair, oid = oid_hex,
              "log_mirror_sync_written failed: {e}");
    }
}
```

(Insert immediately after `log_sync_tag_written` to keep the
ref-related audit helpers grouped.)

### 2c. lib.rs re-export

Edit `crates/reposix-cache/src/lib.rs`. Add `pub mod mirror_refs;` to
the existing pub-mod list (alphabetical: between `meta` and `path`):

```rust
pub mod audit;
pub mod builder;
pub mod cache;
pub mod db;
pub mod error;
pub mod gc;
pub mod meta;
pub mod mirror_refs;     // NEW
pub mod path;
pub mod reconciliation;
pub mod sync_tag;
```

Add the re-export immediately after `sync_tag`'s re-export (lines 52-54):

```rust
pub use mirror_refs::{
    MIRROR_REFS_HEAD_PREFIX, MIRROR_REFS_SYNCED_AT_PREFIX,
    SYNCED_AT_MESSAGE_PREFIX,
    format_mirror_head_ref_name, format_mirror_synced_at_ref_name,
    parse_synced_at_message,
};
```

### 2d. Build + test + commit

Build serially (per-crate per CLAUDE.md "Build memory budget"):

```bash
cargo check -p reposix-cache
cargo clippy -p reposix-cache -- -D warnings
cargo nextest run -p reposix-cache mirror_refs
```

If `cargo clippy` fires `clippy::pedantic` lints on the new module, fix
inline (do NOT add `#[allow(...)]` without a rationale comment per
CLAUDE.md). Each new public fn must have a `# Errors` doc.

Stage and commit:

```bash
git add crates/reposix-cache/src/mirror_refs.rs \
        crates/reposix-cache/src/audit.rs \
        crates/reposix-cache/src/lib.rs
git commit -m "$(cat <<'EOF'
feat(cache): mirror_refs module + log_mirror_sync_written audit helper (DVCS-MIRROR-REFS-01)

- crates/reposix-cache/src/mirror_refs.rs (new) — Cache::write_mirror_head + write_mirror_synced_at + read_mirror_synced_at + refresh_for_mirror_head + log_mirror_sync_written
- Pattern donor: crates/reposix-cache/src/sync_tag.rs (RefEdit shape verbatim for write_mirror_head; annotated-tag via Repository::tag for write_mirror_synced_at, fallback path documented in module if API differs at gix 0.83)
- crates/reposix-cache/src/audit.rs — log_mirror_sync_written added (mirrors log_sync_tag_written; op='mirror_sync_written'; best-effort SQL semantics)
- crates/reposix-cache/src/lib.rs — pub mod + re-exports (constants + name formatters + parse_synced_at_message)
- 4 unit tests in mirror_refs.rs #[cfg(test)] mod tests:
  - write_mirror_head_round_trips
  - write_mirror_synced_at_round_trips (skips when cache HEAD unset; integration test in T04 covers the populated case)
  - read_mirror_synced_at_returns_none_when_absent
  - mirror_ref_names_validate_via_gix (positive + negative — pins validation to gix's enforcement)
- Each new pub fn has # Errors doc; cargo clippy -p reposix-cache -- -D warnings clean
- OP-3 preserved (audit row is unconditional; ref writes are best-effort per RESEARCH.md "Wiring `handle_export` success path")

Phase 80 / Plan 01 / Task 02 / DVCS-MIRROR-REFS-01.
EOF
)"
```

<verify>
  <automated>cargo check -p reposix-cache && cargo clippy -p reposix-cache -- -D warnings && cargo nextest run -p reposix-cache mirror_refs</automated>
</verify>

<done>
- `crates/reposix-cache/src/mirror_refs.rs` exists, ≤ 250 lines.
- `crates/reposix-cache/src/audit.rs` includes `log_mirror_sync_written`
  (mirroring `log_sync_tag_written`).
- `crates/reposix-cache/src/lib.rs` declares `pub mod mirror_refs;`
  and re-exports the constants + name formatters + parser.
- `cargo check -p reposix-cache` exits 0.
- `cargo clippy -p reposix-cache -- -D warnings` exits 0.
- `cargo nextest run -p reposix-cache mirror_refs` exits 0; the 4 new
  unit tests pass (the synced_at round-trip test may early-return in a
  fresh cache without HEAD — that's acceptable; integration test in T04
  covers the populated path).
- Each new pub fn has a `# Errors` doc section.
- The Cache impl block gains `write_mirror_head`, `write_mirror_synced_at`,
  `read_mirror_synced_at`, `refresh_for_mirror_head`, `log_mirror_sync_written`
  (5 new methods).
- If gix `Repository::tag(...)` API differed at the workspace pin
  (RESEARCH.md A1), the fallback path is in place and the divergence
  is documented in the commit message.
- Cargo serialized: T02 cargo invocations run only after T01's commit
  has landed; per-crate fallback used.
</done>
