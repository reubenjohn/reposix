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

## Chapters

- [Implementation — Standard Stack, Architecture Patterns, Don't Hand-Roll, Wiring, Catalog rows](./chapter-implementation.md)
- [Testing, Constraints, and Security](./chapter-testing-and-constraints.md)
