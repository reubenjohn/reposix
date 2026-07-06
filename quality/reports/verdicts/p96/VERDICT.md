# P96 Phase-Close Verdict — v0.13.0 (OP-8 Slot 1: SURPRISES-INTAKE drain + quality-runner self-mutation fix)

**Overall: GREEN**

- **Graded HEAD:** `0bdd752` (`0bdd75238369b18fcd48c7c96b570079bcd9f241`) — local HEAD == `origin/main` (push-before-verifier cadence satisfied), tree clean before and after grading.
- **Verifier:** unbiased phase-close subagent (did not execute the work). Graded goal-backward against reality — ran the gates, the runner, and the cargo suites; SUMMARY/commit-message claims treated as hypotheses, not evidence.
- **Cargo discipline:** one invocation at a time, `-p <crate>`, `CARGO_BUILD_JOBS=2`. No concurrent build.
- **Keystone regression guard:** every catalog byte was md5-snapshotted at grading entry and again at exit → **CATALOGS_UNCHANGED_ACROSS_ENTIRE_VERIFICATION** (a validate-only grading pass did not mutate a single catalog byte — the very fix under test).

---

## Contract Item 1 — Self-mutation fix (keystone, GRADE/PERSIST split) — **PASS**

**Gate run:**
- `bash quality/gates/structure/catalog-immutable-on-read.sh` → **exit 0**, all 4 asserts PASS (3 hermetic + 1 real-tree breadth). Catalog md5s unchanged across the gate (`NO_CATALOG_DRIFT_FROM_GATE`).

**Independent snapshot + real pre-push validate-only (the exact regressing cadence):**
- md5-snapshot all `quality/catalogs/*.json` → `python3 quality/runners/run.py --cadence pre-push` (NO `--persist`) → re-snapshot → **diff empty** (`ZERO_CATALOG_BYTES_CHANGED_ON_PREPUSH`), `git status quality/catalogs/` clean.
- The runner printed exactly the smoking gun, now defused: `note: validate-only run -- 1 catalog(s) have status flips NOT persisted (docs-build.json)`. Pre-fix this flip landed on disk and dirtied every push; post-fix it is graded in-memory, artifact-written, and **not** persisted. `summary: 54 PASS, 1 FAIL, 0 PARTIAL, 1 WAIVED, 0 NOT-VERIFIED -> exit=0`.

**Catalog-first ordering + contract fidelity:**
- Row `structure/catalog-immutable-on-read` (`quality/catalogs/freshness-invariants.json`) and its verifier were both introduced by `1a9f2f2` ("catalog-first — RED pre-fix"), which **precedes** the fix `2359c63`. Verifier subagent reads a row that predates the implementation.
- Row `expected.asserts` (3 strings) match the gate's Layer-A `pass()` strings **verbatim** (gate lines 73–75 == row lines 1149–1151). No test-name-lies gap.
- `run.py` gates persistence correctly: `main()` lines 485–487 — `if catalog_dirty(...): if args.persist: save_catalog(...)`; validate-only path appends to `pending_mint` and prints the note (491–496).

**Mint path NOT frozen (over-correction check):** Layer-A assert 3 proves `--persist` still writes; and Item 4 below is a live `--persist` mint that landed 3 real flips. Grades gated, not frozen.

**Workaround retirement (fix-it-twice):**
- No `.claude/hooks/` script executes `git checkout HEAD -- quality/catalogs/`. No executable (non-comment) `.sh` line anywhere performs it.
- The only surviving reference is `quality/PROTOCOL.md:151` which **forbids re-adding** it. `f19abb6` retired the band-aid across PROTOCOL, runners/README, `94-D4-sweep.sh`, COORD-HANDOFF, SURPRISES-INTAKE, CONSULT-DECISIONS.
- Hermetic unit proof standalone: `python3 -m unittest quality.runners.test_run` → **4 tests OK** (cargo-free).

---

## Contract Item 2 — Wave 2a code fixes (source_hashes bind-state + summary.last_walked) — **PASS**

