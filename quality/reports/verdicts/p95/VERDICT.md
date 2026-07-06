# P95 Phase-Close Verdict — v0.13.0 milestone-close (docs-alignment refresh + marker-footgun doc pass)

**Overall: GREEN**

- **Graded HEAD:** `da63a85` (`da63a8548bdc57057888cb323df64d474887185c`) — local HEAD == `origin/main` (push-before-verifier cadence satisfied), tree clean.
- **Verifier:** unbiased phase-close subagent (did not execute the work). Graded goal-backward against reality, not the executor's word.
- **Scope note:** P95 as scoped by the live plan (STATE.md + `COORD-HANDOFF-P95-P97.md`) is a docs-alignment REFRESH + doc pass + intake drain — NOT the orphaned 15-requirement "RBF-cluster" prose still sitting in `ROADMAP.md` § Phase 95 (that prose is confirmed STALE per handover Hazard #4; its reconciliation is a P97 milestone-close item, not a P95 gap).

---

## Contract Item 1 — Docs-alignment walk clean — **PASS**

**Command evidence:**

- `./target/release/reposix-quality walk` → **exit 0** (P0 gate PASSES). STDOUT empty; STDERR only informational `coverage:` notes (out-of-eligible-file citations), no drift lines.
- Catalog md5 **unchanged** before/after my walk (`b145437f784f4f6b8b82eb9135efd30d`); `git status` clean — the standalone `walk` verb did **not** self-mutate (Hazard #1 is the pre-push/runner path, not `walk`).
- `summary.claims_bound` = **261** (as contracted); `claims_total` 393, `alignment_ratio` 0.7768 (floor 0.5), `claims_retired` 57.
- On-disk verdict distribution: 261 BOUND / 57 RETIRE_CONFIRMED / 75 STALE_TEST_DRIFT. The 75 STALE rows are **non-blocking** — `RowState::blocks_pre_push` (`catalog.rs:431`) excludes `StaleTestDrift`, so exit 0 is legitimate, not a masked P0 failure. None of the 75 are P95-targeted rows.

**All 16 named target rows read BOUND** (JSON parse of `doc-alignment.json`):
- 5 item-1 rows: `docs/decisions/009-stability-commitment/exit-codes-locked`, `docs/index/ci-badge`, `docs/index/quality-score-badge`, `docs/index/quality-weekly-badge`, `git-remote/bulk-delete-override-tag` — all BOUND / BIND_GREEN.
- 8 (actually 10 on disk — intake self-notes the "8" title under-counts) `docs/connectors/guide/backendconnector-*-method` + `trait-method-count-eight` backend.rs rows — all BOUND (P94 `46bd1fa` re-bind holds under walk; P95 verify-and-close, no redundant re-bind).
- 3 workflow rows: `.../polish2-03-bench-cron`, `.../polish2-02-aarch64`, `.../polish-06-binaries` — all BOUND (STALE_TEST_DRIFT → BOUND in `bd18827`).

**Anti-gate-silencing spot-checks (independent hash recompute vs live source — 3 rows, contract required ≥2):**

| Row | Cite | Stored hash | Recomputed (live) | Match |
|---|---|---|---|---|
| `bulk-delete-override-tag` | `bulk_delete_cap.rs::six_deletes_with_allow_tag_actually_deletes` (Rust fn, `hash_test_fn`) | `61284b2c…45e86f` | `61284b2c…45e86f` | ✓ |
| `polish2-02-aarch64` / `polish-06-binaries` | `.github/workflows/release.yml` (`file_hash`/sha256) | `d3075ef6…f38e5c57` | `d3075ef6…f38e5c57` | ✓ |
| `polish2-03-bench-cron` | `.github/workflows/bench-latency-cron.yml` (sha256) | `324fdeec…04db97b` | `324fdeec…04db97b` | ✓ |

**Underlying claims confirmed TRUE in live source** (re-bind is honest, not a hash-only reshuffle): `aarch64-unknown-linux-musl` matrix entry at `release.yml:101`; `persist-credentials: false` at `bench-latency-cron.yml:34` (row rationale correctly bumped line-ref 32→34); 5-target build matrix (`grep -c target:` = 5); bulk-delete test asserts 6 deletes with `[allow-bulk-delete]`.

---

## Contract Item 2 — Marker-footgun documentation pass (`3e9b2b2`) — **PASS**

- **Gate header** `quality/gates/agent-ux/test-name-vs-asserts.sh` — carries the "⚠ MARKER PLACEMENT WINDOW (6-line LOOKBACK)" block: window mechanics + correctly-placed vs mis-placed example + the two silent-footgun consequences (marker ignored; worse, `#[test]` out-of-window ⇒ fn skipped as non-test ⇒ dishonest name passes).
- **`quality/CLAUDE.md`** § Honesty rules — mirror note present ("Marker placement window (test-name-honesty)").
- **Gate PASSES:** `bash quality/gates/agent-ux/test-name-vs-asserts.sh` → **exit 0** ("RAISE set matches the committed baseline"); `bash -n` clean; `CONTEXT_LINES=6` real at line 88, used in the `sed` lookback at line 99 (doc matches code).
- **GOOD-TO-HAVES.md footgun entry → RESOLVED** (commit `3e9b2b2`, line 377) with three honest corrections to the entry-as-filed (lookback not "after"; marker is `// test-name-honesty: ok — <reason>` not `// HONEST:`; the claimed `PROTOCOL.md` marker-format section does not exist).
- **New sibling GOOD-TO-HAVE exists** (line 381, "Preamble-anchored marker scan", severity LOW, STATUS OPEN) with acceptance sketch + why-deferred.
- **"0 of 85 fns beyond window" ownership claim corroborated:** independent scan of `crates/**/*.rs` found 74 `test-name-honesty: ok` markers, **0 orphaned** (every marker has a `fn` signature within its 6-line window). Doc-only ownership call stands.

---

## Contract Item 3 — Intake drains (`da63a85`) — **PASS**

- **3 intake entries marked RESOLVED with commit SHAs** in `SURPRISES-INTAKE.md` (on-disk `grep -c` = 3):
  1. `2026-07-04 05:40` (3 bench-cron/release.yml rows, was DEFERRED-P95) → RESOLVED (P95, `bd18827`), claims verified live before re-bind.
  2. `2026-07-06` (5 self-mutation-loop rows, item-1) → RESOLVED (P95, `bd18827`); documents the sequencing answer (walk self-mutation does not block a clean re-bind).
  3. `2026-07-06 docs-alignment/walk P0` (8→10 backend.rs rows, was OPEN) → RESOLVED (fixed P94 `46bd1fa`, verify-and-close P95 `bd18827`).
- **Intake file NOT split** (bloat-split is P96's job): only `SURPRISES-INTAKE.md` + `GOOD-TO-HAVES.md` exist, no sibling split files. `da63a85` was additive (26 ins / 3 del on the intake; 14 ins on GOOD-TO-HAVES).

---

## NOTICED (ownership deliverable)

1. **`summary.last_walked` staleness (INFO).** `doc-alignment.json` `summary.last_walked` = `2026-07-05T21:29:18Z`, but the P95-rebound rows carry `last_run`/`last_extracted` = `2026-07-06T01:24:xxZ` — the summary block's walk timestamp predates the row re-binds by ~4h. A `bind` updates rows + counts but not `summary.last_walked`; a future reader could misread the catalog freshness. Low severity; candidate note for the P96 catalog-hygiene work.
2. **75 STALE_TEST_DRIFT rows remain on disk (INFO, out of P95 scope).** Non-blocking (excluded from `blocks_pre_push`), above the 0.5 alignment floor at 0.777, and outside P95's 16 targeted rows. This is the broader docs-alignment drift backlog; not a P95 gap but worth a future refresh sweep.
3. **Row-count imprecision in contract/intake ("8" backend.rs rows).** The trait actually has 10 bound per-method rows (`backendconnector-*-method` ×9 + `trait-method-count-eight`); the intake self-notes this. All BOUND regardless — bookkeeping imprecision, not a defect.
4. **`ROADMAP.md` § Phase 95 prose is STALE** (orphaned 15-req RBF-cluster framing). Confirmed by handover Hazard #4; deferred to P97 milestone-close reconciliation. Flagged below as a RAISE for P97, not a P95 gap.
5. **Positive: the standalone `walk` verb is read-only** (no catalog write on my run). Hazard #1's self-mutation lives in the pre-push/runner path (`run.py`/`verdict.py`), consistent with the P96 fix target — good corroboration for the P96 scoping.

## RAISE LIST (forward, not P95 gaps)

- **P96:** confirm the self-mutation fix targets the runner/pre-push path (walk verb is already clean); consider having `bind` refresh `summary.last_walked` or emit an explicit "counts-only, not full-walk" marker (NOTICED #1).
- **P97:** reconcile the stale `ROADMAP.md` P94–P97 prose during milestone-close (NOTICED #4) — this is exactly the stale-doc drift the docs-alignment dimension exists to catch.

---

## Loop-back items (RED)

None. All 3 contract items PASS. No RED.

**STATE.md advance to P96 is the coordinator's action** (not this verifier's).

_Verified: 2026-07-06 · Verifier: Claude Opus 4.8 (1M context), unbiased phase-close subagent · Graded HEAD `da63a85`._
