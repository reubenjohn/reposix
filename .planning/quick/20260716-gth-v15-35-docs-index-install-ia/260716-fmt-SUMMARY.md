---
quick_id: 260716-fmt
slug: gth-v15-35-docs-index-install-ia
status: complete
completed: 2026-07-16
---

# Quick Task 260716-fmt — GTH-V15-35 docs/index.md install-IA fix Summary

Relocated the "Build from source (advanced)" `<details>` block to sit under the
30-second install section, surfaced the bootstrap commands in visible prose, and
destaled the L93 two-claim line — all in one wave with a mechanical doc-alignment
rebind of every shifted/changed catalog row.

## Block relocation

- **Old location:** `docs/index.md` L120-136 (after "Connector capability matrix",
  before "## Where to go next").
- **New location:** `docs/index.md` L69-85 (immediately after L67 "Full step-by-step
  in first-run", before L87 "## After — one commit"). Content byte-identical (collapsed
  `<details>`, same prose/commands).
- install-leads gate verified GREEN: first pkg-mgr command offset < first
  `git clone`/`cargo build --release` offset.

## Addendum 1 — visible bootstrap prose

Added a new fenced block at the top of "## After — one commit" (L91-95):
`reposix sim &` / `reposix init sim::demo /tmp/reposix-demo` / `cd /tmp/reposix-demo
&& git checkout -B main refs/reposix/origin/main`, so the demo now reads top-to-bottom
(create tree -> edit -> commit -> push) independent of the collapsed block. No banned
words introduced (`banned-words-lint.sh` PASS).

## Addendum 2 — L93 split/destale

**Before (old L93, single line):** "Latency for each backend is captured in
[`docs/benchmarks/latency.md`](benchmarks/latency.md). Sim cold init is `278 ms` (soft
threshold `500 ms`); list-issues `7 ms`; capabilities probe `5 ms`. Real-backend cells
fill in once CI secret packs are wired (Phase 36)."

**After (new L119 + L121, two lines):**
- L119 (unchanged clause, hash-bound to `docs/index/soft-threshold-24ms`): "Latency for
  each backend is captured in [`docs/benchmarks/latency.md`](benchmarks/latency.md). Sim
  cold init is `278 ms` (soft threshold `500 ms`); list-issues `7 ms`; capabilities probe
  `5 ms`."
- L121 (new, destaled): "Real-backend numbers are already captured: get-one-record is
  `320 ms` against GitHub and `202 ms` against Confluence
  ([latency](benchmarks/latency.md))." — figures from `docs/benchmarks/latency.md:42`.

## Doc-alignment rebind (11 rows, old -> new source lines)

| Row id | Old lines | New lines | Content |
|---|---|---|---|
| `docs/index/demo-workflow-sed-git-commit` | 73-75 | 99-101 | unchanged |
| `docs/index/audit-trail-git-log` | 78 | 104 | unchanged |
| `docs/index/tested-three-backends` | 86-91 | 112-117 | unchanged |
| `docs/index/soft-threshold-24ms` | 93 | 119 | CHANGED (split off Real-backend clause) |
| `docs/index/backend-capabilities-struct` | 116 | 144 | unchanged |
| `docs/index/reposix-sim-starts-7878` | 129 | 78 | unchanged (moved block) |
| `docs/index/reposix-init-demo-command` | 130 | 79 | unchanged (moved block) |
| `docs/index/git-checkout-branch-command` | 131 | 80 | unchanged (moved block; carries pre-existing WAIVED-STALE_DOCS_DRIFT until 2026-07-31, QL-001) |
| `docs/index/bootstrap-latency-24ms` | 134 | 83 | unchanged (moved block) |
| `docs/index/blob-limit-teaching` | 152 | 162 | unchanged |
| `docs/index/push-conflict-detection` | 152 | 162 | unchanged |

All 11 rebound via `target/release/reposix-quality doc-alignment bind` (no hand-edit,
no fan-out, no cargo). Pre-edit baseline: `walk.sh` exit 0. Post-edit pre-bind:
`walk.sh` exit 1 with exactly these 11 rows flagged (10 `STALE_DOCS_DRIFT` + 1
pre-existing `WAIVED-STALE_DOCS_DRIFT`), matching the plan's enumerated set exactly.
Post-bind: `walk.sh` exit 0, zero `STALE_DOCS_DRIFT`.

