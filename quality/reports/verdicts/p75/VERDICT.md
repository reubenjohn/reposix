---
phase: 75-bind-verb-hash-fix
verifier: gsd-verifier (Path A, unbiased subagent, zero session context)
verified: 2026-04-29T22:05:00Z
overall_verdict: GREEN
score: 13/13 dimensions PASS (0 PARTIAL, 0 RED, 0 NOT_COVERED)
requirement_closed: BIND-VERB-FIX-01
recommendation: ADVANCE to P76 (surprises absorption)
---

# P75 Verifier Verdict — bind-verb hash-overwrite fix

**Phase:** 75-bind-verb-hash-fix
**Requirement:** BIND-VERB-FIX-01
**Plan:** `.planning/phases/75-bind-verb-hash-fix/PLAN.md`
**Summary:** `.planning/phases/75-bind-verb-hash-fix/SUMMARY.md`
**Live evidence:** `quality/reports/verdicts/p75/walk-after-fix.txt`
**Overall verdict:** **GREEN — advance to P76.**

The bind-verb hash-overwrite bug (root-caused in HANDOVER §4 and broadened
by P74's SURPRISES-INTAKE entry) is fixed by construction: `verbs::bind` now
preserves `row.source_hash` on `Source::Multi` paths and refreshes it only
when the result is `Source::Single`. The walker's first-source compare
invariant is restored. All thirteen grading dimensions pass cleanly; no
PARTIAL, no RED, no NOT_COVERED.

## Dimension table

| # | Dimension | Verdict | Evidence |
|---|-----------|---------|----------|
| 1 | Regression tests added (3) and passing | PASS | `cargo test -p reposix-quality --test walk` shows 6 passed / 0 failed; the three new fns (`walk_multi_source_stable_no_false_drift`, `walk_multi_source_first_drift_fires_stale`, `walk_single_source_rebind_heals_after_drift`) exist at `crates/reposix-quality/tests/walk.rs:329, 402, 453`. |
| 2 | Pre-fix RED demonstration | PASS | Verified by checking out commit `5f419a1` (RED) into a separate worktree and running `cargo test -p reposix-quality --test walk`: result `5 passed; 1 failed`. The failing test is `walk_multi_source_stable_no_false_drift` with the expected assertion message *"BIND-VERB-FIX-01: source_hash must be preserved on Single->Multi promotion (== hash of first source)"*. Test commit `5f419a1` precedes fix commit `69a30b0` — true TDD ordering. |
| 3 | D-01 fix correctness | PASS | Read `crates/reposix-quality/src/commands/doc_alignment.rs:285-329` post-fix. The patch introduces `let result_is_single = sources.len() == 1;` and gates `row.source_hash = Some(src_hash);` behind that flag. The `else: preserve existing row.source_hash` comment makes the invariant load-bearing. New-row arm at line 485 still unconditionally writes `source_hash` (correct — new row IS a Single). All three semantic cases (Single, Single→Multi promotion, Multi append) covered exactly per CONTEXT.md D-01. |
| 4 | D-02+D-03+P74 broadening test scope | PASS | All three named tests exist with the specified case shape: Test A (Multi-stable-no-drift), Test B (Multi-first-drift-fires), Test C (Single-rebind-heal). Test B's inline comment explicitly notes the path-(a) limitation and refuses to assert the "B drifts → stays BOUND" sub-case (the right call — asserting it would lock in the limitation). |
| 5 | D-04 live smoke evidence | PASS | `quality/reports/verdicts/p75/walk-after-fix.txt` exists with stdout/stderr capture, BEFORE/AFTER block (linkedin row STALE→BOUND, hash `1a19b86e…` → `7a1d7a4e…`), and net-new STALE_DOCS_DRIFT count = 0. The two pre-existing STALE rows are explicitly named and carved out of P75 scope. |
| 6 | D-05 path-(a) tradeoff documented | PASS | CLAUDE.md:400-403 explicitly states *"Path-(a) tradeoff: the walker only watches `source.as_slice()[0]`. Drift in non-first sources of a `Multi` row will NOT fire `STALE_DOCS_DRIFT`."* with a pointer to v0.13.0 carry-forward. |
| 7 | MULTI-SOURCE-WATCH-01 filed | PASS | `.planning/milestones/v0.13.0-phases/CARRY-FORWARD.md` exists with the entry. Acceptance criteria spell out the schema migration (`source_hash: Option<String>` → `source_hashes: Vec<String>`), the parallel-array invariant, and the migration path via `serde(default)` + one-time backfill. |
| 8 | D-08 CLAUDE.md ≤20 lines | PASS | Awk-counted CLAUDE.md:390-407 = 18 lines (heading + body + trailing blank). Under the 20-line ceiling. Content covers all four required points (invariant, path-(a) tradeoff, v0.13.0 pointer, regression-test pointer). |
| 9 | Atomic commits in canonical order | PASS | `git log --oneline 5f419a1^..HEAD` shows 5 commits in canonical RED→GREEN→SMOKE→DOCS→SUMMARY order: `5f419a1` (test RED) → `69a30b0` (fix GREEN) → `9e07028` (smoke + linkedin heal) → `c13e8e5` (CLAUDE.md + carry-forward) → `d7c8173` (SUMMARY, orchestrator-landed). Each P75 commit cites BIND-VERB-FIX-01 + Phase 75. |
| 10 | Catalog delta plausibility | PASS | `jq '.summary' quality/catalogs/doc-alignment.json` confirms `claims_bound: 329` and `alignment_ratio: 0.9189944…`. Match SUMMARY's claim of 328→329 (+1) and 0.9162→0.9190. The +1 is solely the linkedin row tipping from STALE_DOCS_DRIFT → BOUND. No other rows changed — surgical fix as advertised. |
| 11 | D-09 honesty / OP-8 | PASS | The SUMMARY's claim that "Test C passed pre-fix" is independently verified: pre-fix worktree run shows `walk_single_source_rebind_heals_after_drift ... ok`. The P74 broadening was therefore correctly identified as a procedural finding (walker contract: walks don't heal, binds do), not a second bug. The honesty grade is direct, falsifiable, and falsifiable in either direction — the verifier could have caught a lie. None found. SURPRISES-INTAKE / GOOD-TO-HAVES "none" entries are credible: the fix landed cleanly. |
| 12 | No prohibited actions | PASS | No push/tag/publish/confirm-retire commits in the P75 chain. The `Source` enum at `crates/reposix-quality/src/catalog.rs` is unchanged (path b is explicitly out of scope and filed forward). No new env vars, no new CLI flags, no new build hooks. |
| 13 | Pre-existing STALE rows untouched | PASS | `jq '.rows[] \| select(.last_verdict == "STALE_DOCS_DRIFT")'` returns exactly the two pre-P75 rows (`polish-03-mermaid-render`, `cli-subcommand-surface`). Their P72 SURPRISES-INTAKE provenance is preserved; P75 did not silently update or paper over them. |

