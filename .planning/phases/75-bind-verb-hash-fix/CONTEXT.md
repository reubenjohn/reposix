# Phase 75: Bind-verb hash-overwrite fix (Context)

**Gathered:** 2026-04-29 (autonomous-run prep)
**Status:** Ready for execution
**Mode:** `--auto` (sequential gsd-executor on main; depth-1)
**Milestone:** v0.12.1
**Estimated effort:** 30-45 min wall-clock (1 fix + 1 regression test + verdict)

<domain>
## Phase Boundary

Fix the `bind` verb / walker hash-asymmetry bug surfaced repeatedly during the previous session's cluster sweeps. Root cause: `verbs::bind` appends source citations (turning `Source::Single` into `Source::Multi([A, B])`) AND overwrites `source_hash` with the NEW range's hash, but the walker reads `source_hash` against the FIRST source citation. After every cluster sweep that added a Multi citation, the walker fired false `STALE_DOCS_DRIFT` because `source_hash` was now a hash of B's range while the walker checked A's range.

Two viable fixes; pick the simpler one.

**Path (a) — preserve first-source semantics in `bind` (RECOMMENDED).**
On `Source::Single → Source::Multi` conversion, KEEP the existing `source_hash` (don't overwrite). On rebinds where the existing row is already `Multi`, only update `source_hash` if the FIRST source's range changes.

**Path (b) — walker hashes all sources from a Multi.**
Walker computes a hash per source and compares against `source_hash` (single field) → schema bump to `source_hashes: Vec<String>` with parallel-array invariant on `Multi` rows. More invasive; touches schema; needs migration for the populated 388-row catalog.

**Recommendation:** Path (a). Smaller surface, no schema change, fixes the observed bug. Path (b) is technically more general but P75 is one phase, not a redesign.

**Explicitly NOT in scope:**
- Schema migration to `source_hashes: Vec<String>`.
- Reworking the bind verb's API surface.
- Touching the walker's hash-checking logic beyond the regression test for the Multi path.

</domain>

<decisions>
## Implementation Decisions

### D-01: Path (a) — preserve first-source semantics in `bind`
In `crates/reposix-quality/src/commands/doc_alignment.rs::verbs::bind` (line ~295 area per HANDOVER §4): when the row's existing `source` is `Source::Single` and the bind invocation cites a NEW source (different file or line range), CONVERT to `Source::Multi([existing, new])` BUT keep `source_hash` set to the hash of `existing` (the first element). When the existing source is already `Source::Multi` and the bind cites another, APPEND to the array; do not touch `source_hash`.

### D-02: Add explicit unit test for the bind verb's conversion behavior
`crates/reposix-quality/src/commands/doc_alignment.rs` (or `tests/bind_validation.rs`) gains a test asserting:
1. Bind row to `Source::Single(A)` → `source_hash = hash(A)`.
2. Re-bind same row with new source `B` → `source = Source::Multi([A, B])` AND `source_hash` UNCHANGED (== hash(A)).
3. Re-bind same row with another source `C` → `source = Source::Multi([A, B, C])` AND `source_hash` UNCHANGED.

### D-03: Add walker integration test for Multi rows
`crates/reposix-quality/tests/walk.rs` gains a test that:
1. Seeds a temp catalog with one row whose `source = Source::Multi([fileA:1-5, fileB:1-5])` and `source_hash = hash(fileA:1-5)`.
2. Writes both temp files with stable content.
3. Runs walk; asserts row stays `BOUND`, NO `STALE_DOCS_DRIFT` fires.
4. Edits fileA's range; runs walk; asserts STALE_DOCS_DRIFT fires (catches the case the walker SHOULD detect).
5. Reverts fileA; edits fileB's range; runs walk; asserts row STAYS BOUND (the walker only checks first-source per path-a; B's drift doesn't fire). This is the explicit known limitation of path-a; document it in the PR description AND in a CLAUDE.md note for the next maintainer.

### D-04: Live-catalog smoke after fix
After landing the fix, run `target/release/reposix-quality doc-alignment walk` against the LIVE 388-row catalog. Assert no rows transition to `STALE_DOCS_DRIFT` from a no-op walk. Capture stdout/stderr in the verdict.

### D-05: Document the path-(a) tradeoff
The path-(a) fix means walker only watches the FIRST source citation in a Multi row. Drift in non-first sources WON'T fire. Document this explicitly in CLAUDE.md (a P75 note) so the next maintainer doesn't re-discover by surprise. Path-(b) (full multi-source watch) is filed as v0.13.0 carry-forward `MULTI-SOURCE-WATCH-01`.

### D-06: Verifier subagent dispatch — Path A
Verdict at `quality/reports/verdicts/p75/VERDICT.md`.

### D-07: Cargo memory budget
`cargo test -p reposix-quality` only — single crate. ~15s.

### D-08: CLAUDE.md update
P75 H3 subsection ≤20 lines (smaller than P72-P74 because the fix is smaller). Note the bind-verb behavior change AND the path-(a) tradeoff (walker watches first source only).

### D-09: Eager-resolution
If the regression test reveals OTHER bind/walker bugs, append to SURPRISES-INTAKE.md unless < 30 min to fix.

</decisions>

<canonical_refs>
## Canonical References

- `crates/reposix-quality/src/commands/doc_alignment.rs::verbs::bind` — the buggy verb.
- `crates/reposix-quality/src/catalog.rs` — `Source` enum (`Single` + `Multi(Vec<SourceCite>)`), `Row::source_hash` field.
- `crates/reposix-quality/tests/walk.rs` — existing walker integration test pattern.
- `quality/catalogs/doc-alignment.json` — populated 388-row catalog; the live smoke target.
- HANDOVER-v0.12.1.md § 4 — original bug report.
- `quality/SURPRISES.md` — pivot journal where the bind-verb hash-asymmetry was first observed during P67.

</canonical_refs>

<specifics>
## Specific Ideas

- Read `verbs::bind` source first; the line ~295 reference is approximate. The mutation site is wherever `row.source_hash =` appears in the conversion branch.
- The walker's hash check site is in the `walk` verb's row-iteration loop; check `crates/reposix-quality/src/commands/doc_alignment.rs::verbs::walk` for the comparison.
- `assert_no_panic!` patterns from existing tests apply.
- Phase verdict: include the live walk's stdout/stderr verbatim as evidence the fix landed.

</specifics>

<deferred>
## Deferred Ideas

- Path (b) full multi-source watch — file as v0.13.0 `MULTI-SOURCE-WATCH-01`.
- Refactoring the bind verb's verb-pattern (it's getting busy: bind, propose-retire, confirm-retire, mark-missing-test all call into verbs::*). P77 GOOD-TO-HAVE candidate.
- Telemetry on how often the walker fires STALE_DOCS_DRIFT (helps tune verifier cadences). v0.13.0.

</deferred>

---

*Phase: 75-bind-verb-hash-fix*
*Context gathered: 2026-04-29*
*Source: HANDOVER-v0.12.1.md § 4 + autonomous-run prep.*