- `cargo test -p reposix-quality --test walk` → **14 passed; 0 failed**. Full crate `cargo test -p reposix-quality` → **all binaries green, 0 failed** (23+9+12+14+... across every test target).
- Named regression tests present and green: `walk_multi_source_stable_no_false_drift`, `walk_detects_source_drift_and_preserves_stored_hashes`, `walk_single_source_rebind_heals_after_drift`, `bind_refreshes_summary_last_walked`.
- **Test-name-vs-assertion honesty spot-check (Confirmation-Bias-Counter):**
  - `walk_multi_source_stable_no_false_drift` asserts `source_hash` preserved on Single→Multi promotion (walk.rs:405–407, "BIND-VERB-FIX-01"). Name matches assertion.
  - `bind_refreshes_summary_last_walked` asserts a bind stamps `summary.last_walked` when null AND advances it past a stale sentinel (walk.rs:1069–1088). Name matches assertion.

---

## Contract Item 3 — Wave 2b (same-second CREATE cache-coherence + list_changed_since disposition) — **PASS**

- `cargo test -p reposix-cache --test cache_coherence` → **4 passed; 0 failed**, incl. target `same_second_created_record_resolvable_after_delta_sync` (fn at cache_coherence.rs:399) and ADR-010 rows `ghost_oid_map_row_pruned_after_upstream_delete`, `head_tree_blobs_resolvable_after_{seed,same_second_delta}_sync`.
- Full crate `cargo test -p reposix-cache` → **all binaries green, 0 failed** (34+9+6+... across every target; slowest bin 17.45s, all OK).
- `list_changed_since` finding **RESOLVED-false-alarm** in `SURPRISES-INTAKE.md:79`. The `[SELF]` CONSULT-DECISIONS ledger entry exists (`.planning/CONSULT-DECISIONS.md:416`, "list_changed_since under-materialization — false alarm (DP-2 prove-before-fix)") and cites the exact CREATE-path repro. All-blob-materialization fix correctly REJECTED as an ARCH-01 (lazy blob:none) violation.

---

## Contract Item 4 — P94 re-mint (clean, keystone-unblocked) — **PASS**

On committed disk (`python3` catalog-row scan):

| Row | Disk status | Contract | Match |
|---|---|---|---|
| `agent-ux/p94-pagination-prune-completeness-gate` | **PASS** | PASS (verdict GREEN) | ✓ |
| `agent-ux/p94-git243-fallback-sentinel` | **PASS** | PASS (verdict GREEN) | ✓ |
| `structure/p94-catalog-freshness-sweep` | **PASS** | PASS (verdict GREEN) | ✓ |
| `docs-build/p94-badges-real-vs-transient` | **NOT-VERIFIED** | NOT-VERIFIED (must NOT be PASS) | ✓ |

- The three PASS rows match `quality/reports/verdicts/p94/VERDICT.md` (rows 1/2/4 GREEN).
- `docs-build/p94-badges-real-vs-transient` is correctly **NOT-VERIFIED** on disk — a genuine badge-non-determinism flake. (Note: the p94 VERDICT itself froze a transient PASS for this row at its lucky grading moment; see NOTICED.)
- Commit `0bdd752` diff touches **exactly 3** catalog rows (three `NOT-VERIFIED → PASS` status flips + 3 `last_verified` stamps), and its message documents "Intentionally LEFT NOT-VERIFIED: docs-build/p94-badges." No other row diverged.

---

## Contract Item 5 — Intake hygiene (terminal↔active split, zero row loss) — **PASS**

**Reconciliation (independent count, true entry markers, pre-split HEAD `9a818b7`):**

