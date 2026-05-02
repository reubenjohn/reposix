[index](./index.md)

# 9. Milestone Impact

## Current state

- **v0.9.0** was planned as "Docs IA and Narrative Overhaul" (Phase 30).
- Phase 30 documents the FUSE-based architecture -- user-facing docs, README rewrite, architecture diagrams.

## The problem

If Phase 30 ships as planned, it documents an architecture that will be immediately obsolete. Every diagram, every CLI example, every "how it works" section would reference FUSE mounts, `reposix mount`, and the virtual filesystem model.

## The decision

- **v0.9.0 becomes the architecture pivot milestone.** The FUSE-to-partial-clone migration is the primary deliverable.
- **Phase 30 (docs) is deferred to v0.10.0**, where it will document the NEW architecture (partial clone, `reposix init`, git-native workflow).
- **Estimated scope for v0.9.0:** 3-5 new phases covering:
  1. `reposix-cache` crate (bare-repo cache construction from REST responses).
  2. `stateless-connect` capability in `git-remote-reposix` (protocol-v2 tunnel to cache).
  3. `list_changed_since()` on `BackendConnector` + delta sync integration.
  4. CLI pivot (`reposix init` replacing `reposix mount`) + blob limit enforcement.
  5. Integration testing (round-trip: partial clone + edit + push + conflict detection).
  6. Delete `crates/reposix-fuse/` and all FUSE dependencies.

## What this buys

- Docs written once for the final architecture, not rewritten after the pivot.
- The architecture pivot is scoped as a milestone, not a drive-by refactor.
- Each phase can be independently tested and verified before the next begins.
