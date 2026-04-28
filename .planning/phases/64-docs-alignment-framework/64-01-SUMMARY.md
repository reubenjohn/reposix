---
phase: 64-docs-alignment-framework
plan: 01
subsystem: quality-gates
tags: [quality-gates, docs-alignment, catalog-first, structure-dim, freshness-invariants, skill-scaffolding]

requires:
  - phase: 56-63
    provides: "v0.12.0 Quality Gates framework -- catalog-first rule (PROTOCOL.md Step 3), structure-dimension precedent (P57), subagent-graded skill precedent (P61 reposix-quality-review), per-dimension catalog file convention"
provides:
  - "quality/catalogs/doc-alignment.json (NEW) -- empty-state catalog seed: schema_version='1.0', summary block (9 keys; alignment_ratio=1.0, floor=0.50), rows=[]"
  - "3 new structure-dim freshness rows in quality/catalogs/freshness-invariants.json: doc-alignment-catalog-present (P0 pre-push), doc-alignment-summary-block-valid (P0 pre-push), doc-alignment-floor-not-decreased (P1 weekly)"
  - "3 new --row-id branches in quality/gates/structure/freshness-invariants.py wired into DISPATCH"
  - "quality/gates/docs-alignment/README.md (NEW, 59 lines) -- dimension home"
  - "quality/catalogs/README.md docs-alignment subsection (38 added lines) -- row schema + 8-state row state machine + summary block + floor semantics"
  - "Preflight skill scaffolding: .claude/skills/reposix-quality-doc-alignment/{SKILL.md, refresh.md, backfill.md, prompts/extractor.md, prompts/grader.md} + thin slash-command skills .claude/skills/reposix-quality-{refresh,backfill}/SKILL.md"
affects: [64-02, 64-03, 64-04, 64-05, 65-backfill]

tech-stack:
  added: []
  patterns:
    - "Catalog-first commit per phase (PROTOCOL.md Step 3) -- continued from P56-P63 precedent: end-state in git BEFORE code"
    - "Append-only freshness-invariants.json mutation (existing 13 rows preserved verbatim)"
    - "Empty-state seed pattern: schema valid, zero rows, alignment_ratio=1.0 by definition (claims_total=0 -> denom max(1, 0-0) -> 0/1 = 1.0)"
    - "Floor-monotonicity audit via git log walk (no separate state file; reconstruct from history)"
    - "Preflight skill committal: orchestrator writes skill files BEFORE the catalog-first commit so permission prompts fire at planning time, not execution time"

key-files:
  created:
    - "quality/catalogs/doc-alignment.json -- empty-state seed"
    - "quality/gates/docs-alignment/README.md -- dimension home"
    - ".claude/skills/reposix-quality-doc-alignment/SKILL.md -- umbrella skill"
    - ".claude/skills/reposix-quality-doc-alignment/refresh.md -- refresh playbook"
    - ".claude/skills/reposix-quality-doc-alignment/backfill.md -- backfill playbook"
    - ".claude/skills/reposix-quality-doc-alignment/prompts/extractor.md -- shard extractor subagent prompt"
    - ".claude/skills/reposix-quality-doc-alignment/prompts/grader.md -- per-row grader subagent prompt"
    - ".claude/skills/reposix-quality-refresh/SKILL.md -- thin slash-command entry"
    - ".claude/skills/reposix-quality-backfill/SKILL.md -- thin slash-command entry"
  modified:
    - "quality/catalogs/freshness-invariants.json -- 3 new rows appended (existing 13 untouched)"
    - "quality/gates/structure/freshness-invariants.py -- 3 new verifier branches + DISPATCH entries (+178 lines)"
    - "quality/catalogs/README.md -- 'docs-alignment dimension' subsection (+38 lines)"

key-decisions:
  - "Followed existing freshness-invariants.py dispatch convention (--row-id <slug>) NOT the additional_context's mention of --check <slug>; the additional_context appears to have predated the actual codebase pattern. Existing 11 verifier branches all use --row-id, so consistency with precedent wins."
  - "Used Edit tool to append rows to freshness-invariants.json rather than rewriting the file; preserves existing 13 rows verbatim per plan must_haves.truths."
  - "Floor-not-decreased verifier: walk git log --reverse for the catalog file, git show each commit's content, parse summary.floor, compare to previous; emit asserts_failed naming offending SHA on regression. On a freshly-seeded catalog with <2 historical commits, pass with a 'nothing to compare yet' note."
  - "quality/catalogs/README.md docs-alignment subsection landed at 38 added lines (plan suggested <=30). The mandated content (row schema + 8-state row state machine + summary block + floor_waiver shape) does not compress further without losing normative detail. Documented the 8 state transitions, summary formula, and floor semantics in full."
  - "Did NOT modify .planning/STATE.md or .planning/config.json (they had pre-existing unrelated edits) -- staged exactly the 12 P64 Wave 1 files."

