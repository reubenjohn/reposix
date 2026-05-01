# Phase 80: Mirror-lag refs (`refs/mirrors/<sot>-head`, `<sot>-synced-at`) — Research

**Researched:** 2026-05-01
**Domain:** gix ref-writing in the cache crate; integration with the existing single-backend `handle_export` push path; agent-ux catalog rows for ref-write + vanilla-fetch + reject-message-cite tests.
**Confidence:** HIGH — almost every ingredient already has a precedent in-tree.

## Summary

Phase 80 adds two ref helpers to `crates/reposix-cache/` and one wiring point in `crates/reposix-remote/src/main.rs::handle_export`. The cache crate already has the canonical pattern for ref writing in `src/sync_tag.rs` (uses `gix::Repository::edit_reference` with a `RefEdit` + `Change::Update` + `PreviousValue::Any` for idempotent overwrite). That pattern transfers verbatim to the new helpers. The annotated tag for `synced-at` is the only new wrinkle — `tag_sync` writes a *direct ref* (target = commit OID), but the architecture sketch wants `synced-at` to be an annotated tag whose message body is the timestamp text. gix supports this via `repo.tag(...)` (creates the tag object, returns its OID) followed by a ref edit pointing `refs/mirrors/<sot>-synced-at` at the tag object.

The wiring point in `handle_export` is unambiguous: lines 470–489 (the success branch — `if !any_failure` → `proto.send_line("ok refs/heads/main")`). Both ref writes go between `log_helper_push_accepted` and `proto.send_line("ok ...")`, best-effort (audit-pattern), so a ref-write failure logs WARN and does not poison the ack. The reject path (lines 384–407 conflict, 414–432 plan errors) is touched only to compose the hint string — refs are *read* there, not written.

**Primary recommendation:** Use **gix-based ref writing** (option (a)) for both helpers, mirroring `sync_tag.rs` exactly. Keep refs in the cache's bare repo only — the working-tree clone receives them via the existing helper read path (the helper's refspec advertisement already exposes `refs/heads/*` and the helper can extend its `list` to include `refs/mirrors/*`). Catalog rows land first; tests use a `git --bare init` local mirror as the push target (option (a) for fixtures); the GH mirror smoke is `#[ignore]`-tagged for milestone-close.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Ref write (head + synced-at) | `reposix-cache` (bare repo) | — | Refs are cache state; the cache owns the gix `Repository` handle and the audit DB. |
| Wiring on push success | `reposix-remote::handle_export` | `reposix-cache` | Helper is the only place that knows a push succeeded; it calls cache helpers (mirrors `log_helper_push_accepted`). |
| Reject hint composition | `reposix-remote::handle_export` (reject branch) | `reposix-cache` (reader) | Helper composes stderr; cache supplies `read_mirror_synced_at` so the helper doesn't reach into refs directly. |
| Vanilla `git fetch` propagation | git itself (upstream) | `reposix-remote` (advertise `refs/mirrors/*` in `list`) | Plain-git fetch grabs whatever refs the upload-pack advertises; the helper's existing list path needs one additional ref-prefix entry. |

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

## Test fixture strategy

**Recommendation: option (a) — `git --bare init` local mirror for unit-style tests.**

| Option | Fits where |
|--------|-----------|
| (a) `git --bare init` local | Default for `crates/reposix-remote/tests/mirror_refs.rs` and the three verifier shells. Zero external deps; CI-stable; ~100ms per test. |
| (b) Real GH mirror `reubenjohn/reposix-tokenworld-mirror` | `#[ignore]`-tagged smoke at milestone-close. Already has DVCS-MIRROR-REPO-01 documenting the endpoint [VERIFIED: recent commit ca5164b]. |
| (c) gix in-memory mirror | REJECTED — annotated-tag round-tripping through gix-only mirrors is unproven; `git --bare init` is upstream truth. |

The single-backend push doesn't actually push to a remote git mirror in P80 — refs are written to the **cache's** bare repo. The "mirror" in the ref name is conceptual until P83 (bus push) wires the actual `git push <mirror>` step. P80 verifies refs propagate through the existing helper's `stateless-connect` advertisement; the actual mirror endpoint is not exercised until P83 + P84.

This is a **subtle point worth flagging in the plan**: the success criterion 3 ("vanilla `git fetch` brings refs along") is satisfied by fetching from the *cache via the helper*, not from a real GH mirror. The GH mirror smoke is P83/P84 territory.

## Plan splitting

**Recommendation: single plan, 4 tasks, ≤ 2 cargo-heavy.**

The phase has 6 success criteria but the work is tightly coupled (ref writers + their wiring + their tests live in the same files). Splitting fragments the catalog-row-first ritual.

### Plan 80-01 (single plan)

