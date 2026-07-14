← [back to index](./index.md)

# Implementation — Standard Stack, Architecture Patterns, Wiring, and Catalog Rows

## Standard Stack

### Core (already in `crates/reposix-cache/Cargo.toml`)

| Library | Version | Purpose | Why standard |
|---------|---------|---------|--------------|
| `gix` | 0.83 (workspace `=`-pinned, P78 bump) | Bare-repo handle, ref edits, tag-object creation | Already the cache's git layer; `sync_tag.rs` is the precedent. [VERIFIED: crate Cargo.toml line 25; CLAUDE.md § Tech stack] |
| `chrono` | workspace | `DateTime<Utc>` formatting + parsing for `synced-at` message body and "(N minutes ago)" arithmetic | Already used for sync-tag slug generation [VERIFIED: cache/Cargo.toml line 19, sync_tag.rs uses it] |
| `rusqlite` | 0.32 (bundled) | Audit row insert (`mirror_sync_written` op) | Already the audit-row writer [VERIFIED: cache/Cargo.toml line 20] |

**Nothing new to install.** All dependencies are already in `reposix-cache/Cargo.toml`. [VERIFIED: file read]

### Wiremock / git fixture (already in dev-dependencies)

| Library | Version | Purpose | When |
|---------|---------|---------|------|
| `tempfile` | 3 | Per-test cache + bare-mirror dirs | Existing pattern (sync_tag tests, attach tests) |
| Test git binary | system (`>= 2.34`) | Bare-init + push fixture for vanilla-fetch test | CI already requires git for partial-clone tests |

## Architecture Patterns

### System data flow (Phase 80 wiring)

```
                 git push  (existing single-backend path)
                       │
                       ▼
crates/reposix-remote/src/main.rs::handle_export
   │  ├── parse export stream                       (lines 318–332)
   │  ├── list_records prior + conflict check       (lines 334–407)
   │  ├── plan + execute REST writes                (lines 409–463)
   │  └── ★ on success branch (lines 470–489):
   │         log_helper_push_accepted               (existing)
   │         ★ cache.write_mirror_head(sot_sha)     ← NEW (Phase 80)
   │         ★ cache.write_mirror_synced_at(now)    ← NEW (Phase 80)
   │         proto.send_line("ok refs/heads/main")  (existing)
   │
   ▼
crates/reposix-cache/src/mirror_refs.rs (NEW)
   ├── write_mirror_head(sot_sha, sot_host) → ref edit at refs/mirrors/<sot>-head
   ├── write_mirror_synced_at(ts, sot_host) → tag object + ref edit at refs/mirrors/<sot>-synced-at
   ├── read_mirror_synced_at(sot_host)      → Option<DateTime<Utc>> for hint composition
   └── audit row: op='mirror_sync_written'
```

### Recommended file layout (delta from current state)

```
crates/reposix-cache/src/
├── mirror_refs.rs   ← NEW — ref writers/readers; mirror sync_tag.rs shape
├── sync_tag.rs      (unchanged; donor pattern)
├── audit.rs         (one new fn: log_mirror_sync_written)
└── lib.rs           (one new pub mod + re-export)

quality/gates/agent-ux/
├── mirror-refs-write-on-success.sh        ← NEW (catalog row 1)
├── mirror-refs-readable-by-vanilla-fetch.sh ← NEW (catalog row 2)
└── mirror-refs-cited-in-reject-hint.sh    ← NEW (catalog row 3)

quality/catalogs/agent-ux.json              ← +3 rows (BEFORE impl, per QG-06)
```

### Pattern 1: gix `RefEdit` for direct refs (the head ref)
Source: `crates/reposix-cache/src/sync_tag.rs:153–190` (`Cache::tag_sync`).

```rust
// write_mirror_head — direct ref pointing at the SoT's main SHA.
let ref_name = format!("refs/mirrors/{sot_host}-head");
let full: gix::refs::FullName = ref_name.as_str().try_into()
    .map_err(|e| Error::Git(format!("invalid ref name {ref_name}: {e}")))?;
let edit = RefEdit {
    change: Change::Update {
        log: LogChange { mode: RefLog::AndReference, force_create_reflog: false,
            message: format!("reposix: mirror head sync at {now}").into() },
        expected: PreviousValue::Any,                  // idempotent overwrite
        new: Target::Object(sot_sha),                  // direct ref → commit
    },
    name: full, deref: false,
};
self.repo.edit_reference(edit).map_err(|e| Error::Git(e.to_string()))?;
```