patterns-established:
  - "Empty-state catalog seed must validate: schema_version, summary keys, rows=[], alignment_ratio recomputable. Three structure-dim rows guard each invariant separately so failure mode is precise."
  - "Floor-monotonicity audit via git log walk -- weekly cadence (audit signal, not blocking pre-push) -- reconstructs prior catalog states from history without a sidecar state file."
  - "Preflight skill committal -- orchestrator writes skill files before the catalog-first commit so the catalog-first commit ships normative inputs (refresh/backfill playbooks, extractor/grader prompts) alongside the contracts they are normative inputs to."

requirements-completed: [DOC-ALIGN-04, DOC-ALIGN-06]

duration: 25min
completed: 2026-04-28
---

# Phase 64 Plan 01: Catalog-first commit -- doc-alignment seed + 3 freshness rows + skill scaffolding (preflight) Summary

**P64 Wave 1 ships the GREEN contract for the docs-alignment dimension before any binary code lands -- empty-state catalog seed, 3 structure-dim freshness invariants guarding the catalog file, and the orchestrator-preflight skill scaffolding (umbrella + 2 thin slash-command skills) that Waves 2-5 implement against.**

## Performance

- **Duration:** ~25 min
- **Started:** 2026-04-28T07:42:00Z (approximate, plan-execution start)
- **Completed:** 2026-04-28T08:07:33Z
- **Tasks:** 6 (read context bundle; write doc-alignment.json; append 3 rows to freshness-invariants.json; extend freshness-invariants.py with 3 branches; write quality/gates/docs-alignment/README.md; update quality/catalogs/README.md)
- **Files modified:** 12 (3 modified, 9 created)
- **Commits:** 1 (atomic per plan)

## Accomplishments

- **Catalog-first contract live.** `quality/catalogs/doc-alignment.json` exists, parses, and has the 9-key summary block + zero rows; `alignment_ratio == 1.0` by definition for `claims_total == 0`. Becomes the active catalog for the docs-alignment dimension; P65 populates rows.
- **Three structure-dim guards on the catalog.** `doc-alignment-catalog-present` (P0 pre-push) + `doc-alignment-summary-block-valid` (P0 pre-push) graded PASS in the pre-push runner; `doc-alignment-floor-not-decreased` (P1 weekly) graded PASS by the weekly runner. Invariants now self-check on every push.
- **Dimension-home scaffolding ships.** `quality/gates/docs-alignment/README.md` (59 lines, under the 80-line plan cap) mirrors the structure-dim README precedent. `quality/catalogs/README.md` gains the "docs-alignment dimension" subsection documenting row schema, 8-state row state machine, summary block formula, and floor semantics -- normative reference for Waves 2-5.
- **Preflight skill scaffolding committed.** All 7 orchestrator-preflight skill files (umbrella + 2 thin slash-command skills) tracked in the same atomic commit as the catalog. Subsequent waves (2-5) implement the binary surface that those playbooks invoke.

## Task Commits

Single atomic commit per plan must_haves.truths line 27:

1. **All 6 task outputs in one commit** -- `d0d4730` (`docs(p64): catalog-first -- doc-alignment seed + 3 freshness rows + skill scaffolding (preflight)`) -- 12 files changed, 907 insertions, 0 deletions.

## Files Created/Modified

### Created (9 files)