| Task | Type | Cargo? | Description |
|------|------|--------|-------------|
| 80-01-T01 | Catalog + verifier shells | NO | Land the three `agent-ux.json` rows + three verifier shells (failing initially per QG-06 catalog-first). Commit BEFORE any Rust changes. |
| 80-01-T02 | Cache crate impl | YES (`cargo check -p reposix-cache`) | New `mirror_refs.rs` (writer + reader); new `audit::log_mirror_sync_written`; pub mod + re-exports in lib.rs. Unit tests for writer/reader round-trip in `mirror_refs.rs`. |
| 80-01-T03 | Helper crate wiring | YES (`cargo check -p reposix-remote`) | Insert ref writes into `handle_export` success branch (lines 470–489); add `refs/mirrors/*` to ref advertisement; compose reject hint from `read_mirror_synced_at`. |
| 80-01-T04 | Integration tests + verifier flip | YES (`cargo nextest run -p reposix-remote --test mirror_refs`) | Three integration tests in `tests/mirror_refs.rs`; verifier shells now PASS; CLAUDE.md updated to document the `refs/mirrors/<sot>-{head,synced-at}` namespace; phase close push + verifier-subagent dispatch. |

Cargo invocations sequenced (per CLAUDE.md "Build memory budget"). T02 and T03 each touch one crate (`-p` flag); T04 runs nextest scoped to one test target. No workspace-wide cargo invocations in-phase — pre-push hook handles that.

## Pitfalls

1. **`PreviousValue::Any` overwrites silently.** Idempotent re-runs (good — same SHA writes a no-op edit) but a buggy caller could clobber a newer ref with an older SHA. Mitigation: callers in P80 are confined to `handle_export` success path, which only fires after a successful SoT write — the new SHA is by construction newer.
2. **Annotated tag vs. lightweight tag.** `synced-at` is annotated (per architecture sketch + ROADMAP success criterion 1); `head` is a direct ref (lightweight equivalent). Don't accidentally use the same code path for both — the message body is the whole point of `synced-at`.
3. **Helper's ref advertisement is currently scoped to `refs/heads/main` only.** Vanilla `git fetch` will NOT bring `refs/mirrors/*` along until the helper's `list` output is widened. This is the only externally-visible behavior change in P80 — needs a sentence in CLAUDE.md § Threat model (refs in this namespace are read-only metadata; no privacy implication beyond the SoT SHA, which is also visible in the working tree commit history).
4. **Cache vs. working-tree confusion.** `handle_export` writes to the cache's bare repo (where refs live); the working-tree clone receives them via the helper's read advertisement. A reviewer might expect refs to land in `<working-tree>/.git/refs/mirrors/...` directly — they do not. They land in `<cache>/refs/mirrors/...` and propagate. Document this in `mirror_refs.rs` module-doc.
5. **`sot_host` derivation.** ROADMAP says "for confluence == `confluence`" but the helper today knows `(backend_name, project)` — `state.backend_name` is `"sim" | "github" | "confluence" | "jira"`. Use `backend_name` directly for `<sot-host>` slug; rejection-message text uses the human-readable name interchangeably. Don't introduce a new "host" concept — `backend_name` is sufficient.
6. **Reflog noise.** Every push writes a reflog entry on both refs. For long-lived caches this grows unboundedly; gix doesn't auto-prune. Filed as a v0.14.0 concern (operational maturity scope per the v0.14.0 vision doc) — NOT in P80 scope. Add a one-line note in `mirror_refs.rs` so a future agent finds it.
7. **`first push` reject case.** If a reject fires before any successful push, `read_mirror_synced_at` returns `None` — the hint should omit the timestamp line cleanly, not print "synced at None ago". Test `reject_hint_cites_synced_at_with_age` covers the populated path; add a sibling test for the empty-case hint composition.
8. **Q2.2 doc-clarity contract.** ROADMAP success criterion 4 demands the verbatim Q2.2 clarification appear in user-facing docs (P85 territory). P80 carries the prose into CLAUDE.md only — full docs treatment is P85's job. Don't try to ship `dvcs-topology.md` in this phase.

## Assumptions Log

