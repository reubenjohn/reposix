# Phase 98: Skeleton + shared-compute lift + edge model + walker + catalog + tracker schemas - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-08
**Phase:** 98-skeleton-shared-lift-edge-model
**Mode:** `--auto` discuss (autonomous workstream B)
**Areas discussed (auto-resolved from ADR defaults):** md_walker.rs lift API surface, heading_subtree_hash algorithm details, cross-link walk JSON output schema, catalog row kinds, tracker fixture location

---

## Mode rationale

This phase is heavily SPEC-locked through:
- 9 explicit success criteria in `.planning/milestones/v0.13.2-phases/ROADMAP.md` § "Phase 98"
- 5 named ADRs (ADR-1, ADR-19, ADR-25, ADR-27, ADR-28) in `06-decisions-log.md`
- Explicit file paths + line numbers (`coverage.rs:46`, `coverage.rs:73`, `hash.rs:29`, `hash.rs:92`)
- Explicit test counts (≥3 unit tests for `heading_subtree_hash`)
- Explicit edge-count target (350–450) for the walker
- Owner orchestrator-brief constraint that workstream B runs autonomously, with explicit pause-points only at P102 (API key) and P106 (cost budget)

Per the workflow's `<scope_guardrail>`, discussion clarifies HOW to implement, never WHETHER. Given the WHAT/WHY are locked, the gray-area surface is small enough to auto-resolve with ADR-driven defaults.

---

## Area 1: md_walker.rs lift API surface

| Option | Description | Selected |
|--------|-------------|----------|
| (a) Keep walker private to `coverage.rs`, copy-fork for cross-link | ~150 LOC duplicated; two divergent walkers in one binary | |
| (b) Lift to `crates/reposix-quality/src/md_walker.rs` (intra-crate) | Both gates inside the same crate; minimal blast radius | ✓ |
| (c) Lift to standalone `crates/reposix-md-walker/` | Cleaner for v0.13.3 extraction; adds workspace member | |

**Auto-selection rationale:** ADR-27 explicitly chooses (b) for v1, with (c) as the v0.13.3 extraction target. Captured as D-01 + D-02 + D-03 + D-04.

---

## Area 2: heading_subtree_hash algorithm

| Option | Description | Selected |
|--------|-------------|----------|
| New file, fresh hash module | Parallel to existing hash.rs | |
| Add to existing hash.rs next to source_hash + test_body_hash | Three flavors, one module, identical error handling | ✓ |

**Auto-selection rationale:** ADR-28 chosen option (b) — same-module siblings. Algorithm is pulldown-cmark 0.13 AST walk; sha256 from matched heading until next same-or-higher-level heading or EOF. Captured as D-05 + D-06 + D-07 + D-08.

---

## Area 3: `cross-link walk` JSON output schema

| Option | Description | Selected |
|--------|-------------|----------|
| Array of arrays `[[source, target, anchor], ...]` | Compact, position-dependent | |
| Array of objects `[{"source": ..., "target": ..., "anchor": ...}]` | Self-describing; jq-friendly | ✓ |

**Auto-selection rationale:** Self-describing is forward-compatible (we'll add `edge_id`, `source_line`, `anchor_line_in_target` in P101+); position-dependent arrays would break catalog assertions when fields are added. Captured as D-13.

---

## Area 4: Catalog row kinds (4 rows in P98)

| Option | Description | Selected |
|--------|-------------|----------|
| Mix of mechanical + container kinds | Container for runner conformance; mechanical for builds | |
| All `mechanical` (cadence `pre-push`) | Bottleneck rows fast & deterministic; new dimension proves itself in pre-push budget | ✓ |

**Auto-selection rationale:** P98 ships only the skeleton (no L0/L1/L2/L3 verifiers) — every catalog row at this stage is a deterministic build/walk/schema-validation check. `container` kind is overkill until L3 dispatch in P102. Captured as D-14 + D-15.

---

## Area 5: Tracker example fixture location

| Option | Description | Selected |
|--------|-------------|----------|
| `quality/state/fixtures/cross_link_tracker/` | Co-located with the production tracker | |
| `crates/reposix-quality/tests/fixtures/cross_link_tracker/` | Co-located with the test that validates them | ✓ |

**Auto-selection rationale:** Existing pattern — doc-alignment's tracker schema test fixtures live under `crates/reposix-quality/tests/`. Consistency over re-decision. Captured as D-18.

---

## Claude's Discretion

- Internal helper functions in `md_walker.rs` (path normalization, ignore-rule constants) stay `pub(crate)` unless cross-link's walker explicitly needs them.
- Test-fixture file names under `crates/reposix-quality/tests/fixtures/cross_link_tracker/` may be expanded beyond the three named in D-18 if implementation reveals more boundary cases worth pinning.
- Exact stderr/stdout split for `cross-link walk` follows the existing `doc-alignment walk` convention.

## Deferred Ideas

None new. Items the milestone ROADMAP defers from P98 are listed in CONTEXT.md `<deferred>`.

