---
quick_id: 260713-rug
title: "Green RED-main docs-repro/example-04 via TIMEOUT-BUDGET fix (ruling b773c04)"
status: complete
completed: 2026-07-13
---

# Quick Task 260713-rug — SUMMARY (TIMEOUT-BUDGET fix)

## Problem (REPRODUCED + DIAGNOSED — VERDICT: TIMEOUT-BUDGET)

`quality-post-release` run 29301412750 (sha `05aa23c`) went RED: gate
`docs-repro/example-04-conflict-resolve` FAILED at exactly **300.00s** — the runner's
`subprocess.run(timeout=300)` SIGKILLed it. Not a hang, not a guardrail conflict. The
example workflow runs in ~0.5s and passes all 3 asserts. The 300s cap was consumed by
per-container-row SETUP `apt-get install ... build-essential pkg-config libssl-dev ...` —
compile-time deps that are NEVER exercised: the examples run the host-mounted pre-built
`target/debug/reposix` (on PATH via `--network host` + `-v target:...:rw`); there is no
in-container cargo build anywhere. On a slow CI run, apt drew example-04 past the cap.

## What changed (two clean/honest edits — no assert / waiver / example-proof touched)

**1. `quality/gates/docs-repro/container-rehearse.sh` (`SETUP=`).** Dropped
`build-essential pkg-config libssl-dev`; kept `curl ca-certificates python3 git sqlite3`.
Added a fix-it-twice comment above the line: compiler toolchain intentionally excluded
(examples run a pre-built host-mounted binary); do NOT re-add build-essential.

**2. `quality/catalogs/docs-reproducible.json`.** Bumped `timeout_s` 300→600 symmetrically
on EVERY `kind:container` row — example-01/02/04/05 (catalog lines 27/64/137/174). Left
all non-container rows untouched: tutorial-replay (mechanical) stays 300; the two
benchmark rows stay 60; example-03/snippet-coverage stay 30. JSON revalidated with
`python3 -m json.tool` → VALID.

No `run.py`, `_audit_field.py`, assert, or waiver change. F-K4b logic untouched.

## Prove-before-fix (ACTUAL observed, docker live, host warm binary)

| Row (docs-repro/…)        | harness rc | wall-clock | artifact exit_code / asserts_failed |
|---------------------------|-----------|-----------|--------------------------------------|
| example-01-shell-loop     | 0         | 16s       | 0 / []                               |
| example-02-python-agent   | 0         | 15s*      | 0 / []                               |
| example-04-conflict-resolve | 0       | 16s       | 0 / []  (was 300.00s SIGKILL in CI)  |
| example-05-blob-limit-recovery | 0    | 19s       | 0 / []                               |

`*` example-02 flaked ONCE on the first back-to-back sequential pass (harness rc=0 but a
fresh artifact showed exit_code 1 / `FAIL: sim not reachable at 127.0.0.1:7878`) — a
pre-existing sim-readiness race between rapid consecutive harness invocations (the sim
runs on the HOST, reached via `--network host`; the trim removes only in-container apt
packages the examples never use, so it cannot cause this). Re-run in isolation (port 7878
confirmed free, no lingering sim proc) → rc=0, exit_code 0, asserts_failed []. All four
rows green with asserts holding.

## Honesty guard: CLEAN

No assert weakened, no waiver added, no example-proof shortened. The change only removes
unused packages + grants already-passing rows more wall-clock budget.

## Noticing (OD-3)

- **Harness sim-readiness race (pre-existing, MEDIUM).** Back-to-back
  `container-rehearse.sh` invocations can race on host port 7878 (prior row's ephemeral
  sim still tearing down when the next binds), yielding a transient container-side
  `sim not reachable` and — separately — a harness-rc(0)/artifact-exit_code(1) mismatch
  worth a look (the `exit "$EXIT_CODE"` vs EXIT-trap interaction). Not caused by this fix;
  candidate GOOD-TO-HAVE (bind-retry / port-free wait before docker run, or reconcile the
  rc/artifact codes).
- **`.sim-*.log` under `quality/reports/verifications/docs-repro/` are NOT gitignored**
  (the `*.json` siblings are), so they surface as untracked `??`. Left untouched (not this
  charter); candidate one-line gitignore addition.
- **Header line-budget still stale** (`container-rehearse.sh` top comment says `<=150
  lines`; file is ~200) — already filed by quick 260713-q0e; unchanged here.

## Commit

- (this commit) — `fix(quality): green example-04 via TIMEOUT-BUDGET fix (ruling b773c04)`,
  stacked on HEAD `5a6a042` (unpushed handover; deliberately not reset/rebased). No push —
  orchestrator owns push + `quality-post-release` re-trigger + green confirmation.

Regenerated `quality/reports/verifications/docs-repro/*.json` artifacts are gitignored and
left uncommitted.

## Self-Check: PASSED

- container-rehearse.sh SETUP trim: FOUND (`git show` of this commit).
- catalog 4 container rows at 600, tutorial-replay at 300, JSON valid: FOUND.
- all 4 container rows rc=0 / asserts_failed []: FOUND (observed this session).
- honesty guard clean (no assert/waiver/proof change): FOUND.
