---
phase: 91-attach-sync-real-backend-wiring
plan: 05
subsystem: quality-gates / agent-ux
tags: [litmus, real-backend, milestone-close, D91-06, D90-06, safety-guard]
requires: [91-01, 91-02, 91-03, 91-04]
provides: [milestone-close-vision-litmus real body, mass-delete safety guard, mirror substrate]
affects: [quality/gates/agent-ux/milestone-close-vision-litmus.sh, quality/gates/agent-ux/lib/litmus-flow.sh]
tech-stack:
  added: []
  patterns: [transcript-lib wrapping, sanctioned-target in-body assertion, pre-push mass-delete guard]
key-files:
  created: [quality/gates/agent-ux/lib/litmus-flow.sh]
  modified: [quality/gates/agent-ux/milestone-close-vision-litmus.sh]
decisions: [D91-06, D90-06, D91-07, OD-2]
metrics:
  duration: ~2h30m
  completed: 2026-07-04
---

# Phase 91 Plan 05: Real Milestone-Close Vision-Litmus Body Summary

Replaced the unconditional exit-75 stub `milestone-close-vision-litmus.sh` with the real 8-step D91-06 body — sanctioned-target-asserting, dual-audit-checking, transcript-emitting — plus a mass-delete SAFETY GUARD forced by a CRITICAL substrate bug discovered while proving the script against reality.

## What shipped

- **`milestone-close-vision-litmus.sh` (real body, 7227 chars):** env-gate (exit 75), STEP-1 sanctioned-target in-body assertion (D90-06 proof obligation — Confluence `{TokenWorld,REPOSIX}@reuben-john`, hard-FAIL not 75 on anything else), STEP-2 preflight (confluence-specific reachability gate; OD-2 unreachable-with-creds = hard RED), one-shot `cargo build -p reposix-cli -p reposix-remote`, and dispatch of the round-trip under `lib/transcript.sh`, then artifact asserts-patch + honest exit.
- **`lib/litmus-flow.sh` (new, 7211 chars):** the STEP 3-6 vanilla-clone + `reposix attach confluence::TokenWorld` + edit + `git push` round-trip, the 5 T2 pass boxes, dual-table audit (`audit_events_cache` op + `audit_events` method/path), refs/mirrors advancement check, and the artifact-patch helper. Factored out under the 10k `.sh` budget (mirrors `dark-factory/reconciliation-fixture.sh`).
- Row status UNTOUCHED (runner grades it); no `waiver` added (OD-2 / anti-C7 preserved).

## Mirror substrate (D91-07) — verified + populated

`reubenjohn/reposix-tokenworld-mirror` was content-empty (only `.github/` + README, last push 2026-05-01). Reset to its clean baseline (commit `09dda47`) and re-populated via `reposix refresh --backend confluence --project TokenWorld` (plain-git force-push, NOT the reposix bus helper). Verified: `git ls-remote` → `2a66596 refs/heads/main`; `gh api .../contents/pages` → `2818063.md 7766017.md 7798785.md` (the 3 real TokenWorld records). NOTE: these are `pages/`-bucket records (what `reposix refresh` currently emits) — which the litmus GUARD B then correctly flags (see incident).

## OD-2 exit-code decisions encoded in the script

- **env/creds unset → 75** (honest env-gate; runner maps 75→NOT-VERIFIED; never skip-as-pass).
- **non-sanctioned space/tenant → 1 (hard FAIL, never 75)** — the D90-06 proof obligation.
- **creds set but confluence unreachable → 1 (hard RED)** — "substrate exists, cannot execute" (OD-2), NOT a legitimate 75. Preflight is run for the record but the GATE decision is a confluence-specific probe (preflight is all-or-nothing across 3 backends; an unrelated JIRA/GitHub gap must not sink a confluence litmus).
- **documented happy path disagrees with binary → 1 (hard FAIL)** — GUARD trips, server-confirm mismatch, dual-audit gap.

## Proof against reality (OP-1)

| Run | Exit | Outcome |
|-----|------|---------|
| env-unset | **75** | honest NOT-VERIFIED (creds absent) |
| real TokenWorld | **1** | honest FAIL at GUARD B (pages/ bucket), **non-destructive** (no push), transcript emitted naming `reposix attach confluence::TokenWorld` |

Real-run artifact: `quality/reports/verifications/agent-ux/milestone-close-vision-litmus-real-backend.json` (status FAIL, 6 asserts_passed / 1 asserts_failed). Transcript: `quality/reports/transcripts/milestone-close-vision-litmus-real-backend-<RFC3339>.txt` (gitignored). Reconciliation on the real run: `matched=3 no_id=1 backend_deleted=0 mirror_lag=0` (GUARD A passed; GUARD B caught the `pages/` bucket).

## Deviations from Plan

### [Rule 2 - Critical safety] Added a mass-delete pre-push GUARD

