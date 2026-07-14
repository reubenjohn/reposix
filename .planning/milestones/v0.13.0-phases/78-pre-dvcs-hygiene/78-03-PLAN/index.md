---
phase: 78
plan: 03
title: "MULTI-SOURCE-WATCH-01 — walker schema migration to source_hashes: Vec<String>"
wave: 2
depends_on: [78-01]
requirements: [MULTI-SOURCE-WATCH-01]
files_modified:
  - crates/reposix-quality/src/catalog.rs
  - crates/reposix-quality/src/commands/doc_alignment.rs
  - crates/reposix-quality/tests/walk.rs
  - quality/catalogs/doc-alignment.json
  - CLAUDE.md
autonomous: true
mode: standard
---

# Phase 78 Plan 03 — Multi-source walker schema migration (MULTI-SOURCE-WATCH-01)

<objective>
Close the v0.12.1 P75 carry-forward `MULTI-SOURCE-WATCH-01` via path-(b) — a
schema migration to a parallel-array `source_hashes: Vec<String>` so the
docs-alignment walker AND-compares per-source hashes on `Source::Multi` rows.
P75 shipped path-(a) (preserve first-source hash on `Source::Single →
Source::Multi` promotion); the explicit tradeoff documented at
`crates/reposix-quality/src/commands/doc_alignment.rs:307-310` was that drift
in non-first sources of a `Multi` row does NOT fire `STALE_DOCS_DRIFT` under
the current walker. P78-03 closes that false-negative window before the v0.13.0
docs surfaces (P85 DVCS docs) start adding multi-source rows that the walker
must police.

Schema migration shape (per CARRY-FORWARD.md `MULTI-SOURCE-WATCH-01`
acceptance):
- `Row::source_hashes: Vec<String>` parallel-array to `source.as_slice()`.
- `verbs::walk` hashes each source citation against `source_hashes[i]`; row
  enters `STALE_DOCS_DRIFT` on ANY index drift (AND-compare).
- `verbs::bind` writes/preserves all entries on the parallel array
  (Single result → 1-element vec; Multi append → push the new hash; Multi
  same-source rebind → refresh that index only).
- Existing single-source-hash field (`source_hash: Option<String>`) migrates
  via `serde(default)` + a one-time backfill: read `source_hash` if present,
  push it into `source_hashes[0]` on row load. The legacy field stays
  deserializable (newer code reads either; older catalog snapshots round-trip
  cleanly).
- Regression tests at `crates/reposix-quality/tests/walk.rs::walk_multi_source_*`
  exercise stable / first-drift / **non-first-drift** / single-rebind-heal
  cases. The non-first-drift case is the load-bearing new test — it
  demonstrates the false-negative is closed.

This plan **runs cargo serially** and **depends on 78-01**: per CLAUDE.md
"Build memory budget" only one cargo invocation at a time across all P78
plans. 78-01 (gix bump) holds the cargo lock first; 78-03 starts after 78-01
completes. 78-02 (shell + JSON only) ran in parallel with 78-01 in Wave 1
without contention.

Catalog-first ordering: a tracking row in `quality/catalogs/doc-alignment.json`
records the GREEN contract for this migration BEFORE the schema-migration
commits. The row's claim is the verbatim acceptance from CARRY-FORWARD.md;
its `tests:` field cites `crates/reposix-quality/tests/walk.rs::walk_multi_source_non_first_drift_fires_stale`
(the load-bearing new test) once authored.
</objective>

<must_haves>
- `crates/reposix-quality/src/catalog.rs::Row` has a new `source_hashes:
  Vec<String>` field with `#[serde(default, skip_serializing_if =
  "Vec::is_empty")]` so old catalogs (which lack the field) deserialize
  cleanly and new catalogs without populated hashes serialize compactly.
- The legacy `source_hash: Option<String>` field STAYS on `Row` for one
  release cycle (back-compat: catalog snapshots written by the
  pre-migration binary must still load). After deserialization, a
  one-time backfill copies `source_hash` into `source_hashes[0]` when
  `source_hashes.is_empty() && source_hash.is_some()`. The backfill runs
  inside `Catalog::load` so every read path enters the new world.
- New invariant: `source.as_slice().len() == source_hashes.len()` on every
  row loaded post-backfill. A new helper `Row::set_source(source, hashes)`
  validates this invariant + rejects mismatched lengths (mirroring the
  existing `Row::set_tests` shape at catalog.rs lines 175-185). Pre-existing
  rows that legitimately have `source_hash: None && source_hashes: []` (rows
  with no source_hash recorded — see catalog.rs line 387 in the bind path)
  remain valid: empty `source_hashes` is the "no hash recorded yet"
  semantic, parallel to "empty `tests`" at the parallel-array invariant
  documented in catalog.rs lines 120-124.