### Pattern 2: annotated tag for `synced-at` (gix `repo.tag(...)`)
Annotated tags require a tag *object* (with message body). gix's `Repository::tag(name, target_id, message, force)` is the canonical API: it creates the tag object, writes the ref to point at the tag OID, and returns the new tag's OID. The tag's message body is what `git log refs/mirrors/<sot>-synced-at -1` displays via the upstream "tag" log format.

Recommended message body shape (per success criterion 1 of ROADMAP P80):

```
mirror synced at 2026-05-01T17:30:00Z
```

One human-readable line. No JSON blob — `git log` rendering of annotated tags is plain-text, and a JSON envelope would be hostile to the cold-reader UX the refs exist to provide. Future structured fields can be added as additional `key: value` lines (RFC 822 style) without breaking the first-line contract.

### Anti-patterns to avoid
- **Shelling out to `git update-ref` / `git tag -a`** (option (b)). Adds an exec dependency, complicates error surfaces (parse stderr), and is unrepresented elsewhere in `reposix-cache`. The crate is gix-native; stay gix-native. [REJECTED]
- **Raw filesystem writes to `.git/refs/mirrors/...`** (option (c)). Bypasses gix's reflog handling, breaks ref locking on concurrent writes, and would not produce annotated tag *objects*. [REJECTED]
- **Storing refs only in the working-tree clone, not the cache.** The cache is the bare repo the helper serves from. The working tree fetches from the helper. Refs MUST live in the cache; the working tree gets them via the existing fetch advertisement. [LOCKED by architecture]

## Don't Hand-Roll

| Problem | Don't build | Use instead | Why |
|---------|-------------|-------------|-----|
| Ref name validation | regex / manual byte checks | `gix::refs::FullName::try_from(&str)` | gix already enforces `gix_validate::reference::name` rules (no `:`, no `..`, etc.) [VERIFIED: sync_tag.rs:158 uses this idiom] |
| Annotated tag object creation | hand-build tag-object byte buffer | `gix::Repository::tag(...)` | The crate exists. [CITED: gix docs — `Repository::tag` API] |
| Audit-row writing | inline `conn.execute()` | new `log_mirror_sync_written` in `audit.rs` mirroring `log_sync_tag_written` | Best-effort WARN-on-failure pattern is uniform across the crate [VERIFIED: audit.rs:340–363] |
| "(N minutes ago)" formatting | manual arithmetic | `chrono::Utc::now().signed_duration_since(synced_at)` then format minutes/hours | Standard chrono idiom; cache already uses chrono [VERIFIED] |

**Key insight:** `sync_tag.rs` is a near-isomorphic prior. The new `mirror_refs.rs` should be a copy-and-adapt of that file — not a from-scratch design. Reviewers will recognize the shape; the audit-row pattern, the `RefEdit` struct, the error-mapping idiom all transfer 1:1.

## Runtime State Inventory

> Phase 80 is greenfield ref-writing — not a rename or migration. No existing runtime state needs auditing for old-name embeddings.

| Category | Items found | Action required |
|----------|-------------|------------------|
| Stored data | None — refs are NEW namespace | None |
| Live service config | None | None |
| OS-registered state | None | None |
| Secrets/env vars | None | None |
| Build artifacts | None | None |

**Verified:** the `refs/mirrors/...` namespace does not exist in any current cache; no migration step needed. First push after P80 ships writes both refs from scratch.

## Wiring `handle_export` success path — exact line citations

[VERIFIED: read of `crates/reposix-remote/src/main.rs:280–491`]

- **Lines 300–311:** `handle_export` opens the cache lazily (`ensure_cache`) — the ref helpers will reuse this same handle.
- **Line 312–314:** `log_helper_push_started` precedent — cache audit row write at the start of the push.
- **Lines 470–489 (the success branch):** the wiring point. Insert the two ref writes BETWEEN `log_helper_push_accepted` (line 471) and `proto.send_line("ok refs/heads/main")` (line 486).
- **Reject paths (lines 384–407, 414–432, 464–469):** ref state is *read* (not written). Compose hint string by calling `cache.read_mirror_synced_at(sot_host)`; if `None`, omit the hint cleanly (first push case).