Proving the script against reality (OP-1) uncovered a CRITICAL data-loss bug: the confluence export/push path issues `delete_or_close` against EVERY backend record when the working-tree bucket differs from the cache's canonical `issues/<id>.md` (e.g. the `pages/<id>.md` that `reposix refresh` writes). During development this trashed the protected durable fixtures 7766017/7798785 + Home 2818063. **All three were restored** (v1 content restore trashed→current + re-linked child 7798785→parent 7766017; verified current + hierarchy + `reposix-durable-fixture` labels). The plan's naive "edit + push" flow would be destructive, so GUARD A (reconciliation `matched>=1 && backend_deleted==0`) + GUARD B (refuse `pages/*.md` bucket) were added to REFUSE a delete-shaped diff and hard-FAIL instead — a Rule-2 correctness requirement, not scope creep. The underlying binary bug is filed BLOCKER to SURPRISES-INTAKE (code-lane fix).

### Preflight gate nuance
`preflight-real-backends.sh` is all-or-nothing; the litmus gates on a confluence-specific probe instead. Documented inline + in what-to-notice.

## Known Stubs
None — the stub is fully replaced.

## Threat Flags
| Flag | File | Description |
|------|------|-------------|
| threat_flag: credential-leak | crates/reposix-cli/src/attach.rs | attach folds credential-bearing origin URL into remote.reposix.url + helper echoes it to stderr (filed MEDIUM to intake) |
| threat_flag: data-loss | crates/reposix-cli/src/refresh.rs + reposix-remote/src/diff.rs | pages/-vs-issues/ bucket mismatch mass-deletes backend records on push (filed BLOCKER) |

## NOTICED (ownership charter clause 2)

1. **CRITICAL: confluence push mass-deletes** on bucket mismatch (BLOCKER intake) — the single highest-severity P91 finding; should gate the v0.13.0 tag.
2. **refresh writes `pages/` for confluence** vs canonical `issues/` (builder.rs/diff.rs/D91-01) — root cause of #1; the round-trip cannot work until aligned.
3. **Token leaks into `remote.reposix.url` + helper stderr** (MEDIUM intake).
4. **`git-remote-reposix: unknown command: feature`** stray line after egress-denied bus reject (LOW intake).
5. **`TokenWorld` == `REPOSIX` == space 360450** (same space, two key aliases); the durable fixtures live IN the litmus's mutation target — testing-targets.md implies two distinct spaces (LOW intake).
6. The `git push reposix main` mirror leg requires `https://github.com` in `REPOSIX_ALLOWED_ORIGINS` (the allowlist has `api.github.com`, not `github.com`); the helper teaches the exact fix — good UX, noted for the owner's milestone-close env.

## Intake filed
4 entries appended to `.planning/milestones/v0.13.0-phases/SURPRISES-INTAKE.md` (1 BLOCKER, 1 MEDIUM, 1 LOW-pair) + pre-existing file-size violations logged to phase `deferred-items.md`.

## Coordinator hand-off (D91-07 REOPEN gate)
The mechanical litmus is now real and anchors the REOPEN gate. On the CURRENT substrate it hard-FAILs (exit 1) at GUARD B — the honest signal that the confluence attach→edit→push round-trip is NOT milestone-close-ready (BLOCKER intake). Once the code lane fixes the bucket/mass-delete bug and the mirror is repopulated with `issues/`-shaped records, the coordinator re-runs (a) this script → exit 0 and (b) a fresh dark-factory subprocess agent for the T2 friction grade. Executor did NOT flip the row status or run the fresh-agent grade (coordinator's phase gate).

**Wave-5.5 UPDATE (code fix landed — supersedes the repopulation instruction above):** the bucket/mass-delete BLOCKER is RESOLVED at commit `d6e1411` (+ credential-leak MEDIUM at `e2f8acd`). The design KEEPS `pages/` as the canonical confluence bucket (bucket-aware `reposix_core::path` + id-keyed `diff::plan` — see the RESOLVED intake entry), so the mirror must **stay `pages/`-shaped — do NOT repopulate it to `issues/`**. GUARD B in `lib/litmus-flow.sh` was updated accordingly (asserts ≥1 `pages/*.md` record and picks the edit target from `pages/`); GUARD A and the SG-02 delete cap are unchanged. The coordinator re-run needs no substrate changes beyond creds/env.

## Self-Check: PASSED
- `quality/gates/agent-ux/milestone-close-vision-litmus.sh` — FOUND (7227 chars, exec, `bash -n` clean)
- `quality/gates/agent-ux/lib/litmus-flow.sh` — FOUND (7211 chars, `bash -n` clean)
- commit `5786784` — FOUND
- env-unset exit 75 — VERIFIED; real run exit 1 non-destructive — VERIFIED; durable fixtures current + hierarchy intact — VERIFIED