- `verbs::walk` (in `commands/doc_alignment.rs:827`) replaces the single-source
  drift check at lines 854-868 with a loop over `source.as_slice()` zipped
  against `source_hashes`. Drift detection per index:
  - File missing → drift (matches existing semantic).
  - Hash recompute fails → drift.
  - `now_hash != source_hashes[i]` → drift.
  - Any-index drift sets `source_drift = Some(true)` for the row's verdict
    aggregation. Index of drift surfaces in the diagnostic line for forensic
    clarity (mirrors the existing `drifted_indices` pattern for tests at
    lines 877-879).
- `verbs::bind` (catalog.rs `bind` fn around doc_alignment.rs:224) updates
  `source_hashes` on every code path:
  - **New row** (line ~334): `source_hashes: vec![src_hash]` (one-element
    matching the `Source::Single(new_source)` set at line 333). Also keep
    `source_hash: Some(src_hash)` for back-compat.
  - **Existing row, Single result** (line ~318): refresh `source_hashes[0]`
    AND `source_hash` (heal path). The pre-existing P75 heal logic carries
    forward; just update both fields together.
  - **Existing row, Multi append** (line ~315): push `src_hash` onto
    `source_hashes` so the new index matches the new source. Keep
    `source_hash` UNCHANGED (preserves first-source invariant per P75 ratchet).
  - **Existing row, Multi same-source rebind** (a case that previously fell
    into "Multi append → already_present == true → no-op" at line 294):
    must now refresh JUST that index in `source_hashes`. Find the matching
    `source.as_slice()` index by file + line_start + line_end equality;
    update `source_hashes[matching_index] = src_hash`.
- The verb `merge-shards` (catalog.rs around line 770) ALSO writes
  `source_hashes` consistently when it builds Multi rows from shard prototypes.
  Same shape: each source citation gets its corresponding hash on the parallel
  array. Compute hashes via `hash::source_hash` per cite; record into
  `source_hashes` parallel to `all_sources`.
- 5 new regression tests in `crates/reposix-quality/tests/walk.rs` (under
  `mod tests` or top-level `#[test]` fns following the existing pattern at
  lines 70-507):
  - `walk_multi_source_non_first_drift_fires_stale` — **the load-bearing
    new case**. Build a Multi row with 2 sources where the SECOND source's
    bytes change post-bind. Assert `walk` exits non-zero, prints
    `STALE_DOCS_DRIFT` for that row, and the diagnostic names the
    second-source index (or file path) so the operator knows which source
    drifted.
  - `walk_multi_source_first_drift_fires_stale` — regression of the
    pre-existing path-(a) behavior. First source drifts; walk fires
    `STALE_DOCS_DRIFT` (UNCHANGED behavior; ensure the migration didn't
    break it).
  - `walk_multi_source_stable_no_false_drift` — both sources stable; walk
    exits 0 (UNCHANGED behavior; ensure AND-compare is correct, not
    OR-compare).
  - `walk_legacy_catalog_backfills_source_hash_to_source_hashes` — load a
    catalog written by the pre-migration binary (mock: a hand-rolled JSON
    with `source_hash: "<hex>"` and no `source_hashes` field). Assert that
    after `Catalog::load`, `row.source_hashes == vec!["<hex>"]`. The
    backfill runs once, transparently to callers.
  - `bind_multi_same_source_rebind_refreshes_just_that_index` — bind the
    same source twice to a Multi row whose second source's bytes changed
    between binds. Assert that the rebound index's hash refreshed AND the
    other index's hash stayed the same.
  Some of these test names ALREADY EXIST in walk.rs (lines 323, 396, 447 —
  see canonical_refs). Where they exist, EXTEND or REWORK them; do NOT
  duplicate. The pre-existing tests target single-source scenarios or were
  P75-era approximations of the multi-source contract. The naming
  collision is a feature: same test name, stronger assertion under the
  migrated walker.
- Catalog tracking row added to `quality/catalogs/doc-alignment.json` BEFORE
  the schema-migration commit, citing the load-bearing test as the bound
  test:
  ```json
  {
    "id": "doc-alignment/multi-source-watch-01-non-first-drift",
    "claim": "Walker hashes every source citation in a Source::Multi row; row enters STALE_DOCS_DRIFT on any index drift (path-b per CARRY-FORWARD.md MULTI-SOURCE-WATCH-01)",
    "source": { "file": ".planning/milestones/v0.13.0-phases/CARRY-FORWARD.md", "line_start": 19, "line_end": 35 },
    "source_hash": "<computed via reposix-quality bind>",
    "tests": ["crates/reposix-quality/tests/walk.rs::walk_multi_source_non_first_drift_fires_stale"],
    "test_body_hashes": ["<computed via reposix-quality bind>"],
    "rationale": "P78-03 closes the v0.12.1 P75 carry-forward false-negative window before P85 DVCS docs add multi-source rows.",
    "last_verdict": "BOUND",
    "next_action": "BIND_GREEN",
    "last_run": "<ISO>",
    "last_extracted": "<ISO>",
    "last_extracted_by": "bind-call"
  }
  ```
  The row is minted via `reposix-quality bind` (NOT hand-edited JSON) per
  `quality/PROTOCOL.md` § "Principle A" — subagents propose with citations,
  tools validate and mint. Full minting CLI invocation is in T05.