- **SURPRISES** (`## YYYY-MM-DD` h2 entries): pre-split = **67**. Post-split = 30 active + 5 (ARCHIVE-P78-P88) + 32 (ARCHIVE-P89-P97) = **67**. ✓ No entry lost. Working file carries a `<details>` manifest re-listing the 37 relocated entries as pointers (not duplicate bodies).
- **GOOD-TO-HAVES** (17 `## GOOD-TO-HAVES-NN` + 26 `## YYYY-MM-DD`): pre-split = **43**. Post-split = 40 active (17 numbered + 23 dated) + 3 archived = **43**. ✓
- **Ledger `D-P96-01`** present (`.planning/CONSULT-DECISIONS.md:371`).
- **8 batched RAISE filings** (`9a818b7`) all landed in the files (not just the commit message): (1) STATE.md strict-YAML guard SURPRISES:586; (2) committed-catalog-status-lag SURPRISES:411; (3) `--persist` load-refusal hardening SURPRISES:642; (4) source_hash retirement pre-audit append SURPRISES:572–580; (5) doc_alignment.rs 71k monolith split GTH:821; (6) cache_coherence.rs split GTH:853; (7) run.py persist-gate extract + dead-condition cleanup GTH-06:161; (8) list_changed_since `>`-boundary efficiency residual GTH:385.

**ACCEPTED DEVIATION (not graded RED, per charter):** working intake files remain >20k (active corpus alone is 63k/81k); `structure/file-size-limits` = **WAIVED** to 2026-08-08. Terminal↔active separation with zero row loss is the delivered contract, honored.

---

## NOTICED (ownership deliverable, OD-3)

1. **p94 VERDICT froze a transient badge PASS.** `quality/reports/verdicts/p94/VERDICT.md:24` records `docs-build/p94-badges-real-vs-transient` as **PASS (P2)**, yet the gate is genuinely non-deterministic and reads NOT-VERIFIED on disk. Not a P96 regression (accepted deviation; the 3 minted rows aren't in the pre-push cadence), but the **P97 milestone-close verdict must not inherit p94's transient-PASS badge claim** — grade it NOT-VERIFIED honestly. → RAISE.
2. **Keystone gate's real-tree breadth check runs `--cadence pre-commit`, not the pre-push cadence where the bug actually bit** (deliberate, to avoid recursion + double-cargo; documented in the gate header). Coverage of the pre-push-only `docs-build.json` flip therefore rests on Layer-A's hermetic synthetic-flip proof (which drives `run.main()` cadence-agnostically) rather than a real pre-push. Adequate, but I closed the residual gap manually by running a real `--cadence pre-push` validate-only here (zero drift). Low concern; noting for transparency.
3. **run.py dead condition already self-filed:** `if pending_mint and not args.persist:` (`run.py:491`) carries a redundant `not args.persist` (`pending_mint` only fills in the `else args.persist` branch). Correctly filed as GTH-06 down-payment (GOOD-TO-HAVES.md:161) — good hygiene, no action for P96.
4. **STATE.md not advanced** — still `status: executing-p95-post-close-drain`, `next_phase: P96`. Correct: STATE advance to P97 is the coordinator's action per charter; verifier did not touch it.

## RAISE LIST for P97 (milestone-close, OP-8 Slot 2 / OP-9)

- **[P0-honesty]** P97 milestone-close verdict: grade `docs-build/p94-badges-real-vs-transient` NOT-VERIFIED (do not inherit p94's frozen transient PASS). The `pre-release-real-backend` 9th probe is separately non-skippable.
- **[file-size]** Intake >20k waiver expires **2026-08-08**; P97 (or a P97-filed P-arch decision) must renew or resolve the active-corpus fragmentation question, not silently let the waiver lapse.
- **[debt-drain, already filed]** doc_alignment.rs 71k + cache_coherence.rs 23.4k module splits, run.py persist-gate extraction + dead-condition drop, `--persist` load-refusal hardening, source_hashes parallel-array load-backfill (closes the MEDIUM false-negative at doc_alignment.rs:1119-1122) — all sitting in the OP-8 Slot 2 drain queue.

---

_Verdict: GREEN — all 5 P96 GREEN-contract items PASS with reproduced command evidence; no P96 regressions; accepted deviations honored. STATE.md advance to P97 is the coordinator's action._

_Verified: graded HEAD `0bdd752`; verifier ran gates + runner + `reposix-quality` + `reposix-cache` suites; catalogs byte-immutable across the entire pass._