| # | Claim | Section | Risk if wrong |
|---|-------|---------|---------------|
| A1 | `gix::Repository::tag(...)` API exists at the workspace pin (gix 0.83) and creates an annotated tag object + ref atomically | Architecture Patterns / Pattern 2 | If the API differs, the planner uses two RefEdits instead (one to write the tag object via `repo.write_object()`, one to point the ref at it). Either path works; the API name only changes the implementation by ~10 lines. [ASSUMED — sync_tag.rs uses `edit_reference` only, not `tag`; planner should grep gix 0.83 docs to confirm] |
| A2 | `parsed.commits.last()` (or analogous field of `ParsedExport`) yields the SoT SHA the working-tree commit landed at | Wiring `handle_export` | If the field name differs, the planner needs to read `crates/reposix-remote/src/export_parser.rs` (or wherever `ParsedExport` is defined) for the right accessor. Mechanical fix; doesn't change the design. [ASSUMED] |
| A3 | The helper's `list` advertisement is currently limited to `refs/heads/main` and adding `refs/mirrors/*` is a one-line widening | Pitfall 3 | If the advertisement uses a different mechanism (e.g., `gix-protocol` object-info filtering), the change might be more involved — but still bounded. [ASSUMED — `crates/reposix-remote/src/stateless_connect.rs` not read in this research session] |

These assumptions are LOW-impact: each has a known fallback path (slightly different gix API; slightly different parser field name; one extra line of advertisement). None blocks the design.

## Open Questions

1. **Should `mirror_refs.rs` be a sibling of `sync_tag.rs` or extend `sync_tag.rs`?**
   - What we know: both write refs to the cache's bare repo; both audit; both use `RefEdit`.
   - What's unclear: the architecture sketch treats them as conceptually distinct (sync tags = "what did we observe at time T"; mirror refs = "what did we last successfully publish to the mirror"). The audit ops are different (`sync_tag_written` vs. `mirror_sync_written`).
   - **Recommendation:** sibling file. Keeps each concern legible; the copy-paste cost is ~30 lines.

2. **Does the helper's reject path read `synced_at` from the *cache* or from the *working tree's* `refs/mirrors/<sot>-synced-at`?**
   - The helper has the cache handle (it writes audit rows there). Reading from the cache is one-line.
   - **Recommendation:** cache. Matches the cache-as-source-of-truth contract; working tree is the consumer.

## Environment Availability

| Dependency | Required by | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| `git` | Verifier shells (vanilla-fetch test); `git --bare init` fixtures | ✓ | system (CI requires `>= 2.34` per CLAUDE.md) | — |
| `cargo` | `cargo check -p reposix-cache` / `-p reposix-remote`; `cargo nextest` for integration | ✓ | 1.82+ via rust-toolchain.toml | — |
| `reposix-sim` | Each verifier starts a sim on a unique port (mirrors `reposix-attach.sh:34`) | ✓ | local crate | — |

**Missing dependencies with no fallback:** none. **Missing dependencies with fallback:** none.

## Validation Architecture

### Test framework
| Property | Value |
|----------|-------|
| Framework | `cargo test` / `cargo nextest run` (workspace-default) |
| Config file | `.cargo/config.toml` (workspace-pinned) |
| Quick run | `cargo nextest run -p reposix-remote --test mirror_refs` |
| Full suite | `cargo nextest run --workspace` (gated by pre-push) |

### Phase requirements → test map
| Req ID | Behavior | Test type | Automated command | File exists? |
|--------|----------|-----------|-------------------|-------------|
| DVCS-MIRROR-REFS-01 | Helper APIs in `crates/reposix-cache/` | unit | `cargo nextest run -p reposix-cache mirror_refs` | ❌ Plan creates |
| DVCS-MIRROR-REFS-02 | `handle_export` updates both refs on success | integration | `cargo nextest run -p reposix-remote --test mirror_refs::write_on_success_updates_both_refs` | ❌ Plan creates |
| DVCS-MIRROR-REFS-03 | Reject messages cite both refs | integration | `cargo nextest run -p reposix-remote --test mirror_refs::reject_hint_cites_synced_at_with_age` | ❌ Plan creates |

### Sampling rate
- **Per task commit:** `cargo check -p <crate>` (per-crate, per CLAUDE.md memory budget)
- **Per task verification:** `cargo nextest run -p <crate>` scoped
- **Phase gate:** pre-push hook runs full workspace; verifier-subagent grades the three catalog rows from `quality/gates/agent-ux/mirror-refs-*.sh` artifacts

### Wave 0 gaps
- [ ] `crates/reposix-cache/src/mirror_refs.rs` — new module
- [ ] `crates/reposix-remote/tests/mirror_refs.rs` — new test file
- [ ] `quality/gates/agent-ux/mirror-refs-*.sh` — three verifier shells

## Security Domain

| ASVS category | Applies | Standard control |
|---------------|---------|-----------------|
| V2 Authentication | no | Refs are written by the helper which already holds backend creds; no new auth surface. |
| V5 Input Validation | yes | `gix::refs::FullName::try_from` validates ref names; `sot_host` slug is from the controlled `state.backend_name` enum, not user input. |
| V6 Cryptography | no | No new crypto — refs are git plumbing. |
| V8 Data Protection (`Tainted<T>`) | yes | The SoT SHA written into the head ref is itself derived from the working-tree commit (already-tainted territory). No new tainted-byte path; ref content is the SHA only, which is metadata, not body bytes. |