- `quality/catalogs/doc-alignment.json` -- empty-state catalog seed (schema_version, summary, rows wrapper); becomes active for the docs-alignment dimension.
- `quality/gates/docs-alignment/README.md` -- dimension home (59 lines): 1-line summary, quick-start, empty row table, structure-dimension guards table, conventions, cross-references.
- `.claude/skills/reposix-quality-doc-alignment/SKILL.md` -- umbrella skill (refresh + backfill modes; top-level only; cross-refs to architecture + execution-modes briefs).
- `.claude/skills/reposix-quality-doc-alignment/refresh.md` -- single-doc refresh playbook (orchestrator-normative).
- `.claude/skills/reposix-quality-doc-alignment/backfill.md` -- full audit playbook per P65 brief.
- `.claude/skills/reposix-quality-doc-alignment/prompts/extractor.md` -- per-shard Haiku extractor subagent prompt.
- `.claude/skills/reposix-quality-doc-alignment/prompts/grader.md` -- per-row Opus grader subagent prompt.
- `.claude/skills/reposix-quality-refresh/SKILL.md` -- thin slash-command entry; delegates to umbrella.
- `.claude/skills/reposix-quality-backfill/SKILL.md` -- thin slash-command entry; delegates to umbrella.

### Modified (3 files)

- `quality/catalogs/freshness-invariants.json` -- 3 new rows appended under `rows[]` (existing 13 rows preserved verbatim with status/last_verified intact). New rows: `structure/doc-alignment-catalog-present` (P0 pre-push); `structure/doc-alignment-summary-block-valid` (P0 pre-push); `structure/doc-alignment-floor-not-decreased` (P1 weekly).
- `quality/gates/structure/freshness-invariants.py` -- 3 new verifier branches (`verify_doc_alignment_catalog_present`, `verify_doc_alignment_summary_block_valid`, `verify_doc_alignment_floor_not_decreased`) wired into DISPATCH; +178 lines.
- `quality/catalogs/README.md` -- 'docs-alignment dimension' subsection added (+38 lines): row schema (id/claim/source/source_hash/test/test_body_hash/last_verdict/last_run/last_extracted/last_extracted_by), 8-state row state machine (BOUND / MISSING_TEST / STALE_DOCS_DRIFT / STALE_TEST_DRIFT / STALE_TEST_GONE / TEST_MISALIGNED / RETIRE_PROPOSED / RETIRE_CONFIRMED), summary block + alignment_ratio formula, floor semantics (audit-only, no waiver -- monotone non-decreasing by design).

## Decisions Made

- **Dispatch convention (--row-id, not --check).** The additional_context block in the prompt referenced `--check <slug>` in several spots, but every existing freshness-invariants.py verifier uses `--row-id <slug>` as the dispatch flag. Followed precedent. The catalog rows in freshness-invariants.json all use `args: ["--row-id", "<slug>"]`.
- **README.md docs-alignment subsection at 38 lines (plan suggested <=30).** The mandated content (row schema + 8-state row state machine + summary block formula + floor_waiver shape) does not compress to 30 lines without dropping normative detail required by Waves 2-5. Documented all 4 mandated items in full -- minor overshoot.
- **Floor-not-decreased verifier algorithm.** Walks `git log --reverse --format=%H -- quality/catalogs/doc-alignment.json` (all commits touching the file, oldest first). For each SHA, runs `git show <sha>:<path>` and parses `summary.floor`. On floor regression, names the offending SHA in `asserts_failed`. Single-commit history (this plan's first commit) passes with a "nothing to compare yet" note + `commits_walked` recorded in the artifact.
- **Did not stage .planning/STATE.md / .planning/config.json.** Both had pre-existing unrelated session edits in the working tree. Staged exactly the 12 P64 Wave 1 files. Orchestrator owns post-wave roadmap/state writes per phase prompt.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking discrepancy] Dispatch flag mismatch between prompt context and codebase precedent**
- **Found during:** Task 3 (extending freshness-invariants.py)
- **Issue:** The plan's additional_context block referenced `--check <slug>` as the dispatch flag for new verifier branches, but every existing branch in `freshness-invariants.py` (11 branches across P57/P58/P62) uses `--row-id <slug>`. Catalog rows match: `args: ["--row-id", "<slug>"]`.
- **Fix:** Followed existing `--row-id` precedent. Added 3 functions named `verify_doc_alignment_*` and wired them into the existing `DISPATCH` dict keyed by row-id slug. Catalog row args use `--row-id`.
- **Files modified:** `quality/gates/structure/freshness-invariants.py`, `quality/catalogs/freshness-invariants.json`
- **Verification:** All 3 verifier invocations exit 0; pre-push runner picks up both pre-push rows and graded PASS.
- **Committed in:** d0d4730

---

**Total deviations:** 1 auto-fixed (Rule 3)
**Impact on plan:** Trivial; the plan's contract was clear (3 new check branches matching 3 row IDs). Following the existing dispatch convention preserves single-source-of-truth and lets the runner invoke the verifier the same way it invokes every other structure-dim check.