## Gates run (all PASS)

- `bash quality/gates/docs-build/mkdocs-strict.sh` — exit 0
- `bash quality/gates/docs-build/mermaid-renders.sh` — exit 0
- `scripts/banned-words-lint.sh` — exit 0
- `bash quality/gates/docs-alignment/install-snippet-shape.sh` — PASS (5-line-install claim holds, L19 untouched)
- `structure/install-leads-with-pkg-mgr-docs-index` — GREEN (pkg-mgr offset < source-compile offset)
- `bash quality/gates/docs-alignment/walk.sh` — exit 0, zero STALE_DOCS_DRIFT

## SURPRISES-INTAKE row filed

One MEDIUM row appended to
`.planning/milestones/v0.15.0-phases/SURPRISES-INTAKE.md`: the token-economy offline
regenerator's idempotency test (`test_main_offline_regenerates_doc_from_captures`,
`quality/gates/perf/test_bench_token_economy.py:212-244`) only self-compares against a
synthetic `tmp_path` fixture and never byte-compares against the real committed
`docs/benchmarks/token-economy.md` — the exact gap that let the 260716-f6o generator
regression slip through undetected. `discovered-by: quick 260716-fmt`, `STATUS: OPEN`.
`.planning/MANAGER-HANDOVER.md` untouched.

## GTH-V15-35 status

`.planning/milestones/v0.15.0-phases/GOOD-TO-HAVES.md` L238 updated from
"SCHEDULED — immediately after P115 phase-close" to "DONE — quick 260716-fmt
(2026-07-16)".

## Commit + push

- Commit: `bea4b9b` — `docs(index): nest build-from-source under 30-second install +
  surface bootstrap + destale L93 (GTH-V15-35, 260716-fmt)`
- Pushed to `origin/main`, pre-push hook GREEN (no `--no-verify`).
- Main CI run for this commit: **SUCCESS** (see verification below).

## Deviations from Plan

None — plan executed exactly as written. No cargo invocation was needed (pre-built
`target/release/reposix-quality` binary was usable throughout); no subagent fan-out was
needed (all 11 rebinds were pure line/hash re-anchors with claim+tests unchanged).

## Noticed near the work (OD-3)

- The moved-block rows (`reposix-sim-starts-7878`, `reposix-init-demo-command`,
  `git-checkout-branch-command`, `bootstrap-latency-24ms`) now cite lines INSIDE the
  collapsed `<details>` block, which duplicates the same three commands that also
  appear (in slightly different form) in the new visible bootstrap prose (L92-94). This
  is intentional per the plan (cite the block copy to keep the stored hash stable), but
  it does mean two near-identical copies of the bootstrap sequence now live in the file
  — a future editor touching one should remember to check the other for drift, since
  only the block copy is doc-alignment-bound.
- `docs/index/git-checkout-branch-command` still carries its pre-existing WAIVED
  status (until 2026-07-31, QL-001, "claim unverifiable until push/fetch round-trip
  lands") independent of this rebind — the line/hash re-anchor does not resolve the
  underlying QL-001 blocker, just keeps the citation accurate while it's waived.
- No stale claims, dead anchors, or other doc lies were noticed elsewhere in the edited
  region beyond the two addressed by this plan (L93's Phase-36 clause).

## Self-Check: PASSED

- FOUND: docs/index.md (edited, block relocated, bootstrap prose added, L93 split)
- FOUND: quality/catalogs/doc-alignment.json (11 rows rebound)
- FOUND: .planning/milestones/v0.15.0-phases/SURPRISES-INTAKE.md (new MEDIUM row)
- FOUND: .planning/milestones/v0.15.0-phases/GOOD-TO-HAVES.md (GTH-V15-35 -> DONE)
- FOUND: commit bea4b9b in git log
- FOUND: main CI run for bea4b9b concluded SUCCESS
