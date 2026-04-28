---
phase: 64-docs-alignment-framework
plan: 03
subsystem: quality-gates
tags: [quality-gates, docs-alignment, runner-integration, hook-wiring, protocol-md, claude-md, requirements-flip, verifier-dispatch, path-b]

requires:
  - phase: 64
    plan: 01
    provides: "doc-alignment empty-state catalog + 3 freshness invariants + skill scaffolding (preflight)"
  - phase: 64
    plan: 02
    provides: "reposix-quality umbrella crate + clap surface + hash binary + 28 tests"
provides:
  - "quality/gates/docs-alignment/walk.sh NEW (release-first, debug-fallback wrapper) wiring the docs-alignment walker into the runner"
  - "quality/catalogs/freshness-invariants.json gains docs-alignment/walk row (P0 pre-push) -- the runner gate registry entry"
  - "quality/PROTOCOL.md gains 'Two project-wide principles' H2 section (~37 added lines under 80 cap): Principle A (subagents propose with citations; tools validate and mint) + Principle B (tools fail loud, structured, agent-resolvable) with cross-tool examples"
  - "CLAUDE.md gains: dimensions matrix bump 8 -> 9 with docs-alignment row + P64 H3 subsection (~28 added lines under 40 cap)"
  - "quality/SURPRISES.md gains 3 P64 entries (no-significant-pivots note + gate-registry-placement pivot + walker-last_walked-churn pivot)"
  - ".planning/REQUIREMENTS.md flips DOC-ALIGN-01..07 to [x] (P64, 2026-04-28)"
  - ".planning/STATE.md gains P64 SHIPPED ROADMAP-EVOLUTION stanza + Current Position cursor advance + frontmatter progress bump (v0.12.0 8/10 -> 9/10 = 90%)"
  - "quality/reports/verdicts/p64/VERDICT.md NEW: GREEN, Path B in-session, 14/14 success criteria graded with primary-source evidence"
  - "Phase 64 closes; P65 (top-level execution mode -- backfill audit) ready to begin"
affects: [65-docs-alignment-backfill, milestone-close-v0.12.0]

tech-stack:
  added: []
  patterns:
    - "Runner integration as a catalog row (not a python registry) -- the runner is purely catalog-driven; new gates land as rows in any catalog file with `cadence` set"
    - "Wrapper script as the gate's `verifier.script` -- target/release/<binary> first, target/debug fallback, stderr forwarded verbatim so runner surfaces slash-command hints"
    - "Path B in-session verifier dispatch -- gsd-executor lacks Task; verifier IS the executor with explicit context-reset; 4-disclosure block per P56-P63 precedent"
    - "Two project-wide principles section in PROTOCOL.md -- generalizes the runtime contract into operational guidance for every verifier in quality/gates/<dim>/"

key-files:
  created:
    - "quality/gates/docs-alignment/walk.sh -- 1-line bash wrapper detecting release-vs-debug binary, exec walker, forward stderr verbatim"
    - "quality/reports/verdicts/p64/VERDICT.md -- GREEN verdict, 14 per-criterion grades, 4-disclosure block, suspicion-of-haste section"
  modified:
    - "quality/catalogs/freshness-invariants.json -- 1 new row (`docs-alignment/walk`, P0 pre-push, 17 rows total; existing 16 unchanged)"
    - "quality/PROTOCOL.md -- new H2 section 'Two project-wide principles' inserted between 'Reading order' and 'Failure modes' (37 added lines)"
    - "CLAUDE.md -- dimensions matrix 8 -> 9 + new docs-alignment row + P64 H3 subsection (~28 added lines; total 31,734 bytes under 40 KB cap)"
    - "quality/SURPRISES.md -- 3 P64 entries appended"
    - ".planning/REQUIREMENTS.md -- DOC-ALIGN-01..07 flipped to [x] with (P64, 2026-04-28) date stamp"
    - ".planning/STATE.md -- P64 SHIPPED stanza + cursor advance + frontmatter progress bump"
    - "quality/catalogs/doc-alignment.json -- summary.last_walked timestamp mutated by walker (intentional per walker contract; v0.12.1 carry-forward to address per-pre-push churn)"

