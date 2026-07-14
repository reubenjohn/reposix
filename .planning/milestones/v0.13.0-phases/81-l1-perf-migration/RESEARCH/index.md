# Phase 81: L1 perf migration — `list_changed_since`-based conflict detection — Research

**Researched:** 2026-05-01
**Domain:** git remote helper conflict detection; cache delta sync; CLI surface extension
**Confidence:** HIGH for trait + cache state inventory; MEDIUM for net algorithm shape (one architectural surface needs planner decision — see §3 "delete detection")

## Summary

Phase 81 replaces the unconditional `state.backend.list_records(...)` call in `handle_export` (currently around line 334–348 of `crates/reposix-remote/src/main.rs` — POST P80 the line numbers shifted; the call site is unchanged in shape but accompanied by P80's mirror-ref writes after acceptance). The substrate is already in place:

- **`BackendConnector::list_changed_since(project, since) -> Vec<RecordId>`** exists on the trait with a default impl and is overridden by all four shipped backends (`sim`, `confluence`, `github`, `jira`) using native incremental queries (`?since=`, CQL `lastModified > "..."`, JQL `updated >= "..."`).
- **`Cache::sync()`** in `crates/reposix-cache/src/builder.rs` already implements the L1 algorithm end-to-end against its OWN cursor (`meta.last_fetched_at`): read cursor → `list_changed_since` → eager-materialize changed blobs → rebuild full tree → atomic SQL transaction. Phase 81's helper-side precheck reuses the same surfaces.
- **`meta.last_fetched_at`** is already the canonical cursor row in `cache.db`. P80's `refs/mirrors/<sot>-synced-at` is a SEPARATE timestamp (last successful OUTBOUND mirror sync) and must not be conflated with the INBOUND SoT-sync cursor used here.

**Primary recommendation:** Single plan, four tasks (Task 1: catalog rows; Task 2: helper precheck rewrite + delete-detection seam decision; Task 3: `reposix sync --reconcile` subcommand; Task 4: N=200 perf regression test + CLAUDE.md doc update + L2/L3 inline comment). The one decision the planner cannot punt on is the **delete-detection seam** in §3 — `list_changed_since` returns IDs of changed records but does NOT signal backend-side deletions on Confluence (nothing to find via `lastModified > x`). The recommended path is "L1 trusts the cache for the prior set; deletes are detected the way today's `plan()` does, but against `cache.list_record_ids()` instead of a freshly-fetched `list_records()`." Documented inline as a known L2/L3-hardening surface.

## Chapters

- **[Architecture](./architecture.md)** — Responsibility map, standard stack, system diagram, project structure, cursor read/write patterns, anti-patterns, don't-hand-roll table.
- **[Common Pitfalls](./pitfalls.md)** — Seven pitfalls: first-push fallback, clock skew, delete-detection seam (L1 caveat), cache write failure, prior-blob parse cost, async boundary, tainted-byte leak.
- **[Code Examples](./code-examples.md)** — Rust code for the new precheck loop and `reposix sync --reconcile` CLI subcommand.
- **[Catalog Row Design and Test Fixture Strategy](./catalog-and-tests.md)** — Three catalog rows; wiremock N=200 regression test with `expect(0)` assertion.
- **[Plan Splitting, Risks, and Documentation Deferrals](./plan-and-risks.md)** — 4-task breakdown, risk table, documentation deferrals.
- **[Sources, Metadata, and Open Questions](./sources-and-questions.md)** — Sources, confidence breakdown, validity window, five open questions for the planner.
