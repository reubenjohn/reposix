---
phase: 126-docs-alignment-tooling-polish
subphase: 126-01
review_type: DP-2 mechanism-vs-symptom
reviewed: 2026-07-19T00:00:00Z
depth: deep
verdict: PASS
commits_reviewed:
  - 44783ebe  # RED repro
  - 65e8c497  # GREEN fix
  - d0753ef6  # fix-twice doc
files_reviewed:
  - quality/catalogs/agent-ux.json
  - quality/runners/run.py
  - quality/runners/_audit_field.py
  - quality/runners/test_run.py
  - quality/runners/test_audit_field.py
  - quality/CLAUDE.md
findings:
  blocker: 0
  warning: 1
  total: 1
status: issues_found
---

# P126 W1 — DP-2 Mechanism-vs-Symptom Review of the real-git-push-e2e Landmine Fix

**Reviewed:** 2026-07-19
**Depth:** deep (cross-file: run.py ↔ _audit_field.py ↔ catalogs ↔ tests, RED/GREEN re-run against archived pre-fix tree)
**Verdict:** **PASS** — the fix addresses the ROOT mechanism (whole class), not the symptom. One WARNING (a controlled, acceptable test-side gap) is noted, non-blocking.

## Summary

Two distinct landmines were defused: (1) a legacy catalog row (`agent-ux/real-git-push-e2e`, no `minted_at`) whose git≥2.34 `--persist` verifier stamps a fresh `last_verified ≥ P90_MINT_CUTOFF`, raising an uncaught `SystemExit` in `load_catalog` that crashes ALL of `run.py` for every cadence loading `agent-ux.json`; and (2) a validate-only (non-`--persist`) cadence writing/corrupting a catalog. Both are fixed at the mechanism level. I re-ran the RED repro against the archived pre-fix tree (`git archive 44783ebe`) and the full suite at HEAD; verified the invariant's teeth by arming a corpus copy; traced the write boundary to confirm `save_catalog` is the sole catalog writer.

## Mechanism verdict — 5 points

**1. Systemic, not one-row — PASS.** `TestNoArmedMintedAtLandmine` scans the REAL corpus (`Path(__file__).resolve().parents[2]/quality/catalogs/*.json`, all 13 files) and flags any row with `last_verified ≥ P90_MINT_CUTOFF` and no `minted_at`. Its skip set (`-allowlist` stems + `docs-alignment` dimension) EXACTLY mirrors the real crash surface: `run.py::_catalogs()` (line 102-104) drops `-allowlist` files before `load_catalog`, and `load_catalog` (line 122) skips `validate_row` for `docs-alignment` — so the invariant scans precisely the set that can crash, no scope gap. **Teeth verified against reality:** copied the corpus to /tmp, confirmed 0 offenders baseline, then stripped `minted_at` + set fresh `last_verified` on the real row → the actual test class FAILED with 1 failure naming `agent-ux.json::agent-ux/real-git-push-e2e`. A future re-arm is caught in CI, not by a mid-cadence crash.

**2. Write-boundary guard is real — PASS.** `save_catalog(path, data, *, persist: bool)` — `persist` is a REQUIRED keyword (no default → `TypeError` if omitted) and raises `RuntimeError` BEFORE any write when `persist=False`. Grep confirms `save_catalog` is the ONLY catalog writer in production runner modules (line 213 `write_artifact` targets `quality/reports/`, not catalogs; zero raw catalog writes elsewhere). The `main()` call-site (line 616) is already nested under `if args.persist:` (line 589), so the guard is genuine forward-defense at the boundary, not a superficial early-return a caller routes around. `TestValidateOnlyMultiCatalogByteIdentical` drives a real non-`--persist` `main(["--cadence","pre-push"])` over a multi-catalog set including (a) a would-flip row (committed NOT-VERIFIED, live grade PASS — proven non-vacuous by asserting the committed status stayed NOT-VERIFIED) and (b) em-dash/en-dash/curly-quote Unicode, asserting every file byte-identical. GREEN at HEAD.

**3. Repro integrity — PASS.** `test_real_row_survives_lastverified_aging` reads the REAL committed row, ages `last_verified` in memory, writes ONLY a `/tmp` copy, and drives `run.load_catalog()` — it never `--persist`-arms the live catalog (verified by reading the test AND by `git status` staying clean after the full suite). Re-ran against `git archive 44783ebe` (true pre-fix tree): **RED** — `SystemExit: ...lacks a write-once minted_at anchor`. At HEAD: **GREEN**. Genuine flip, not a pre-passing assertion.