### Threat patterns
| Pattern | STRIDE | Mitigation |
|---------|--------|------------|
| Malicious ref name injection | Tampering | gix `FullName::try_from` rejects `..`, `:`, control bytes; `sot_host` derived from enum (`backend_name`), not user input. |
| Stale `synced-at` misread as "current SoT state" | Information Disclosure (misleading) | Q2.2 verbatim doc clarification carried into CLAUDE.md in same PR; full docs treatment in P85. |
| Reflog growth on long-lived caches | DoS (disk) | Filed as v0.14.0 operational concern; not P80 scope. |

## Project Constraints (from CLAUDE.md)

- **Build memory budget:** per-crate cargo only; no parallel cargo invocations across subagents.
- **Catalog-first:** the three `agent-ux.json` rows + verifier shells land in T01, BEFORE T02's Rust impl.
- **Per-phase push:** phase closes with `git push origin main` BEFORE verifier-subagent dispatch.
- **CLAUDE.md update in same PR:** `refs/mirrors/<sot>-{head,synced-at}` namespace + Q2.2 clarification get a paragraph (not a narrative) inserted into the Architecture or Threat-model section.
- **OP-1 simulator-first:** all verifier shells use `reposix-sim`; real-mirror smoke is `#[ignore]`-tagged.
- **OP-3 audit non-optional:** new `mirror_sync_written` audit op; best-effort WARN-on-fail consistent with sibling helpers.
- **OP-7 verifier-subagent on phase close:** dispatch reads catalog rows from artifacts only.
- **OP-8 +2 reservation:** items < 1hr / no new dep get fixed in-phase; else `SURPRISES-INTAKE.md` or `GOOD-TO-HAVES.md`.

## Sources

### Primary (HIGH confidence)
- `/home/reuben/workspace/reposix/.planning/research/v0.13.0-dvcs/architecture-sketch.md` § "2. Mirror-lag observability via plain-git refs" (lines 52–82) and § "3. Bus remote ... step 8" (line 130–131).
- `/home/reuben/workspace/reposix/.planning/research/v0.13.0-dvcs/decisions.md` § "Phase-N+1 (mirror-lag refs) decisions" (lines 20–30) — Q2.1/Q2.2/Q2.3 ratified.
- `/home/reuben/workspace/reposix/.planning/REQUIREMENTS.md` lines 65–68 — DVCS-MIRROR-REFS-01..03 verbatim.
- `/home/reuben/workspace/reposix/.planning/ROADMAP.md` Phase 80 (lines 83–101) — success criteria.
- `/home/reuben/workspace/reposix/crates/reposix-cache/src/sync_tag.rs` — donor pattern for ref writing.
- `/home/reuben/workspace/reposix/crates/reposix-cache/src/audit.rs` lines 340–363 (`log_sync_tag_written`) — donor pattern for new audit row.
- `/home/reuben/workspace/reposix/crates/reposix-remote/src/main.rs` lines 280–491 (`handle_export`) — wiring point.
- `/home/reuben/workspace/reposix/crates/reposix-cache/Cargo.toml` — dependency confirmation (gix, chrono, rusqlite all present).
- `/home/reuben/workspace/reposix/quality/gates/agent-ux/reposix-attach.sh` — verifier-shell shape.
- `/home/reuben/workspace/reposix/quality/catalogs/agent-ux.json` — `agent-ux/reposix-attach-against-vanilla-clone` row shape.

### Secondary (MEDIUM confidence)
- `crates/reposix-cache/src/db.rs` lines 1–120 — confirmed `audit_events_cache` table schema applies; new `op` value drops in without DDL changes.
- `crates/reposix-remote/src/main.rs` lines 309–314 (`ensure_cache` → `state.cache.as_ref()`) — confirmed cache handle is available in `handle_export`'s success branch.

### Tertiary (LOW confidence)
- `gix::Repository::tag(...)` API existence at gix 0.83 — assumed standard; planner verifies in T02.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — every dependency already in cache `Cargo.toml`.
- Architecture: HIGH — `sync_tag.rs` is a near-isomorphic donor.
- Wiring point: HIGH — line numbers verified by direct read of `handle_export`.
- Catalog row design: HIGH — `reposix-attach.sh` is a shipped precedent (P79).
- Annotated tag implementation: MEDIUM — `Repository::tag` API name assumed; falls back to two `RefEdit`s if needed.
- Test fixture choice: HIGH — `git --bare init` is the obvious cheap path; #[ignore] smoke matches v0.13.0 milestone-close pattern.

**Research date:** 2026-05-01
**Valid until:** 2026-05-31 (30 days — stable APIs; gix 0.83 just bumped in P78 so unlikely to drift mid-milestone).