## Findings (additional notes)

### Test C honesty cross-check (high-confidence positive)

The SUMMARY explicitly notes that Test C was *confirmed-not-a-bug* — i.e.
the test PASSED pre-fix, which means the existing Single re-bind path was
already correct, and the P74 linkedin row stayed STALE because nobody
re-ran `bind` after the prose edit (walks don't heal). This is the kind
of finding an unbiased verifier should be able to falsify if false. I
ran `git worktree add /tmp/p75-prefix-check 5f419a1 && cd /tmp/p75-prefix-check
&& cargo test -p reposix-quality --test walk` and observed:

```
test walk_multi_source_stable_no_false_drift ... FAILED
test walk_multi_source_first_drift_fires_stale ... ok
test walk_single_source_rebind_heals_after_drift ... ok
```

Test C passed pre-fix. The SUMMARY's interpretation is correct.

### CLAUDE.md word-budget

CLAUDE.md:390-407 is 18 lines (counted via `awk 'NR>=390 && NR<=407' CLAUDE.md | wc -l`). Under the 20-line CONTEXT.md D-08 budget. The progressive-disclosure principle (OP-4) is honoured: the H3 names the invariant + the tradeoff + the carry-forward + the test pointer; the long-form rationale lives in PLAN.md and SURPRISES-INTAKE.md.

### Banned-words check

`grep -n -E "FUSE filesystem|MCP tool schemas|custom CLI" CLAUDE.md` returns
only the legitimate "no MCP tool schemas, no custom CLI, no FUSE mount"
positioning line in the elevator pitch. No banned-word violations.

### Memory-budget compliance

The fix lives in one crate (`reposix-quality`); the test run is
`cargo test -p reposix-quality --test walk` (single crate, single
invocation). The release-binary smoke step uses a single
`cargo build -p reposix-quality --release` invocation followed by
direct binary calls. CLAUDE.md memory-budget rules respected.

### Catalog smoke pre/post mapping

| Metric | Pre-P75 (per SUMMARY) | Post-P75 (live `jq` on catalog) | Match |
|--------|-----------------------|----------------------------------|-------|
| `claims_bound` | 328 → 329 (+1) | 329 | yes |
| `alignment_ratio` | 0.9162 → 0.9190 (+0.0028) | 0.918994… | yes |
| linkedin row `last_verdict` | `STALE_DOCS_DRIFT` → `BOUND` | `BOUND` | yes |
| linkedin row `source_hash` | `1a19b86e…` → `7a1d7a4e…` | `7a1d7a4ea4c7891bff5bedc2e0191633dcd60d840625efd13aa63427c33768ed` | yes |
| Pre-existing STALE rows | 2 (`polish-03-mermaid-render`, `cli-subcommand-surface`) | 2 (same) | yes |

## Recommendations

1. **Advance to P76.** All P75 deliverables ship cleanly. The +2 reservation slot 1 (P76 surprises absorption) can now drain `SURPRISES-INTAKE.md` confident that the P74 linkedin entry is closed (linkedin row already BOUND in the catalog) and the P72 entry remains the only outstanding STALE inventory.
2. **No loop-back required.** The verdict is GREEN across all dimensions; no PARTIAL items requiring follow-up before milestone close.
3. **MULTI-SOURCE-WATCH-01 is durably filed.** The v0.13.0 carry-forward entry has acceptance criteria precise enough to be actionable without re-discovery. The next milestone planner can pick it up directly.

---

*Verified by gsd-verifier (Path A, unbiased subagent), 2026-04-29.*
*Worktree probe: `/tmp/p75-prefix-check` (cleaned up post-grading).*
