---
phase: 66-coverage-ratio
plan: 01
subsystem: docs-alignment
tags: [coverage, metric, walker, status-verb, gap-target-view, v0.12.1]
dependency_graph:
  requires:
    - v0.12.0 P64 (catalog + walker + skill + dimension framework live)
    - v0.12.0 P65 (388-row backfill populated the catalog)
  provides:
    - reposix_quality::coverage module (eligible_files, line_count, merge_ranges, covered_lines_for_file, compute_per_file, compute_global, PerFileCoverage)
    - Summary fields: coverage_ratio, lines_covered, total_eligible_lines, coverage_floor
    - Walker BLOCK on coverage_ratio < coverage_floor
    - status verb --top/--all/--json + per-file table sorted worst-first (agent's gap-target view)
    - quality/catalogs/README.md 2x2 alignment-vs-coverage matrix + ratchet semantics
    - CLAUDE.md v0.12.1 in-flight section + P66 H3
    - quality/reports/verdicts/p66/VERDICT.md (Path B in-session, GREEN)
  affects:
    - v0.12.1 P72+ docs-alignment cluster phases (per-file table is their gap-target list)
tech_stack:
  added: []
  patterns:
    - serde back-compat via #[serde(default)] for Summary field additions
    - process-wide cwd_lock Mutex for tests that mutate CWD (cargo runs tests in parallel)
    - merge_ranges: sort-then-fold inclusive 1-based range union (folds overlap AND adjacent)
    - eligible_files mirrors collect_backfill_inputs (single point of update, accept dup for v0.12.1)
key_files:
  created:
    - crates/reposix-quality/src/coverage.rs (310 LOC; 6 public fns + PerFileCoverage struct)
    - crates/reposix-quality/tests/coverage.rs (10 integration tests)
    - .planning/phases/66-coverage-ratio/CONTEXT.md
    - .planning/phases/66-coverage-ratio/66-01-PLAN.md
    - .planning/phases/66-coverage-ratio/66-01-SUMMARY.md
    - quality/reports/verdicts/p66/VERDICT.md
  modified:
    - crates/reposix-quality/src/catalog.rs (Summary +4 fields + default_coverage_floor fn)
    - crates/reposix-quality/src/lib.rs (pub mod coverage)
    - crates/reposix-quality/src/commands/doc_alignment.rs (walk integration + status verb extension)
    - crates/reposix-quality/tests/walk.rs (coverage_floor: 0.0 in synthetic seed)
    - quality/catalogs/doc-alignment.json (summary populated by live walker)
    - quality/catalogs/README.md (2x2 matrix + ratchet semantics section)
    - CLAUDE.md (v0.12.1 H2 + P66 H3)
    - .planning/milestones/v0.12.1-phases/ROADMAP.md (P66 inserted; P67-P71 renumbered)
    - .planning/milestones/v0.12.1-phases/REQUIREMENTS.md (COVERAGE-01..05 added + flipped)
    - .planning/STATE.md (Accumulated Context P66 entry)
decisions:
  - coverage_floor defaults to 0.10 -- low enough that even sparse mining typically clears it; ratchets up by deliberate human commits as gap-closure phases land. Walker NEVER auto-tunes the floor.
  - compute_global empty-state (0,0,0.0) deliberately differs from alignment_ratio's snap-to-1.0; you cannot claim full coverage when there's nothing to cover.
  - merge_ranges folds adjacent ranges (gap of 0 lines) AND overlapping; both yield contiguous coverage in line-coverage semantics.
  - Out-of-eligible row citations warn to stderr + skip (do NOT count); forensic signal for moved/renamed files.
  - Multi-source rows attribute to each cited file independently; no inflation of any single file's coverage.
  - Per-file table sorted ascending by ratio so worst-covered surfaces first -- agent's gap-target view.
  - NO waivers re-added (owner removed v0.12.0 P65 floor_waiver + walker waiver intentionally; pre-push BLOCKING on docs-alignment is the INTENDED state of v0.12.1 until cluster phases close).
metrics:
  duration_min: 45
  completed_date: 2026-04-28
  commits: 6
  tests_added: 18 (10 integration + 8 unit)
  tests_total_after: 47 reposix-quality tests pass
---

# Phase 66 Plan 01: coverage_ratio metric — docs-alignment second axis

**One-liner:** Adds `coverage_ratio = lines_covered / total_eligible_lines` to the docs-alignment dimension as the second axis alongside `alignment_ratio`, with a per-file (worst-first) status table that becomes the agent's gap-target view for v0.12.1 cluster phases.

## What shipped

The docs-alignment dimension now grades along TWO axes:

|                  | high alignment       | low alignment        |
|------------------|----------------------|----------------------|
| **high coverage**| ideal                | extracted; unbound   |
| **low coverage** | tested what we found | haven't started      |

Without coverage, an agent could ship high `alignment_ratio` by extracting only easy claims. The coverage axis closes that loophole.

### Implementation

- **`reposix_quality::coverage`** (NEW module, 310 LOC): `eligible_files()` (mirrors chunker's `collect_backfill_inputs`), `line_count`, `merge_ranges` (folds overlap+adjacent inclusive 1-based ranges), `covered_lines_for_file` (clamps OOB; multi-source independent), `compute_per_file` (sorts ascending by ratio; emits stderr warnings deduped by `(row_id, file)` for out-of-eligible cites), `compute_global` (empty -> `(0, 0, 0.0)`), `PerFileCoverage` struct.
- **Summary struct** gains 4 fields with serde back-compat: `coverage_ratio`, `lines_covered`, `total_eligible_lines`, `coverage_floor` (default 0.10 via `default_coverage_floor` fn). Existing 388-row catalog deserializes unchanged.
- **Walker integration**: AFTER `recompute_summary`, populates global coverage fields. NEW BLOCK condition: `coverage_ratio < coverage_floor` pushes structured stderr line naming `/reposix-quality-backfill` as recovery move OR a deliberate floor-down commit. Walker NEVER auto-tunes the floor. Existing `alignment_ratio < floor` BLOCK unchanged; both can fire.
- **Status verb** gains `--top N` (default 20), `--all`, extended `--json` (emits `{ global, per_file }` with `per_file` ascending by ratio). Table mode shows `== global ==` (both axes with floors + numerator/denominator) AND `== per-file (worst-covered first) ==` (aligned columns, ZERO ROWS hint when `row_count==0 && total_lines>50`, "...(N more; use --all)" footer).
- **Tests**: 10 integration in `tests/coverage.rs` (empty / single / overlap / adjacent / disjoint / multi-source / out-of-eligible / sort + 2 smoke); 8 unit inline `coverage::tests` (merge_ranges variants + global empty). Process-wide `cwd_lock` Mutex serializes CWD mutations because `cargo test` runs in parallel. Existing `tests/walk.rs` synthetic seed catalog gains `"coverage_floor": 0.0` so temp-dir-rooted walker tests don't trip the new BLOCK on unrelated data (Rule 1 fix).

### Schema spec + CLAUDE.md

- `quality/catalogs/README.md` gains "Two metrics: alignment + coverage" subsection: 2x2 matrix, `coverage_floor` ratchet semantics ("default 0.10; ratcheted up by deliberate human commits as gap-closure phases land"), eligible-file-set definition, out-of-eligible warning shape. Notes the symmetric `coverage_floor`-monotone audit row is NOT yet in place; flag if it becomes a footgun.
- CLAUDE.md gains new "v0.12.1 — in flight" H2 + "P66 — coverage_ratio metric" H3 (~22 lines under 30 cap). Banned-words clean.

## Live measured baseline (388-row catalog after walker run)

```
== global ==
  claims_total           388
  claims_bound           171
  claims_missing_test    166
  claims_retire_proposed 41
  claims_retired         0
  alignment_ratio        0.4407   (171 bound / 388 non-retired)
  alignment_floor        0.5000     <-- BLOCKING (intended; v0.12.1 P72+ closes)
  coverage_ratio         0.2055   (1132 covered / 5508 total eligible lines)
  coverage_floor         0.1000     <-- PASS (baseline for ratchet)
  trend_30d              +0.00
  last_walked            2026-04-28T14:32:47Z
```

## Worst-10 per-file table (gap-target view for v0.12.1 P72+)

```
file                                                       total  covered    ratio    rows
------------------------------------------------------------------------------------------
docs/connectors/guide.md                                       9        0    0.000       0
docs/reference/crates.md                                     147        0    0.000       0   <-- ZERO ROWS
docs/security.md                                               9        0    0.000       0
docs/why.md                                                   12        0    0.000       0
docs/social/linkedin.md                                       41        1    0.024       1
docs/social/twitter.md                                        35        1    0.029       1
docs/decisions/003-nested-mount-layout.md                    236        7    0.030       1
docs/how-it-works/filesystem-layer.md                         71        4    0.056       5
docs/guides/integrate-with-your-agent.md                      98        6    0.061       2
docs/reference/git-remote.md                                 130       11    0.085       8
... (36 more; use --all)
```

Top miss: `docs/reference/crates.md` (147 lines, 0 rows). The 4 zero-coverage files at `< 0.000` are stub/redirect shapes (≤12 lines except `crates.md`) — small files don't get the ZERO ROWS hint by design.

## Commit log

| # | Hash | Message |
|---|------|---------|
| A | `0b1ee6f` | docs(p66): scope coverage ratio v0.12.1 phase + renumber P66->P67 etc. |
| B | `83dd3c7` | feat(reposix-quality): coverage module + Summary fields + walk integration + tests |
| - | `45b6526` | fix(.githooks/pre-push): runner exit code masked by `if ! cmd; exit $?` bug (orchestrator-injected during P66 execution; not authored by P66 but interleaved) |
| C+D | `76eeb6c` | feat(reposix-quality): status verb global+per-file + schema spec + CLAUDE.md P66 (folded after the hook-fix interleaving) |
| E | `6512a70` | docs(p66): verifier verdict GREEN -- Path B in-session |
| F | (this commit) | docs(state): P66 SHIPPED -- coverage metric + per-file gap-target view live |

## Carry-forwards (filed in v0.12.1)

- **Symmetric `coverage_floor`-monotone audit.** Currently no structure-dimension row asserts `coverage_floor` is monotone non-decreasing across commits (alignment_floor has `structure/doc-alignment-floor-not-decreased`). If `coverage_floor` becomes a footgun (ratcheted up, then quietly lowered to dodge a BLOCK), file a parallel row.
- **Eligible-set drift between coverage and plan-backfill.** Both `coverage::eligible_files` and `commands::doc_alignment::collect_backfill_inputs` define the same set independently. If one drifts the other will too; consolidate into a single helper in v0.12.1.
- **Out-of-eligible warning batch reduction.** Live walk produces 12 warnings every run (rows citing `crates/*` or `ARCHIVE.md` instead of `REQUIREMENTS.md`). Either widen `eligible_files` to include `crates/*` docs (probably no) OR fix the citations (v0.12.1 work).

## Self-Check: PASSED

Created files exist:
- `crates/reposix-quality/src/coverage.rs` FOUND
- `crates/reposix-quality/tests/coverage.rs` FOUND
- `.planning/phases/66-coverage-ratio/CONTEXT.md` FOUND
- `.planning/phases/66-coverage-ratio/66-01-PLAN.md` FOUND
- `quality/reports/verdicts/p66/VERDICT.md` FOUND

Commits exist:
- `0b1ee6f` (A: scope) FOUND
- `83dd3c7` (B: coverage module) FOUND
- `76eeb6c` (C+D: status + schema + CLAUDE.md) FOUND
- `6512a70` (E: verdict) FOUND
