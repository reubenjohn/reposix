---
quick_id: 260713-rug
title: "Green RED-main docs-repro/example-04 via TIMEOUT-BUDGET fix (ruling b773c04)"
status: ready
created: 2026-07-13
---

# Quick Task 260713-rug — Green example-04 (TIMEOUT-BUDGET fix)

Owner ruling b773c04 FIX-FIRST: green the RED-main gate
`docs-repro/example-04-conflict-resolve`, which FAILED at exactly 300.00s in
`quality-post-release` run 29301412750 (sha 05aa23c).

## Problem (REPRODUCED + DIAGNOSED — VERDICT: TIMEOUT-BUDGET)

Not a hang, not a guardrail conflict. The example workflow runs in ~0.5s and passes
all 3 asserts. The 300s cap was consumed by per-container-row SETUP
`apt-get install ... build-essential pkg-config libssl-dev ...` — compile-time deps
that are NEVER exercised (examples run the host-mounted pre-built `target/debug/reposix`;
there is no in-container cargo build anywhere). On the slow CI run, apt drew example-04
past the 300s cap and the runner's `subprocess.run(timeout=300)` SIGKILLed it.

## Fix (two clean/honest edits — no assert, waiver, or example proof touched)

**a. `quality/gates/docs-repro/container-rehearse.sh` (`SETUP=`, ~line 150):**
DROP `build-essential pkg-config libssl-dev`; KEEP `curl ca-certificates python3 git
sqlite3`. One-line fix-it-twice comment above: compiler toolchain intentionally excluded
(examples run a pre-built host-mounted binary); do NOT re-add build-essential.

**b. `quality/catalogs/docs-reproducible.json`:** bump `timeout_s` 300→600 symmetrically
on EVERY `kind:container` row (example-01/02/04/05). Do NOT touch non-container rows
(tutorial-replay stays 300). JSON validated with `python3 -m json.tool`.

## Acceptance (prove-before-fix — do NOT commit until it passes)

Run the FULL container-rehearse suite locally for EVERY `kind:container` docs-repro row:
`bash quality/gates/docs-repro/container-rehearse.sh docs-repro/<row-id>` for
example-01/02/04/05. ALL must be rc=0 with asserts holding, and the apt trim must not
have removed a package any example needs. If ANY row fails rc≠0 → STOP, revert, report.
Docker-based (NOT cargo — no OOM risk); artifacts land gitignored under `quality/reports/`.

## Honesty guard

If applying this somehow required weakening an assert / adding a waiver / shortening what
an example proves → STOP and report. It should not: this only removes unused packages +
grants passing rows more wall-clock.

## Out of scope / notes

- ZERO cargo (binary already built at `target/debug/reposix`).
- Commit atomically (NO --no-verify; let hooks run). Stacks on HEAD 5a6a042 (an unpushed
  handover) — expected; do NOT reset/rebase it away.
- DO NOT PUSH — the L0 orchestrator owns push + `quality-post-release` re-trigger.