## Issues Encountered

- **Pre-commit soft warning on freshness-invariants.py size.** The personal pre-commit hook warned `quality/gates/structure/freshness-invariants.py is 25209 chars (limit: 15000)` -- a soft warning (commit succeeded). The 25K size is the result of P57+P62+P64 verifier accretion (now ~580 lines including this plan's +178). This is a known carry-forward already journaled in `.planning/STATE.md` line 22: "helper-module extraction for 402-LOC freshness-invariants.py -- P62 carry-forward; reassess in v0.12.1 if Wave 6 flagged it." After this plan it's now ~580 LOC -- the v0.12.1 reassess is more justified. Not blocking; deferred per existing carry-forward.

## Self-Check: PASSED

### Files exist on disk

- `quality/catalogs/doc-alignment.json` -- FOUND
- `quality/gates/docs-alignment/README.md` -- FOUND
- `quality/catalogs/freshness-invariants.json` (16 rows, 3 new appended) -- FOUND
- `quality/gates/structure/freshness-invariants.py` (DISPATCH includes 3 new keys) -- FOUND
- `quality/catalogs/README.md` (docs-alignment subsection at line 77) -- FOUND
- `.claude/skills/reposix-quality-doc-alignment/SKILL.md` + 4 sibling files -- FOUND
- `.claude/skills/reposix-quality-{refresh,backfill}/SKILL.md` -- FOUND

### Commits exist

- `d0d4730 docs(p64): catalog-first -- doc-alignment seed + 3 freshness rows + skill scaffolding (preflight)` -- FOUND in `git log --oneline -3`

### Verification commands

- `python3 -c "import json; ..."` empty-state invariants -- exit 0 (PASS)
- `python3 quality/gates/structure/freshness-invariants.py --row-id structure/doc-alignment-catalog-present` -- exit 0
- `python3 quality/gates/structure/freshness-invariants.py --row-id structure/doc-alignment-summary-block-valid` -- exit 0
- `python3 quality/gates/structure/freshness-invariants.py --row-id structure/doc-alignment-floor-not-decreased` -- exit 0
- `python3 quality/runners/run.py --cadence pre-push` -- 21 PASS, 0 FAIL, 3 WAIVED, 0 NOT-VERIFIED, exit 0 (the 2 new pre-push rows graded PASS)
- `bash scripts/banned-words-lint.sh` -- exit 0

## Out of Scope (Deferred to Subsequent Waves)

Per plan "Out of scope" section -- subsequent waves of P64 own:

- Crate skeleton (`crates/reposix-quality/`) -- Wave 2
- Subcommand implementations (`bind`, `mark-missing-test`, `propose-retire`, `confirm-retire`, `verify`, `walk`, `plan-refresh`, `plan-backfill`, `merge-shards`, `status`, `run`) -- Waves 2-4
- Hash binary (`quality/gates/docs-alignment/hash_test_fn`) -- Wave 4 (or as `[[bin]]` inside reposix-quality crate per CONTEXT.md specifics)
- Hook wiring (`scripts/hooks/pre-push`) -- Wave 5
- `quality/PROTOCOL.md` updates (two project-wide principles) -- Waves 5-6
- CLAUDE.md updates (dimension matrix row + P64 H3 subsection) -- Wave 6 (or last code-touching wave)
- Verifier subagent dispatch (`quality/reports/verdicts/p64/VERDICT.md`) -- Wave 11 (phase close)

## Cross-references

- `.planning/phases/64-docs-alignment-framework/64-01-PLAN.md` -- the plan executed.
- `.planning/phases/64-docs-alignment-framework/64-CONTEXT.md` -- locked decisions and canonical refs.
- `.planning/research/v0.12.0-docs-alignment-design/02-architecture.md` -- catalog row schema + summary block + state machine + hash semantics (normative for downstream waves).
- `.planning/research/v0.12.0-docs-alignment-design/05-p64-infra-brief.md` -- P64 implementation spec; Section "Catalog rows P64 ships" was the contract this plan implements.
- `quality/PROTOCOL.md` Step 3 -- catalog-first rule.
- `quality/catalogs/README.md` § "docs-alignment dimension" -- row schema spec landed by this plan.
- `quality/gates/docs-alignment/README.md` -- dimension home landed by this plan.