- `CLAUDE.md` § "v0.12.1 — in flight" → "P75 — bind-verb hash-overwrite fix"
  → the "Path-(a) tradeoff" paragraph: replace the sentence "Path (b)
  (parallel `source_hashes: Vec<String>` + per-source walker compare) is
  filed as v0.13.0 carry-forward `MULTI-SOURCE-WATCH-01`." with "Path (b)
  closed in v0.13.0 P78-03 via the parallel-array `source_hashes: Vec<String>`
  schema migration; non-first-source drift now fires `STALE_DOCS_DRIFT`."
  Cite the commit SHA in the edit.
- Workspace gates GREEN (cargo check / clippy / nextest) per CLAUDE.md
  "Build memory budget" — single invocation at a time; serial.
- Per-phase push `git push origin main` BEFORE phase verifier-subagent
  dispatch (CLAUDE.md § "Push cadence — per-phase").

Threat model: schema migration is local-only data flow; no new network or
unsafe surface. The new walker AND-compare strengthens the
"docs-claim-bound-to-test" invariant (more drift gets caught) — net
defensive. No `<threat_model>` delta required.
</must_haves>

<canonical_refs>
- `.planning/milestones/v0.13.0-phases/CARRY-FORWARD.md:6-37`
  (`MULTI-SOURCE-WATCH-01` — verbatim acceptance criteria).
- `.planning/REQUIREMENTS.md` MULTI-SOURCE-WATCH-01.
- `crates/reposix-quality/src/catalog.rs:85-108` — `Source` enum (Single |
  Multi) + `as_slice` adapter (untouched by this plan).
- `crates/reposix-quality/src/catalog.rs:118-165` — `Row` struct (`source`,
  `source_hash`, parallel-array invariant doc-comment); add `source_hashes`
  here.
- `crates/reposix-quality/src/catalog.rs:175-185` — `Row::set_tests` pattern
  for the parallel-array setter; mirror this for the new
  `Row::set_source` setter.
- `crates/reposix-quality/src/commands/doc_alignment.rs:224-349` — `bind` fn;
  the four code paths (new row, Single result, Multi append, Multi
  same-source rebind) need `source_hashes` updates.
- `crates/reposix-quality/src/commands/doc_alignment.rs:455-510` — `bind` fn's
  alternate path (search for the second `src_hash` computation; lines 459-485
  per `grep -n source_hash` output); also needs the source_hashes update.
- `crates/reposix-quality/src/commands/doc_alignment.rs:770-816` —
  `merge-shards` builds Multi rows from shard prototypes; must compute +
  store per-source hashes.
- `crates/reposix-quality/src/commands/doc_alignment.rs:820-` — `walk` fn;
  the source-drift loop at lines 854-868 is the surface to migrate.
- `crates/reposix-quality/tests/walk.rs:323-447` — pre-existing
  `walk_multi_source_*` tests + `walk_single_source_rebind_heals_after_drift`;
  cohabit / extend; do not duplicate.
- `crates/reposix-quality/src/commands/doc_alignment.rs:296-316` — P75
  bind-verb-fix prose comment; the rationale carries forward; the path-(a)
  limitation note (line 307-310) gets replaced post-migration with a brief
  reference to P78-03.
- `quality/catalogs/doc-alignment.json` — the catalog where the
  MULTI-SOURCE-WATCH-01 tracking row lands. Pre-migration: 388 rows per
  CARRY-FORWARD.md note; post-migration each row carries `source_hashes` (1
  or N elements depending on Single vs Multi).
- `quality/PROTOCOL.md` § "Principle A — Subagents propose with citations;
  tools validate and mint" — the catalog row is minted via `reposix-quality
  bind`, not hand-edited.
- `CLAUDE.md` § "v0.12.1 — in flight" → "P75 — bind-verb hash-overwrite fix"
  — the path-(a) tradeoff paragraph that gets updated.
- `CLAUDE.md` "Build memory budget (load-bearing — read before
  parallelizing)" — cargo serialization rules.
- `CLAUDE.md` "Push cadence — per-phase" — `git push origin main` before
  verifier-subagent dispatch.
</canonical_refs>

---

## Chapters

- **[T01](./t01-row-schema.md)** — `source_hashes: Vec<String>` field on `Row`, `set_source` setter, `Catalog::load` backfill.
- **[T02](./t02-walk-migration.md)** — `verbs::walk` per-index loop over `source_hashes`; drifted-index diagnostic.
- **[T03](./t03-bind-merge-migration.md)** — `verbs::bind` (4 paths) + `merge-shards` write `source_hashes`; P75 comment updated.
- **[T04](./t04-regression-tests.md)** — 5 regression tests: `walk_multi_source_non_first_drift_fires_stale` (load-bearing) + 4 others.
- **[T05](./t05-catalog-mint-gates.md)** — MULTI-SOURCE-WATCH-01 catalog row mint, serial workspace gates, CLAUDE.md update.
- **[T06](./t06-push.md)** — Stage, commit, push; pre-push failure modes; terminal signal.
