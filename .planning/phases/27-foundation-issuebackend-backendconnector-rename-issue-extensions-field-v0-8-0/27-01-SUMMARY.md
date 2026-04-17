---
phase: 27
plan: "27-01"
subsystem: reposix-core
tags: [rename, refactor, trait, backend]
dependency_graph:
  requires: []
  provides: [BackendConnector trait in reposix-core]
  affects: [reposix-core/src/backend.rs, reposix-core/src/lib.rs, reposix-core/src/backend/sim.rs]
tech_stack:
  added: []
  patterns: [trait rename, hard remove (no backward-compat alias)]
key_files:
  created: []
  modified:
    - crates/reposix-core/src/backend.rs
    - crates/reposix-core/src/lib.rs
    - crates/reposix-core/src/backend/sim.rs
decisions:
  - "Hard rename only — no type alias IssueBackend = BackendConnector per plan instructions"
  - "Re-export in lib.rs uses alphabetical order: BackendConnector before BackendFeature"
metrics:
  duration: "~5 minutes"
  completed: "2026-04-16"
  tasks_completed: 2
  tasks_total: 2
  files_changed: 3
---

# Phase 27 Plan 01: Rename IssueBackend → BackendConnector in reposix-core Summary

**One-liner:** Hard rename of `IssueBackend` trait to `BackendConnector` in reposix-core with zero backward-compat aliases, `cargo check -p reposix-core` clean.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Rename IssueBackend → BackendConnector in backend.rs | 64eb127 | crates/reposix-core/src/backend.rs |
| 2 | Update lib.rs re-export and sim.rs impl | 64eb127 | crates/reposix-core/src/lib.rs, crates/reposix-core/src/backend/sim.rs |

## What Was Done

Renamed the `IssueBackend` trait to `BackendConnector` across the three files in `reposix-core`:

- **backend.rs**: Updated module-level doc comments (lines 1, 32, 33), trait definition, `Box<dyn>` / `Arc<dyn>` references in trait doc, dyn-compatibility test function signature, and `impl IssueBackend for Stub` in test.
- **lib.rs**: Updated `pub use` re-export from `IssueBackend` to `BackendConnector`, maintaining alphabetical order (`BackendConnector`, `BackendFeature`, `DeleteReason`).
- **backend/sim.rs**: Updated module-level doc comment, struct-level doc comment, import line, trait impl declaration, and one inline comment in the test section (line 606).

## Verification Results

```
grep -rn "IssueBackend" crates/reposix-core/src/ --include="*.rs"
# → zero matches

cargo check -p reposix-core
# → Finished `dev` profile [unoptimized + debuginfo]

grep -n "pub trait BackendConnector" crates/reposix-core/src/backend.rs
# → 123:pub trait BackendConnector: Send + Sync {

grep -n "pub use backend.*BackendConnector" crates/reposix-core/src/lib.rs
# → 21:pub use backend::{BackendConnector, BackendFeature, DeleteReason};

grep -n "impl BackendConnector for SimBackend" crates/reposix-core/src/backend/sim.rs
# → 205:#[async_trait]
# → 206:impl BackendConnector for SimBackend {
```

## Deviations from Plan

None — plan executed exactly as written. External crates (`reposix-fuse`, `reposix-remote`, `reposix-cli`, etc.) will fail to compile until Wave 2 updates their imports, which is expected per plan design.

## Known Stubs

None.

## Threat Flags

None — pure symbol rename within reposix-core; no new network, data, or trust-boundary surface introduced.

## Self-Check: PASSED

- `crates/reposix-core/src/backend.rs` exists with `pub trait BackendConnector`
- `crates/reposix-core/src/lib.rs` exists with `BackendConnector` re-export
- `crates/reposix-core/src/backend/sim.rs` exists with `impl BackendConnector for SimBackend`
- Commit 64eb127 exists in git log
