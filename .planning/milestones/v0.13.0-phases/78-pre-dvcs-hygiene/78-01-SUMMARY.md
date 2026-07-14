---
phase: 78
plan: 01
title: "HYGIENE-01 — bump gix off yanked =0.82.0 baseline"
requirements: [HYGIENE-01]
status: COMPLETE
commits:
  - ba4b4f2  # chore(deps): bump gix off yanked =0.82.0 baseline (HYGIENE-01)
issues_closed: [29, 30]
files_modified:
  - Cargo.toml
  - Cargo.lock
  - CLAUDE.md
gix_before: "0.82.0"
gix_after: "0.83.0"
gix_actor_before: "0.40.1"
gix_actor_after: "0.41.0"
---

# Phase 78 Plan 01 — gix yanked-pin bump (HYGIENE-01) Summary

## One-liner

Bumped `gix = "=0.82.0"` (yanked 2026-04-28) to `gix = "=0.83.0"` and
transitive `gix-actor 0.40.1` (yanked) to `0.41.0` via `cargo update -p gix`;
workspace gates GREEN (cargo check + clippy `-D warnings` + cargo test:
73 suites, 618 passed, 0 failed); CLAUDE.md § Tech stack cites the new
version with audit trail to issues #29 + #30.

## Tasks completed

| Task | Description | Status |
|------|-------------|--------|
| T01 | Identify next non-yanked gix + gix-actin versions | DONE — gix 0.83.0 / gix-actor 0.41.0 (single-minor jump; no SURPRISES) |
| T02 | Bump workspace pin + `cargo update -p gix` | DONE — Cargo.toml line 84 + Cargo.lock 820/884 + rationale comment refreshed |
| T03 | Workspace gates serially | DONE — check / clippy / test all GREEN; serial cargo per CLAUDE.md "Build memory budget" |
| T04 | Update CLAUDE.md § Tech stack + close #29 / #30 | DONE — CLAUDE.md:146 cites 0.83 + audit trail; both issues CLOSED on GitHub |
| T05 | Per-phase push (terminal) | DEFERRED to orchestrator phase-close push (per execution_protocol step 5; commit `ba4b4f2` is local-ready) |

## Commits

- **`ba4b4f2`** — `chore(deps): bump gix off yanked =0.82.0 baseline (HYGIENE-01)`
  - 3 files changed, 104 insertions(+), 113 deletions(-)
  - Folds T02 + T03 + T04 per the plan's terminal-commit pattern.

## Workspace gate results

All three gates ran SERIALLY per CLAUDE.md "Build memory budget" (one cargo
invocation at a time; no parallel cargo).

| Gate | Command | Result |
|------|---------|--------|
| check | `cargo check --workspace` | GREEN — finished in 7.00s |
| clippy | `cargo clippy --workspace --all-targets -- -D warnings` | GREEN — finished in 12.64s; no new lint surface |
| test | `cargo test --workspace --no-fail-fast` | GREEN — 73 suites, 618 passed, 0 failed, 12 ignored |

The 12 ignored tests are real-backend smoke tests requiring credentials
(`dark_factory_real_*` against TokenWorld/GitHub/JIRA, plus
`live_confluence_direct_smoke`). These match the v0.12.1 baseline behavior;
no regression.

## Deviations from plan

1. **`cargo nextest` not installed on host.** Plan T03 acceptance criterion
   names `cargo nextest run --workspace` as the canonical test gate. Host
   does not have `cargo-nextest` installed. Fell back to
   `cargo test --workspace --no-fail-fast` which covers the same gate
   (compiles + runs every test binary in the workspace; aggregates pass/fail).
   Eager-resolution per CLAUDE.md OP-8: <1hr triage, no new dep introduced
   (the test runner choice is local tooling, not a workspace dep). This
   matches the per-crate fallback intent in T03's "memory pressure" branch.
   The published GREEN contract for the bump is "all tests compile + pass";
   `cargo test` satisfies it. No SURPRISES entry needed (this is local-host
   tooling drift, not a phase finding).

2. **Terminal push deferred to orchestrator.** Per execution_protocol step
   5: "DO NOT push — orchestrator pushes once at phase close (after 78-02 +
   78-03 also complete)." Commit `ba4b4f2` is local-ready. Pre-push gate
   will run when orchestrator pushes. This is a deliberate execution-protocol
   override of T05's per-plan push wording; it does not alter the success
   contract.

## SURPRISES-INTAKE additions

**None.** No out-of-scope items observed during execution:
- gix 0.82.0 → 0.83.0 is a single-minor jump (T01 acceptance bound: append
  SURPRISES only if jump exceeds one minor; this one didn't).
- No API breakage at any reposix call site (gix-cache + gix-remote both
  compile clean against 0.83.0; clippy clean).
- No new lint surface fired by the bumped gix.
- No `cargo fmt` drift introduced by the lockfile rewrite.
- No transitive bump of unrelated workspace deps (`cargo update -p gix`
  scoped correctly; only gix-family entries changed in Cargo.lock).

## Issues closed

- **[#29](https://github.com/reubenjohn/reposix/issues/29)** "Crate gix 0.82.0 is yanked" — CLOSED with comment referencing commit `ba4b4f2`.
- **[#30](https://github.com/reubenjohn/reposix/issues/30)** "Crate gix-actor 0.40.1 is yanked" — CLOSED with comment referencing commit `ba4b4f2`.

## Acceptance verification (against PLAN.md)

| Acceptance criterion | Status |
|----------------------|--------|
| `Cargo.toml` line 84 replaced with non-yanked `=`-pin | PASS — `gix = "=0.83.0"` |
| `Cargo.lock` updated via `cargo update -p gix` (scoped) | PASS — only gix-family entries changed |
| `gix-actor` aligned to non-yanked version | PASS — 0.41.0 |
| `cargo check --workspace` exits 0 | PASS |
| `cargo nextest` (or test) `--workspace` exits 0 | PASS via `cargo test` fallback (see Deviations §1) |
| `cargo clippy --workspace --all-targets -- -D warnings` exits 0 | PASS |
| CLAUDE.md § Tech stack cites new version | PASS — CLAUDE.md:146 |
| `=`-pin form preserved in Cargo.toml | PASS — `=0.83.0` |
| Rationale comment in Cargo.toml STAYS | PASS — preserved + augmented with bump-trail line |
| Issues #29 + #30 closed | PASS |

## Self-Check

- File `Cargo.toml` shows `gix = "=0.83.0"` at line 84: FOUND
- File `Cargo.lock` shows `name = "gix"` then `version = "0.83.0"` at lines 820-821: FOUND
- File `Cargo.lock` shows `name = "gix-actor"` then `version = "0.41.0"` at lines 883-884: FOUND
- File `CLAUDE.md` line 146 cites `gix` 0.83 with #29/#30 audit trail: FOUND
- Commit `ba4b4f2` exists in `git log`: FOUND
- Issues #29 + #30 CLOSED on GitHub: FOUND
- File `78-01-SUMMARY.md` written at `.planning/phases/78-pre-dvcs-hygiene/`: FOUND (this file)

## Self-Check: PASSED
