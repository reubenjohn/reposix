---
phase: 33
plan: 01
title: "BackendConnector::list_changed_since — trait method + 4 backend impls"
wave: 1
depends_on: []
requirements: [ARCH-06]
files_modified:
  - crates/reposix-core/src/backend.rs
  - crates/reposix-core/src/backend/sim.rs
  - crates/reposix-sim/src/routes/issues.rs
  - crates/reposix-github/src/lib.rs
  - crates/reposix-confluence/src/lib.rs
  - crates/reposix-jira/src/lib.rs
autonomous: true
mode: standard
---

# Phase 33 Plan 01 — `list_changed_since` trait + backend impls

<objective>
Add `BackendConnector::list_changed_since(project, since) -> Result<Vec<IssueId>>`
as a default-provided trait method, teach the simulator's `GET /projects/:slug/issues`
handler to honor a `?since=<ISO8601>` query parameter, and override the method
on `SimBackend`, `GithubReadOnlyBackend`, `ConfluenceBackend`, and `JiraBackend`
to use each backend's native incremental query. Returns IDs only (not full
`Issue`s) so callers (the cache) can decide whether to materialize eagerly or
lazily — symmetric with the Phase 31 lazy-blob design.
</objective>

<must_haves>
- `list_changed_since` is dyn-compatible (no generics, no `Self: Sized`).
- Default impl returns `list_issues().into_iter().filter(|i| i.updated_at > since).map(|i| i.id).collect()` — belt-and-suspenders for backends without native `since`.
- Sim route: absent `since` → returns full set (backwards compatible); present `since` → filters in SQL `WHERE updated_at > ?`.
- Malformed `since` on the sim returns HTTP 400 (not 500).
- All `impl` methods return `Vec<IssueId>` (not `Vec<Issue>`) to avoid wasteful body payloads across the wire for backends with native `since`.
- Every network path runs through `reposix_core::http::client()` (SG-01 allowlist applies automatically).
</must_haves>

<canonical_refs>
- `.planning/phases/33-.../33-CONTEXT.md` §Trait method signature (locked).
- `.planning/research/v0.9-fuse-to-git-native/architecture-pivot-summary.md` §4 Sync and Conflict Model.
- `crates/reposix-core/src/backend.rs` — existing trait definition; extend here.
- `crates/reposix-core/src/backend/sim.rs` — `SimBackend` lives here.
- `crates/reposix-sim/src/routes/issues.rs:145` — `list_issues` sim route.
- `crates/reposix-github/src/lib.rs:344-410` — GitHub `impl BackendConnector`.
- `crates/reposix-confluence/src/lib.rs:1476-1508` — Confluence trait impl + `list_issues_impl` at line 813.
- `crates/reposix-jira/src/lib.rs:522-538` — JIRA trait impl + `list_issues_impl` at line 246.
</canonical_refs>

## Chapters

- [T01 — Add `list_changed_since` to `BackendConnector` trait with default impl](./T01-trait-default-impl.md)
- [T02 — Sim route: accept `?since=<ISO8601>` query parameter](./T02-sim-route.md)
- [T03 — `SimBackend::list_changed_since` — pass `?since=` on the wire](./T03-sim-backend.md)
- [T04 — `GithubReadOnlyBackend::list_changed_since` — native `?since=` param](./T04-github.md)
- [T05 — `ConfluenceBackend::list_changed_since` — CQL `lastModified > "<iso>"`](./T05-confluence.md)
- [T06 — `JiraBackend::list_changed_since` — JQL `updated >= "<iso>"`](./T06-jira.md)
- [T07 — Workspace-wide gate: clippy + tests](./T07-gate.md)
