---
quick_id: 260713-q0e
title: "Fix RED main — honest bounded F-K4b container-congruence fix (example-05 asserts reworded to the truth)"
status: complete
completed: 2026-07-14
---

# Quick Task 260713-q0e — SUMMARY

> **Reworked under Manager Ruling #5 (Option A), 2026-07-13.** The first attempt
> (`0f2b7c5`) shipped the `container-rehearse.sh` emission mechanism but failed adversarial
> verification as a SYMPTOM-FIX: example-05's catalog asserts overclaimed. This SUMMARY
> describes the FINAL honest state. The held `0f2b7c5`/`058c465` were un-stacked via
> `git reset --soft d68fa8a` and re-committed as one honest bounded arc — `0f2b7c5` no
> longer stands alone as the pushed fix.

## Problem

`quality-post-release` (CI run 29298424648, sha `d68fa8a`, v0.14.0 tag) went RED on 4 P1
docs-repro example gates (01/02/04/05). Root cause = harness gap, NOT a product regression:
every example container **exits 0** (the examples genuinely work), but the generic
`container-rehearse.sh` emitted only ONE generic `asserts_passed` line, which the runner's
F-K4b per-expected-assert congruence (`quality/runners/_audit_field.py::asserts_congruent`)
rejects unless every `expected.asserts` entry token-maps to some `asserts_passed` entry.
Latent since P106 hand-minted `status:PASS` + retired the waivers (804eedc/c4f1261,
2026-07-12); the first post-P106 real post-release run surfaced it.

## What changed (honest bounded fix — commit `03e7a6f`)

**1. KEEP the emission mechanism (verified honest for examples 01/02/04).**
`container-rehearse.sh`, on container `exit_code == 0`, emits the row's `expected.asserts`
verbatim as `asserts_passed` (in addition to the generic "ran and exited 0" line). A fresh
verifier confirmed examples 01/02/04 `run.sh` are fail-loud (`set -euo pipefail` + explicit
conflict-forcing / `check=True` pushes), so exit 0 genuinely ⟺ their asserts hold.

**2. REWORD example-05's asserts #2/#3 to the TRUTH** (`quality/catalogs/docs-reproducible.json`).
The old #2 claimed the agent "observes the blob-limit error from helper stderr and recovers"
— a LIE: the runtime blob-limit error provably never fires (fast-import bypasses the per-RPC
`command=fetch` check; `run.sh:28` greps a SOURCE CONSTANT; `expected-output.md` documents
zero `blob_limit_exceeded` rows). Old #3 ("siblings stay sparse") was not load-bearing —
`run.sh:51`'s bare `ls issues/*.md` only needs ≥1 file. Reworded to describe exactly what
run.sh exit-0 establishes (see the assert→line map in the report). `claim_vs_assertion_audit`
+ `owner_hint` reworded to match; the owner_hint now warns future readers NOT to re-introduce
the runtime-error claim.

**3. Honesty-caveat comment** added to `container-rehearse.sh`: emitting `expected.asserts`
verbatim makes F-K4b a TAUTOLOGY for container rows (a no-op `exit 0` script would pass);
honesty rests on each script being fail-loud + each catalog assert describing only what
exit 0 establishes. Redesign filed → v0.15.0.

No changes to `run.py` or `_audit_field.py` (F-K4b logic untouched; NO waivers).

## Before / after runner summary

**Before (repro, matches CI run 29298424648 @ `d68fa8a`):** `2 PASS, 4 FAIL -> exit=1`
(01/02/04/05 FAIL on `F-K4b: expected assert(s) not covered by any asserts_passed entry`).

**After (this fix, local run, docker live, validate-only):**
```
quality/runners/run.py --cadence post-release  [validate-only (catalog writes OFF)]
    [PASS] docs-repro/tutorial-replay            (P0, 2.73s)
    [PASS] docs-repro/example-01-shell-loop      (P1, 134.96s)
    [PASS] docs-repro/example-02-python-agent    (P1, 47.09s)
    [PASS] docs-repro/example-04-conflict-resolve(P1, 38.90s)
    [PASS] docs-repro/example-05-blob-limit-recovery (P1, 49.22s)
    [PASS] release/cargo-binstall-resolves       (P1, 3.49s)
summary: 6 PASS, 0 FAIL, 0 PARTIAL, 0 WAIVED, 0 NOT-VERIFIED -> exit=0
```
Spot-checked `example-05-blob-limit-recovery.json`: `asserts_passed` lists the generic line
+ all 3 reworded asserts verbatim; `asserts_failed` is `[]` — no `F-K4b` line. The captured
stdout confirms the reworded asserts are truthful (teaching string echoed with `{N}`/`{M}`
placeholders = source constant, not a runtime error; pre-emptive sparse-checkout narrows 3
then 4 files).

## Commit arc (no push — orchestrator-gated)

- `03e7a6f` — `fix(quality): honest bounded F-K4b fix — reword example-05 asserts to the truth`
- `3775075` — `docs(intake): file v0.15.0 F-K4b container-tautology + example-05 deeper-fix`
- (this commit) — ledger (CONSULT-DECISIONS Ruling #5) + quick docs + STATE row

Regenerated `quality/reports/verifications/docs-repro/*.json` artifacts from the validate
run remain untracked/uncommitted — left for the orchestrator.

## Follow-up filed (v0.15.0 SURPRISES-INTAKE, MEDIUM)

ONE row covering (a) F-K4b container-tautology redesign (per-step-earned emission like
`tutorial-replay.sh`, OR a fail-loud meta-check) + (b) example-05 real-runtime-error deeper
fix (drive the genuine observe-error → sparse-checkout → retry cycle; today covered only by
`quality/gates/agent-ux/dark-factory.sh`).

## Noticing (OD-3)

- **Header-comment line-budget already stale.** `container-rehearse.sh`'s top comment says
  `<=150 lines`; the file was already >150 lines before this arc and is now ~196. A smaller
  instance of the exact claim-vs-reality gap this task fixes, just in a comment. Not fixed
  here (out of the honest-fix charter) — a candidate GOOD-TO-HAVE (trim the script or update
  the stated ceiling).
- **F-K4b container-class tautology (root finding).** Now that container-rehearse emits
  expected.asserts verbatim, F-K4b passes by construction for every container row — honesty
  rests entirely on fail-loud scripts + truthful assert text. Filed to v0.15.0 (intake `3775075`).
- **example-05 does not exercise the real runtime blob-limit error at all** (only the
  pre-emptive stand-in). The genuine runtime recovery lives only in the dark-factory arm.
  Filed to v0.15.0 (same intake row, sub-item b).
- No new drift in `_audit_field.py::apply_pass_gates`/`asserts_congruent` — read as
  instructed; behavior matches the described per-pair token congruence, `minted_at`-gated,
  no-op on empty either-list. F-K4b logic left untouched (charter constraint).

## Self-Check

- `03e7a6f` reworks the fix + reworded example-05 asserts: FOUND (`git show 03e7a6f`).
- `0f2b7c5` no longer stands alone: FOUND (`git reset --soft d68fa8a` un-stacked it; the
  landed arc is `03e7a6f`/`3775075`/ledger).
- post-release `6 PASS / 0 FAIL / exit 0` reproduced live: FOUND (captured this session).
- example-05 artifact `asserts_passed` = honest reworded asserts, `asserts_failed` empty: FOUND.
- v0.15.0 intake row filed: FOUND (`3775075`).

## Self-Check: PASSED
