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
| `cadences` | yes | non-empty list; each element one of: `pre-commit`, `pre-push`, `pre-pr`, `weekly`, `pre-release`, `post-release`, `on-demand`. A single gate MAY fire at multiple cadences (e.g., a cheap mechanical check tagged `["pre-commit", "pre-push", "pre-pr"]` runs at every relevant trigger). |
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

## docs-alignment dimension

The catalog at `quality/catalogs/doc-alignment.json` extends the unified
schema with a per-dimension shape: a wrapper `summary` block + per-row
`source_hash` / `test_body_hashes` / `last_verdict` / `last_extracted` /
`last_extracted_by` fields. Source of truth:
`.planning/research/v0.12.0-docs-alignment-design/02-architecture.md`.

**Top-level catalog field:** `schema_version` (string, currently `"2.0"`).
v1.0 carried singular `test` / `test_body_hash` fields; v2.0 (P71/W7,
commit `d2127c3`) replaces them with the parallel-array shape below. The
388-row catalog was migrated in place by
`scripts/migrate-doc-alignment-schema-w7.py`; readers MUST accept v2.0,
and the structural verifier accepts both `"1.0"` and `"2.0"` during the
v0.12.1 transition.

**Row schema (v2 — added fields):** `id` (kebab-case `<slug>` -- the
dimension prefix is implicit), `claim` (one-sentence behavioral claim),
`source` (`f:l-l` citation, multi-source rows carry an array of
citations), `source_hash` (`sha256` of the cited line range; computed by
`bind`), `tests` (`Vec<String>` of `f::sym` Rust test fn citations;
empty vec means "no test bound yet" — replaces v1.0's `Option<String>`),
`test_body_hashes` (`Vec<String>`, parallel to `tests`; each element is
`syn::ItemFn::to_token_stream()` then `sha256` for the corresponding
test fn; comments + whitespace normalized away), `rationale` (optional;
walker tolerates absence via `serde(default)`), `last_verdict`,
`last_run`, `last_extracted`, `last_extracted_by`.

**Parallel-array invariant (v2).** `tests.len() == test_body_hashes.len()`
at all times. The invariant is enforced at deserialize via
`Row::validate_parallel_arrays` and the gatekeeping setter
`Row::set_tests(tests, hashes) -> Result<()>` (Rust); `Row::clear_tests`
is the explicit detach used by `mark-missing-test`. Walker drift
detection iterates per-element: any missing test fn flips the row to
`STALE_TEST_GONE`; else any drifted hash flips to `STALE_TEST_DRIFT`;
else `BOUND`. One row may bind to multiple tests (e.g. a JIRA-writes
claim binding to create + update + delete + conflict-recovery tests).

**Row state machine:** `BOUND` (grader GREEN), `MISSING_TEST` (extractor
found claim with no test -- pre-push BLOCKS), `STALE_DOCS_DRIFT` (source
hash changed -- pre-push BLOCKS, refresh required), `STALE_TEST_DRIFT`
(test body hash changed -- soft, re-grade at next phase close),
`STALE_TEST_GONE` (cited test path/symbol no longer resolves -- pre-push
BLOCKS), `TEST_MISALIGNED` (grader RED -- pre-push BLOCKS),
`RETIRE_PROPOSED` (extractor proposed retirement -- pre-push BLOCKS until
human confirms), `RETIRE_CONFIRMED` (env-guarded human-only).

**Summary block:** `claims_total`, `claims_bound`, `claims_missing_test`,
`claims_retire_proposed`, `claims_retired`, `alignment_ratio` (=
`claims_bound / max(1, claims_total - claims_retired)`; 1.0 when
`claims_total == 0`), `floor` (initially 0.50; only ratchets up via
deliberate human commit), `trend_30d`, `last_walked`, `coverage_ratio`,
`lines_covered`, `total_eligible_lines`, `coverage_floor` (P66 v0.12.1).
Pre-push BLOCKS when `claims_total > 0 && alignment_ratio < floor` OR
when `coverage_ratio < coverage_floor`. Both blocks can fire on the same
walk.

### Two metrics: alignment + coverage (P66 v0.12.1)

The dimension grades along TWO axes. Each answers a different question;
together they yield the agent's mental model:

- `alignment_ratio = claims_bound / max(1, claims_total - claims_retired)`
  -- "of the claims we extracted, how many bind to passing tests?"
- `coverage_ratio = lines_covered / total_eligible_lines`
  -- "of the prose we said we'd mine, what fraction of lines did we
  actually cover with at least one row?"

|                  | high alignment                      | low alignment                            |
|------------------|-------------------------------------|------------------------------------------|
| **high coverage**| ideal -- most prose mined, most claims tested | extracted everything; most claims unbound |
| **low coverage** | tested what we found; missed most prose       | haven't started                          |

Without the coverage axis, an agent could ship a high `alignment_ratio`
by extracting only easy claims and binding them. The coverage axis
closes that loophole; the per-file `status --top 10` table is the
agent's gap-target view.

**`coverage_floor` ratchet semantics.** Default 0.10 (low; even sparse
mining usually clears it). Ratcheted up by deliberate human commits as
gap-closure phases (v0.12.1 P72+) widen extraction. The walker NEVER
auto-tunes `coverage_floor` -- only the alignment_ratio floor does
(monotone non-decreasing, audited by `structure/doc-alignment-floor-not-decreased`).
There is currently no symmetric audit row for `coverage_floor`; if it
becomes a footgun (e.g., ratcheted too high then quietly lowered to dodge
a BLOCK), file a structure-dimension row to assert monotone behavior.

**Eligible file set.** `docs/**/*.md` + `README.md` + archived
`REQUIREMENTS.md` for v0.6.0 -- v0.11.0. Mirrors the chunker's
`collect_backfill_inputs`. Rows whose `source.file` cite a path OUTSIDE
this set get a stderr warning (`coverage: row {id} cites out-of-eligible
file {path}`) and their lines are NOT counted -- forensic signal that a
citation drifted (file moved/renamed/deleted).

**Floor waiver shape:** standard catalog `waiver` block per
`quality/PROTOCOL.md` § "Waiver protocol" applies at the row level; the
floor itself has no waiver semantics -- it is monotone non-decreasing by
design and the structure-dimension row
`structure/doc-alignment-floor-not-decreased` audits the git history.

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
- `.planning/research/v0.12.0/naming-and-architecture.md` — schema design rationale.
- `.planning/research/v0.12.0/vision-and-mental-model.md` — dimension/cadence/kind taxonomy.
- `.planning/research/v0.12.0/decisions-log.md` D2 (gates as umbrella term) and D11 (per-milestone REQUIREMENTS scoping).
