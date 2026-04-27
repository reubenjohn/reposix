<!-- quality/catalogs/README.md — schema spec for Quality Gates catalogs. -->
<!-- ≤200 lines. New schema fields require a v0.X.Y migration; do not extend ad-hoc. -->

# Quality Gates catalogs — schema spec

## What is a catalog?

A catalog is the data layer of a Quality Gates dimension. Each row is one
`(gate, verifier, expected-outcome)` triple. The runner (`quality/runners/run.py`)
reads catalogs, runs the verifiers within timeout, and writes per-row JSON
artifacts. The verdict generator (`quality/runners/verdict.py`) reads
artifacts and emits the human-readable verdict + a shields.io endpoint
badge JSON.

Catalogs are pure JSON. No comments, no logic. New gates land here as
data, not code; verifier implementations live under `quality/gates/<dim>/`.

## Unified schema

Every catalog row across every dimension uses this exact shape:

| Field | Required? | Notes |
| --- | --- | --- |
| `id` | yes | stable slug, never reused; `<dimension>/<slug>` form |
| `dimension` | yes | one of: `code`, `docs-build`, `docs-repro`, `release`, `structure`, `agent-ux`, `perf`, `security` |
| `cadence` | yes | one of: `pre-push`, `pre-pr`, `weekly`, `pre-release`, `post-release`, `on-demand` |
| `kind` | yes | one of: `mechanical`, `container`, `asset-exists`, `subagent-graded`, `manual` |
| `sources` | recommended | file:line refs to the doc/code surface this gate is about; lets the verifier flag drift |
| `command` | for docs-repro / install rows | the literal command a user runs |
| `expected.asserts` | yes | concrete predicates ("exit 0", "stdout matches /.../", "file X exists with mode 0755") |
| `verifier.script` | yes | path under `quality/gates/<dim>/` |
| `verifier.args` | optional | command-line args list |
| `verifier.timeout_s` | yes | runtime budget; runner kills + records FAIL on overage |
| `verifier.container` | for container-kind | image name (e.g., `ubuntu:24.04`) |
| `artifact` | yes | path the verifier writes; runner uses for `last_verified` + verdict |
| `status` | yes | runner updates after each verify; not hand-edited |
| `last_verified` | yes | RFC3339 UTC; null until first verify |
| `freshness_ttl` | for subjective/manual | duration string ("14d", "30d"); expired rows flip to NOT-VERIFIED; null for mechanical |
| `blast_radius` | yes | `P0`, `P1`, or `P2` — drives verdict severity routing |
| `owner_hint` | recommended | one-liner naming where the FIX likely lives |
| `waiver` | nullable | the principled escape hatch (see PROTOCOL §waivers); shape `{until: RFC3339, reason: str, dimension_owner: str, tracked_in: str}` |

The catalog file itself wraps these rows in a small metadata object:

```jsonc
{
  "$schema": "https://json-schema.org/draft-07/schema#",
  "comment": "Quality Gates catalog — <dim>. Schema: quality/catalogs/README.md.",
  "dimension": "<dim>",
  "rows": [ <row1>, <row2>, ... ]
}
```

## Per-dimension catalog files

| File | Owner dimension | Source-of-truth seed |
| --- | --- | --- |
| `freshness-invariants.json` | structure | migrated from `scripts/end-state.py` freshness rows + new QG-08 + BADGE-01 |
| `docs-reproducible.json` | docs-repro | `.planning/docs_reproducible_catalog.json` (P59) |
| `release-assets.json` | release | populated in P58 from RELEASE-04 |
| `perf-targets.json` | perf | stub in v0.12.0; full in v0.12.1 |
| `subjective-rubrics.json` | (cross-dimension) | seeded with hero-clarity, install-positioning, headline-numbers (P61) |
| `orphan-scripts.json` | (meta) | waivers for scripts that genuinely resist absorption |

## Status legend

The runner sets exactly one of the following on each row after every run:

| Status | Meaning |
| --- | --- |
| `PASS` | Verifier exited 0 AND every `expected.asserts` predicate matched. The row is GREEN. |
| `FAIL` | Verifier exited non-zero, or one or more `expected.asserts` predicates failed. The row is RED. P0+P1 FAIL blocks GREEN verdicts. |
| `PARTIAL` | Verifier exited 2 (mixed result) — some asserts matched, others did not. Treated as RED for blast_radius P0+P1. |
| `NOT-VERIFIED` | No artifact dated this session, or artifact predates `freshness_ttl`. The row is RED for grading purposes; the runner did not produce a determination. |
| `WAIVED` | An in-scope `waiver` block applies and `waiver.until` is in the future. The row is treated as GREEN for verdict purposes; the verifier is not invoked. Waiver expiry flips the row back to FAIL on next verify. |

## SIMPLIFY-03 boundary statement (per-FILE catalog vs per-CHECK catalog)

This directory's catalogs answer the question: **"What quality gates are
GREEN?"** — one row per check, one verifier per row, one artifact per
verification.

The script `scripts/catalog.py` answers a different question: **"What
files in this v0.X session are KEEP / TODO / DONE / REVIEW / DELETE /
REFACTOR?"** — one row per FILE, one disposition per file, no verifier.
It reads `.planning/v0.11.1-catalog.json` (and similar per-milestone
files) and emits a markdown report.

Initial assessment per SIMPLIFY-03: keep `scripts/catalog.py` separate;
do NOT fold. Rationale: per-FILE catalog and per-CHECK catalog answer
different questions; conflating them would degrade both. The per-FILE
catalog is a planning aid (where do I focus this session?); the
per-CHECK catalog is an enforcement aid (what regressed?). They share
no fields, no consumers, and no lifecycle. If a verifier-runner cohesion
emerges in v0.13.x or later, this boundary may be revisited; until then
each remains canonical for its own domain.

SIMPLIFY-03 closes with this README boundary doc. `scripts/catalog.py`
stays in place.

## Cross-references

- `quality/PROTOCOL.md` — autonomous-mode runtime contract that consumes these catalogs.
- `.planning/research/v0.12.0-naming-and-architecture.md` — schema design rationale.
- `.planning/research/v0.12.0-vision-and-mental-model.md` — dimension/cadence/kind taxonomy.
- `.planning/research/v0.12.0-decisions-log.md` D2 (gates as umbrella term) and D11 (per-milestone REQUIREMENTS scoping).