**4. Known gap (freshness_synth raw `json.dumps`) — ACCEPTABLE controlled gap (WARNING, non-blocking).** `test_freshness_synth.py:97,126` write catalogs via raw `json.dumps(indent=2)` (no `ensure_ascii=False`), bypassing `save_catalog`. This is bounded by the `backup_catalogs` fixture (f1959373): it copies every catalog before mutation and restores all on teardown — which pytest runs even on assertion failure. Blast radius is a SIGKILL-mid-test window only, and it is TEST infra, not the production validate path the guard charters. Leaving it outside the boundary guard is acceptable; see WR-01 for the defense-in-depth recommendation.

**5. `--no-verify` not used; fix-twice doc accurate — PASS.** No forensic git marker for `--no-verify` exists, but the committed state is fully hook-consistent: the pre-commit hook runs `run.py --cadence pre-commit` (validate-only — the exact crash surface), and the real `agent-ux.json` loads clean (no `SystemExit`, byte-identical after load); full suite 150 passed; tree byte-clean. The stack is 4 commits ahead of `origin/main` (unpushed), so RED→GREEN→doc push together as GREEN — no bypass needed at the push boundary. The `d0753ef6` doc accurately states: `save_catalog` requires `persist=`/raises on non-persist; the invariant `TestNoArmedMintedAtLandmine` FAILs on a re-armed row; and the `raw json.dumps` prohibition — all verified true.

## Warnings

### WR-01: test-side raw `json.dumps` catalog write bypasses the new boundary guard

**File:** `quality/runners/test_freshness_synth.py:97,126`
**Classification:** WARNING (non-blocking; controlled by the `backup_catalogs` restore fixture).
**Issue:** These two writes use `json.dumps(data, indent=2)` without `ensure_ascii=False`, exactly the anti-pattern `d0753ef6` bans, and do NOT route through `save_catalog`. On a normal pass the fixture restores originals, but on a hard crash between write and restore a catalog could be left `\u`-escaped/reformatted. This is the one code path still able to mutate a committed catalog outside the persist boundary.
**Fix:** Route both through `run.save_catalog(path, data, persist=True)` (preserves `ensure_ascii=False` and centralizes the write discipline), keeping the `backup_catalogs` fixture as the restore net. Not required for this phase to ship — the guard's charter (production validate path) is fully met.

## NOTICED

- **Invariant is well-scoped, not over/under-broad.** The `-allowlist` + `docs-alignment` skips in `TestNoArmedMintedAtLandmine` are not arbitrary — they precisely match `_catalogs()` filtering and `load_catalog`'s `docs-alignment` skip. An allowlist row can never reach `validate_row`, so excluding it from the invariant is correct, not a hole. Good engineering.
- **`minted_at` (2026-07-19) newer than `last_verified` (2026-07-04) is correct for a legacy retrofit.** Because `minted is not None`, `is_new` is computed from `minted_at` and the lv-based raise (line 305) lives only in the `else` branch — so no matter how fresh a future `--persist` stamps `last_verified`, the crash is unreachable forever. The fix is permanent, as claimed, not merely for the current timestamp.
- **`transport_claim: false` short-circuits `is_transport_or_perf_row`** (explicit opt-out branch), so adding `minted_at` does NOT newly demand `coverage_kind: real-backend` — the fix cannot swap one load crash for another. Verified in code.
- **Guard error message is Rust-compiler-grade** — names the offending file, teaches the read-only rule, gives the copy-paste `--persist` recovery, and distinguishes mint vs grade intent. `TestSaveCatalogPersistGuard` asserts the message teaches both "read-only" and "--persist".
- The `agent-ux.json` comment/`owner_hint` refresh (git 2.25.1 → 2.50.1) is an honest, accurate provenance update matching this box's real git.

## Required-change list

None blocking. WR-01 is a recommended defense-in-depth follow-up (file to GOOD-TO-HAVES if not eager-fixed; <1h, no new dependency).

## Evidence log (verify-against-reality)

- RED at pre-fix: `git archive 44783ebe` → `pytest test_real_row_survives_lastverified_aging` → FAILED (`SystemExit ...lacks a write-once minted_at anchor`).
- GREEN at HEAD: 10 targeted tests passed; full `quality/runners/` suite 150 passed; tree byte-clean after.
- Invariant teeth: real-corpus copy → 0 offenders; armed copy → test class produced 1 failure naming the re-armed row.
- Real `agent-ux.json` via `run.load_catalog`: OK, byte-identical after load.
- Write-boundary: grep confirms `save_catalog` is the sole catalog writer in production modules.

---

_Reviewed: 2026-07-19_
_Reviewer: Claude (gsd-code-reviewer) — DP-2 mechanism-vs-symptom_
_Depth: deep_