key-decisions:
  - "Gate registry placement -- the runner is catalog-driven; the docs-alignment.json catalog has its own claim-row schema (id/claim/source/source_hash/test/...) that the binary's Catalog struct deserializes. Mixing a runner-style gate row (cadence/verifier/artifact) into rows[] would break deserialization. Resolved by adding the docs-alignment/walk row to freshness-invariants.json (the structure dimension's catalog) under dimension=docs-alignment. The runner is catalog-agnostic -- it discovers rows across every catalog file. Filed as a P64 SURPRISES entry."
  - "Walker wrapper script name -- walk.sh (consistent with hash_test_fn wrapper precedent in the same dir). Detects target/release first, target/debug fallback. Forwards stderr verbatim so the slash-command hint reaches the user."
  - "PROTOCOL.md insertion point -- between 'Reading order' (l13) and 'Failure modes' (l24). The plan said 'between Runtime contract and Failure modes'; there is no explicit 'Runtime contract' section header but the top section serves that role. The chosen point is just before the failure-modes table, which is the most natural place for a section that operationalizes the runtime contract into per-tool guidance."
  - "CLAUDE.md size budget -- 31,734 bytes after Wave 3 additions; well under 40 KB project hard cap. No archive rotation needed. Wave 3 additions: ~28 lines for P64 H3 + 1 row added to dimensions table + dimensions count bumped 8 -> 9."
  - "Verifier dispatch -- Path B in-session per P56-P63 precedent. The Task tool is unavailable inside gsd-executor; the verifier IS the executor with explicit context-reset (re-reading 05-p64-infra-brief.md success criteria + primary-source artifacts; refusing to use the executor's own SUMMARY.md as evidence). 4-disclosure block honored verbatim."
  - "REQUIREMENTS.md flips -- DOC-ALIGN-01..07 with (P64, 2026-04-28) date stamp, matching the existing flip-style precedent (see ORG-01, POLISH-ORG which use `**SHIPPED P62.**` text but this plan adopts the date-stamp format from CLAUDE.md updated-fields convention)."
  - "STATE.md progress fields -- frontmatter does not have v0.12.0-specific phase counters; added two new keys (v0_12_0_phases_completed: 9, v0_12_0_percent: 90) alongside the global completed_plans bump (13 -> 16). The plan's 8 -> 9 phases / 80% -> 90% targets reflect v0.12.0 milestone scope (P56-P65 = 10 phases; P56-P64 = 9 shipped)."
  - "Walker `last_walked` mutation accepted as v0.12.0 behavior (per architecture spec); per-pre-push catalog churn filed as v0.12.1 MIGRATE-03 carry-forward (h) -- either move into artifact OR extend catalog_dirty() to ignore summary.last_walked drift."

requirements-completed: [DOC-ALIGN-01, DOC-ALIGN-02, DOC-ALIGN-03, DOC-ALIGN-04, DOC-ALIGN-05, DOC-ALIGN-06, DOC-ALIGN-07]

duration: ~30min
completed: 2026-04-28
---

# Phase 64 Plan 03: Runner integration + hook + PROTOCOL.md principles + CLAUDE.md + verifier dispatch Summary

**P64 Wave 3 ships the runner integration (docs-alignment/walk gate row), hook validation (no structural change needed), the two project-wide principles in PROTOCOL.md, CLAUDE.md updates (dimensions matrix + P64 H3 subsection), REQUIREMENTS.md flips for DOC-ALIGN-01..07, STATE.md cursor advance, and the in-session Path B verifier dispatch that closes P64 with a GREEN verdict at quality/reports/verdicts/p64/VERDICT.md.**

## Performance