**SoT SHA derivation** for `write_mirror_head`: the `parsed` fast-import stream's commit OID is what the SoT's `main` will point at after this push. The helper does NOT itself update `refs/heads/main` in the cache (the cache's main is the synthesis tree, not the working-tree commit) — but the *SoT-mirror* ref records the SHA the GH mirror will end up at after the corresponding `git push` to the mirror happens (in P83 bus path; in P80 single-backend, conceptually the same SHA the working-tree commit landed at, derived from `parsed.commits.last()` or equivalent — the planner needs to confirm which field of `ParsedExport` holds this).

**Best-effort vs. hard error.** Per Q2.2 ("refs lag the SoT — that's the point"), a ref-write failure does NOT poison the push. The semantics: SoT write succeeded; the *observability* of that success may be slightly stale. Match the existing audit-write pattern: WARN on error, return ok to git anyway. NO new error variant — the helper's existing `Error` type already wraps `cache::Error`, which the WARN path drops via `let _ = ...`. [PATTERN-VERIFIED: audit.rs §"Audit-row write failures log WARN via tracing::warn but do NOT poison the caller's flow"]

**Audit row.** New op: `mirror_sync_written`. Schema fits existing `audit_events_cache` shape (op + backend + project + reason + oid). `oid` = the SoT SHA written; `reason` = the ref name pair (`refs/mirrors/<sot>-head,refs/mirrors/<sot>-synced-at`). Add `log_mirror_sync_written` to `audit.rs` mirroring `log_sync_tag_written` (lines 340–363).

## Reject-message hint composition

```rust
// New cache reader
pub fn read_mirror_synced_at(&self, sot_host: &str) -> Result<Option<DateTime<Utc>>>
// Resolves refs/mirrors/<sot>-synced-at to the tag object, reads the
// message body's first line, parses RFC3339. None if ref is absent.
```

**Hint string template** (mirrors success criterion 4 verbatim from ROADMAP):
```
hint: your origin (GH mirror) was last synced from confluence at <ts> (<N> minutes ago)
hint: run `reposix sync` to update local cache from confluence directly, then `git rebase`
```

`chrono` confirmed in cache `Cargo.toml` line 19. [VERIFIED]

## Catalog row design (lands FIRST per QG-06)

Three rows in `quality/catalogs/agent-ux.json` mirroring the shape of `agent-ux/reposix-attach-against-vanilla-clone` (which is the P79 precedent — `_provenance_note` carries the same hand-edit flag because `reposix-quality bind` only supports the docs-alignment dimension at v0.13.0).

### Row 1: `agent-ux/mirror-refs-write-on-success`
- **Verifier:** `quality/gates/agent-ux/mirror-refs-write-on-success.sh`
- **Test target:** `crates/reposix-remote/tests/mirror_refs.rs::write_on_success_updates_both_refs`
- **Invariant:** after `cargo run -p reposix-cli -- init sim::demo /tmp/T && cd /tmp/T && <commit> && git push`, the cache's bare repo has both `refs/mirrors/sim-head` and `refs/mirrors/sim-synced-at` resolvable; `git log refs/mirrors/sim-synced-at -1 --format=%B` matches `mirror synced at <RFC3339>`.

### Row 2: `agent-ux/mirror-refs-readable-by-vanilla-fetch`
- **Verifier:** `quality/gates/agent-ux/mirror-refs-readable-by-vanilla-fetch.sh`
- **Test target:** `crates/reposix-remote/tests/mirror_refs.rs::vanilla_fetch_brings_mirror_refs`
- **Invariant:** after a single-backend push, a fresh `git clone` of the cache's bare repo (or `git fetch` from an existing clone) brings `refs/mirrors/<sot>-head` and `refs/mirrors/<sot>-synced-at` along; `git for-each-ref refs/mirrors/` returns both.
- **Note:** vanilla `git fetch` brings refs only if the helper's `list` advertisement includes `refs/mirrors/*`. The current helper advertises only `refs/heads/main` (per `crates/reposix-remote/src/stateless_connect.rs` — the planner should verify the exact constant). One-line addition to the advertisement is part of P80 scope.

### Row 3: `agent-ux/mirror-refs-cited-in-reject-hint`
- **Verifier:** `quality/gates/agent-ux/mirror-refs-cited-in-reject-hint.sh`
- **Test target:** `crates/reposix-remote/tests/mirror_refs.rs::reject_hint_cites_synced_at_with_age`
- **Invariant:** after a successful push (refs populated) followed by a SECOND push with a stale prior, the conflict-reject stderr contains both `refs/mirrors/sim-synced-at` and a parseable RFC3339 timestamp + `(N minutes ago)`.

Each verifier shell follows the `reposix-attach.sh` shape: `cargo build -p reposix-cli`, start sim on a unique port, `mktemp -d` working tree, run the scenario, assert via `git for-each-ref` / `git log` / stderr grep. ~50 lines each.