- **Duration:** ~30 min wall-clock (Wave 3 only)
- **P64 total:** ~1h08m wall-clock from Wave 1 commit `d0d4730` (~07:42Z) to Wave 3 verdict commit (~08:50Z)
- **Tasks:** 2 atomic commits (Commit A runner+hook+PROTOCOL; Commit B CLAUDE+SURPRISES+REQ+STATE+verdict)
- **Files modified:** 1 created (walk.sh) + 1 created (VERDICT.md) + 6 modified (PROTOCOL.md, freshness-invariants.json, CLAUDE.md, SURPRISES.md, REQUIREMENTS.md, STATE.md, doc-alignment.json) = 9 unique paths
- **Tests:** 28 reposix-quality tests re-run PASS during verifier dispatch; workspace cargo test 68 groups, 0 failures

## Accomplishments

- **Runner integration live.** New `docs-alignment/walk` row in `quality/catalogs/freshness-invariants.json` (P0 pre-push, dimension=docs-alignment) shells to `quality/gates/docs-alignment/walk.sh`. The wrapper detects `target/release/reposix-quality` first, falls back to debug, forwards stderr verbatim. `python3 quality/runners/run.py --cadence pre-push` shows `[PASS] docs-alignment/walk (P0, 0.01s)` in the rollup. `.githooks/pre-push` chain through `python3 quality/runners/run.py --cadence pre-push` picks the new gate up via the registry; no structural change needed.
- **Two project-wide principles in PROTOCOL.md.** New H2 section "Two project-wide principles" inserted between "Reading order" and "Failure modes". Principle A (Subagents propose with citations; tools validate and mint) and Principle B (Tools fail loud, structured, agent-resolvable) verbatim from `02-architecture.md`, each with cross-tool examples enumerated as bullet lists. ~37 added lines under 80-line cap. Banned-words clean.
- **CLAUDE.md updated.** Dimensions matrix bumped 8 -> 9 with new docs-alignment row. New H3 subsection "P64 -- Docs-alignment dimension framework + skill" under § "v0.12.0 Quality Gates -- milestone shipped 2026-04-27" summarises Wave 1 catalog-first + Wave 2 binary surface + Wave 3 runner integration + verifier dispatch path. The "Orchestration-shaped phases run at top-level" note from `03-execution-modes.md` was already shipped in commit `7abd43c` during phase scoping; verified still in place. ~28 added lines under 40-line cap. CLAUDE.md total size: 31,734 bytes (under 40 KB hard cap).
- **REQUIREMENTS.md flips.** 7 requirements flipped to shipped (P64): DOC-ALIGN-01..07 each stamped `(P64, 2026-04-28)`. Existing checked entries untouched.
- **STATE.md advanced.** Frontmatter `v0_12_0_phases_completed: 9` (was 8 implicit) + `v0_12_0_percent: 90`. ROADMAP-EVOLUTION P64 SHIPPED stanza prepended (cites Wave 1 + Wave 2 + Wave 3 commits, binary surface, catalog state, verdict path, next step). Current Position cursor advanced to "Phase 65 -- READY TO EXECUTE (top-level mode)" with the P64 cursor archived in place.
- **SURPRISES.md gains 3 P64 entries.** No-significant-pivots note + gate-registry-placement pivot (the docs-alignment.json catalog's claim-row schema vs the runner's gate-row schema; resolved by adding to freshness-invariants.json) + walker-last_walked-churn pivot (filed as v0.12.1 MIGRATE-03 carry-forward h).
- **Verifier verdict GREEN.** `quality/reports/verdicts/p64/VERDICT.md` Path B in-session per P56-P63 precedent. 14/14 success criteria from `05-p64-infra-brief.md` graded GREEN with primary-source evidence (test names + test bodies + catalog row IDs + diff hunks). 4-disclosure block honored verbatim. Suspicion-of-haste check executed: 3 random catalog rows + 3 random tests spot-checked; cargo workspace re-ran clean; alignment passes. P64 wall-clock < 2h triggers the haste check; spot-checks confirm no haste-induced inconsistency.
- **Workspace cargo gates clean (FINAL).** `cargo check --workspace` finished in 11.48s. `cargo clippy --workspace --all-targets -- -D warnings` finished in 0.22s clean. `cargo fmt --all -- --check` exit 0. `cargo test --workspace` 68 test groups, 0 failures. Memory budget honored -- one cargo invocation at a time.

## Task Commits

Two atomic commits per plan must_haves.truths:

1. **Commit A (`7036643`)** -- `feat(reposix-quality): runner integration + hook wiring + PROTOCOL.md two principles` -- 4 files changed, 99 insertions, 1 deletion. New walk.sh wrapper + new freshness-invariants row + PROTOCOL.md two-principles section + walker mutated last_walked timestamp.
2. **Commit B (`5a1c6b9`)** -- `docs(p64): CLAUDE.md update + SURPRISES + REQUIREMENTS flip + STATE + verifier verdict GREEN` -- 6 files changed, 298 insertions, 15 deletions. CLAUDE.md updates + SURPRISES.md entries + REQUIREMENTS.md flips + STATE.md cursor + GREEN VERDICT.md.

## Files Created/Modified

### Created (2 files)

- `quality/gates/docs-alignment/walk.sh` -- 1-line bash wrapper (chmod 755) detecting release-vs-debug binary, exec'ing walker, forwarding stderr verbatim. Mirrors the `hash_test_fn` wrapper precedent in the same directory.
- `quality/reports/verdicts/p64/VERDICT.md` -- GREEN verdict. Path B in-session disclosure per P56-P63 precedent. 14 per-criterion grades against `05-p64-infra-brief.md` success criteria. 4-disclosure block. Suspicion-of-haste check section. Recommends "P65 may begin".

### Modified (7 files)

- `quality/catalogs/freshness-invariants.json` -- 1 new row appended: `docs-alignment/walk` (dimension=docs-alignment, cadence=pre-push, kind=mechanical, blast_radius=P0, verifier.script=quality/gates/docs-alignment/walk.sh, expected.asserts cover the walker's contract). 17 rows total; existing 16 untouched.
- `quality/PROTOCOL.md` -- new H2 section "Two project-wide principles" inserted between "Reading order for an agent picking up a phase" and "Failure modes the protocol protects against". Principle A + Principle B + cross-tool examples as bullet lists. 37 added lines.
- `CLAUDE.md` -- dimensions matrix bumped 8 -> 9; new `docs-alignment | claims have tests; hash drift detection | quality/gates/docs-alignment/ (P64 -- shipped)` row added. New H3 subsection "P64 -- Docs-alignment dimension framework + skill (added 2026-04-28)" under § "v0.12.0 Quality Gates -- milestone shipped 2026-04-27". ~28 added lines under 40-line cap.
- `quality/SURPRISES.md` -- 3 P64 entries appended (no-significant-pivots note + gate-registry-placement pivot + walker-last_walked-churn pivot). Active line count ~245.
- `.planning/REQUIREMENTS.md` -- DOC-ALIGN-01..07 (7 entries) flipped from `[ ]` to `[x]` with `(P64, 2026-04-28)` date stamp.
- `.planning/STATE.md` -- frontmatter `last_updated` advanced to `2026-04-28T08:40:00Z`; new keys `v0_12_0_phases_completed: 9` + `v0_12_0_percent: 90`; global `completed_plans` bumped 13 -> 16. ROADMAP-EVOLUTION P64 SHIPPED stanza prepended. Current Position cursor advanced to Phase 65; P64 cursor archived in place.
- `quality/catalogs/doc-alignment.json` -- `summary.last_walked` timestamp mutated by walker on Wave 3 runner sweep (intentional per walker contract; v0.12.1 carry-forward (h) addresses per-pre-push churn).

## Decisions Made

(See frontmatter `key-decisions:` for the full list. Highlights below.)

- **Gate registry placement.** The docs-alignment.json catalog has its own rigid claim-row schema; the runner-style gate row had to land in freshness-invariants.json instead. The runner's per-row dimension tag (`row.dimension`) means gate rows can live in any catalog file -- the per-file dimension boundary is a convenience, not a contract.
- **Walker wrapper script approach.** `walk.sh` mirrors the `hash_test_fn` wrapper precedent in the same dir. Release-first, debug-fallback, stderr forwarded.
- **PROTOCOL.md insertion point.** Between "Reading order" and "Failure modes" -- the most natural place for a section that operationalizes the runtime contract.
- **Verifier dispatch via Path B.** Per P56-P63 precedent; Task tool unavailable inside gsd-executor; verifier IS the executor with explicit context-reset.
- **STATE.md frontmatter additions.** Two new keys (`v0_12_0_phases_completed`, `v0_12_0_percent`) capture v0.12.0 milestone-specific progress alongside the global counters.

## Deviations from Plan

### Auto-fixed issues

**1. [Rule 3 - Blocking discrepancy] Gate registry shape vs catalog schema mismatch**

- **Found during:** Step 2 (designing the runner integration).
- **Issue:** The plan said "add a new gate row entry in [quality/runners/run.py]'s registry pointing at the docs-alignment dimension." The runner is purely catalog-driven (no Python registry); rows live in catalog JSON files. The natural target catalog `quality/catalogs/doc-alignment.json` has its own rigid claim-row schema (id/claim/source/source_hash/test/...) that the binary's Rust `Catalog` struct deserializes. Mixing a runner-style gate row (cadence/verifier/artifact) into `rows[]` would break the binary's deserialization.
- **Fix:** Added the `docs-alignment/walk` row to `quality/catalogs/freshness-invariants.json` (the structure dimension's catalog) under `dimension=docs-alignment`. The runner is catalog-agnostic -- it discovers rows across every catalog file by `cadence`, not by `dimension==catalog-name`. New gate row landed at P0 pre-push without schema change to either catalog.
- **Files modified:** `quality/catalogs/freshness-invariants.json` (1 row appended).
- **Verification:** `python3 quality/runners/run.py --cadence pre-push` shows `[PASS] docs-alignment/walk (P0, 0.01s)` in the rollup; exit 0.
- **Documented in:** `quality/SURPRISES.md` P64 Wave 3 entry; `quality/reports/verdicts/p64/VERDICT.md § Per-criterion grading 7`.
- **Committed in:** Commit A (`7036643`).

**2. [Rule 2 - Critical correctness] Walker per-pre-push catalog churn**

- **Found during:** Step 2 verification (running the runner sweep).
- **Issue:** `cat.summary.last_walked = Some(now_iso())` in `crates/reposix-quality/src/commands/doc_alignment.rs:780` writes the catalog on every walker invocation, mutating `quality/catalogs/doc-alignment.json` on every pre-push. This violates the runner's `catalog_dirty()` philosophy (per-run timestamp churn lives in artifacts, not committed catalogs).
- **Fix path (deferred to v0.12.1):** Either move `last_walked` into the artifact (`quality/reports/verifications/docs-alignment/walk.json`) or extend `catalog_dirty()` to ignore `summary.last_walked` drift the same way it ignores per-row `last_verified` drift.
- **Decision:** Accepted for v0.12.0 because the walker spec at `02-architecture.md` treats `last_walked` as a catalog-summary field; the change is non-blocking for P65 backfill. Filed as v0.12.1 MIGRATE-03 carry-forward (h).
- **Documented in:** `quality/SURPRISES.md` P64 Wave 3 entry; `quality/reports/verdicts/p64/VERDICT.md § v0.12.1 MIGRATE-03 carry-forwards`.

---

**Total deviations:** 2 (1 Rule 3 auto-fixed, 1 Rule 2 deferred to v0.12.1).
**Impact on plan:** Trivial. Both deviations resolved while honoring the plan's contract; the gate is live, the verdict is GREEN, the v0.12.1 carry-forward is filed.

## Issues Encountered

- **Pre-commit soft warnings on REQUIREMENTS.md (38,140 chars) and STATE.md (58,859 chars).** Both files exceed the personal pre-commit's 20,000-char soft limit. Both warnings are pre-existing (carried forward from prior phases); commits succeeded. Helper-extraction or progressive-disclosure refactor is a v0.12.1+ concern.
- **doc-alignment.json walker churn.** Documented above as Rule 2 deviation. Non-blocking; v0.12.1 carry-forward.

## Self-Check: PASSED

### Files exist on disk

- `quality/gates/docs-alignment/walk.sh` (chmod 755) -- FOUND
- `quality/reports/verdicts/p64/VERDICT.md` -- FOUND
- `quality/catalogs/freshness-invariants.json` (17 rows; new docs-alignment/walk at index 16) -- FOUND
- `quality/PROTOCOL.md` (with "Two project-wide principles" H2 section) -- FOUND
- `CLAUDE.md` (with docs-alignment dimension row + P64 H3 subsection) -- FOUND
- `quality/SURPRISES.md` (with 3 new P64 entries) -- FOUND
- `.planning/REQUIREMENTS.md` (DOC-ALIGN-01..07 flipped to [x]) -- FOUND
- `.planning/STATE.md` (P64 SHIPPED stanza + cursor advance) -- FOUND

### Commits exist

- `7036643 feat(reposix-quality): runner integration + hook wiring + PROTOCOL.md two principles` -- FOUND in `git log --oneline -3`
- `5a1c6b9 docs(p64): CLAUDE.md update + SURPRISES + REQUIREMENTS flip + STATE + verifier verdict GREEN` -- FOUND

### Verification commands

- `python3 quality/runners/run.py --cadence pre-push` -- 22 PASS, 0 FAIL, 0 PARTIAL, 3 WAIVED, 0 NOT-VERIFIED, exit 0 (the new docs-alignment/walk row graded PASS)
- `python3 quality/runners/run.py --cadence pre-pr` -- 2 PASS + 3 WAIVED, exit 0
- `cargo check --workspace` -- exit 0 (11.48s)
- `cargo clippy --workspace --all-targets -- -D warnings` -- clean (0.22s)
- `cargo fmt --all -- --check` -- exit 0
- `cargo test --workspace` -- 68 test groups, 0 failures
- `cargo test -p reposix-quality` -- 28 PASS
- `bash scripts/banned-words-lint.sh` -- exit 0
- `wc -c CLAUDE.md` -- 31,734 bytes (under 40 KB hard cap)

## Threat Flags

(No new threat surface introduced by P64. The walker reads files and writes catalog state under git control; no new HTTP egress, no new credential paths, no new auth boundaries. Empty section retained per template convention.)

## Out of Scope (Deferred to P65 / v0.12.1)

Per plan "Out of scope" section:

- **P65 backfill execution** (top-level mode) -- separate phase; orchestrator dispatches ~25-35 shard subagents in waves of 8 (Haiku tier; Path A via Task tool). Cannot run inside `/gsd-execute-phase` (depth-2 unreachable). Brief at `06-p65-backfill-brief.md`.
- **Closing any MISSING_TEST or RETIRE_PROPOSED row** -- v0.12.1 work; rows are populated by P65 backfill, then graded.
- **Walker last_walked churn fix** -- v0.12.1 MIGRATE-03 (h) carry-forward.

## Cross-references

- `.planning/phases/64-docs-alignment-framework/64-03-PLAN.md` -- the plan executed.
- `.planning/phases/64-docs-alignment-framework/64-CONTEXT.md` -- locked decisions and canonical refs.
- `.planning/phases/64-docs-alignment-framework/64-01-SUMMARY.md` -- Wave 1 (catalog-first commit + skill scaffolding).
- `.planning/phases/64-docs-alignment-framework/64-02-SUMMARY.md` -- Wave 2 (crate skeleton + binary surface + hash binary + 28 tests).
- `.planning/research/v0.12.0-docs-alignment-design/02-architecture.md` -- normative source of the two project-wide principles.
- `.planning/research/v0.12.0-docs-alignment-design/03-execution-modes.md` -- normative source of the orchestration-shaped phases note.
- `.planning/research/v0.12.0-docs-alignment-design/05-p64-infra-brief.md` § "Success criteria" -- the 14 criteria graded by the verifier.
- `quality/reports/verdicts/p64/VERDICT.md` -- this plan's GREEN verifier verdict; gate state for P65.
- `quality/PROTOCOL.md` § "Two project-wide principles" -- landed by this plan; cross-cutting guidance for every dimension going forward.
- `CLAUDE.md` § "Quality Gates -- dimension/cadence/kind taxonomy" -- 9th dimension docs-alignment registered.
