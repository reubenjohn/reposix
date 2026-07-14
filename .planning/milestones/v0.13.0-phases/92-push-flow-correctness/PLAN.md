---
phase: 92-push-flow-correctness
plan: overview
type: execute
wave: 1
depends_on: [89, 90, 91]
requirements: [RBF-B-01, RBF-B-02, RBF-B-03, RBF-B-04, RBF-B-05, RBF-B-06, RBF-B-07]
autonomous: true
---

<objective>
Fix (or lock, where already fixed) the v0.9.0 architectural cornerstone (`git pull --rebase`
recovery after a push-time conflict) and close the OP-3 audit-log silence — both broken on
every push from a partial-clone working tree per the May-02 dark-factory T4 finding and the
p83/p86 phase audits. Full goal text: `.planning/milestones/v0.13.0-phases/ROADMAP.md` §
"Phase 92" (SC1-8 reproduced below).

This overview covers TWO executors:
- **Executor 1 (this session):** catalog-first scaffold for SC1-8 + SC1's sim-arm regression
  test (T4 prove-before-fix, DP-2 discipline — the heavy mechanism fix `cb630e5` already
  landed pre-phase; this executor's job is to PROVE the current state and lock it).
- **Executor 2 (follow-on):** SC2-SC6 implementation (dual-table audit query, behavioral
  no-retry verifier, mid-stream litmus re-run).
</objective>

<residual-items>
Per the P92 recon (D-P92-01, `.planning/CONSULT-DECISIONS.md`), the heavy mechanism fixes
already landed on `main` BEFORE this phase started:
- `cb630e5` — scrubs `GIT_DIR`/`GIT_WORK_TREE`/`GIT_INDEX_FILE`/`GIT_COMMON_DIR`/
  `GIT_OBJECT_DIRECTORY`/`GIT_NAMESPACE` before the bare-cache `git config` shell-out in
  `Cache::open` (`crates/reposix-cache/src/cache.rs`); root-caused the cache-open failure
  that silently disabled push-side OP-3 bookkeeping.
- `a0c84a3` — chains `.with_audit(audit_conn)` on the Confluence + JIRA connectors.

Four residual items carry into this phase:
1. **T4 regression test** (rebase-ancestry, prove-before-fix) — Executor 1.
2. **`bus_write_audit_completeness.rs` dual-table upgrade** — query `audit_events` directly
   (not the wiremock request-log proxy) — Executor 2.
3. **Behavioral no-retry verifier** replacing the source-grep at `bus-write-no-helper-retry`
   — Executor 2.
4. **TokenWorld smoke** for the two-writer conflict scenario — scaffolded (NOT-VERIFIED) by
   Executor 1; implemented by a later phase/executor with TokenWorld access budget.
</residual-items>

<success_criteria>
Reproduced verbatim from ROADMAP.md § Phase 92 (SC1-8), for catalog-row traceability:

1. Two-writer conflict scenario in dark-factory T4 (rebase recovery) completes step 6 + step
   7 against sim AND TokenWorld (no fresh root commit on helper-side fetch).
2. After every `git push` from a partial-clone working tree (sim + real Confluence + real
   GH issues + real JIRA), `audit_events_cache` AND `audit_events` BOTH show rows for the
   action; `cache.db` is created on first push if missing.
3. `bus_write_audit_completeness.rs` queries both tables; OP-3 dual-table assertion is real
   not metaphorical (closes p83 F3 / p86 F5).
4. Verifier subagent's "honesty spot-check" treats audit-row absence as RED, not "out of
   scope for this layer."
5. Behavioral no-retry verifier at `bus-write-no-helper-retry` replaces source-grep approach.
6. Mid-stream litmus checkpoint: after this phase declares GREEN, re-run dark-factory T1 + T4
   against sim AND TokenWorld; REOPENS on ≥1 HIGH friction.
7. Catalog rows mint NOT-VERIFIED first (with `coverage_kind: real-backend` per RBF-FW-06);
   CLAUDE.md updated in same PR.
8. Phase close: `git push origin main`; verifier subagent grades GREEN; verdict at
   `quality/reports/verdicts/p92/VERDICT.md`.
</success_criteria>

<verdict_location>
quality/reports/verdicts/p92/VERDICT.md
</verdict_location>

<context>
@.planning/milestones/v0.13.0-phases/ROADMAP.md (Phase 92 section)
@.planning/CONSULT-DECISIONS.md (D-P92-01)
@.planning/research/v0.13.0-real-backend-frictions/01-dark-factory-may02/T4-conflict-recovery.md
@.planning/research/v0.13.0-real-backend-frictions/03-synthesis/REMEDIATION-PLAN.md (§ P92, Cluster B, Cluster C)
</context>

<executor_1_findings>
T4's two-writer conflict + rebase-recovery scenario was reproduced end-to-end against a
real sim + real git (container-based, git matching CI's ~2.54; see
`.planning/phases/92-push-flow-correctness/92-T4-REPRO-NOTES.md` for the full transcript
citations). Findings:

- **HIGH-1 (fresh root commit / no ancestry) is FIXED.** Two independent per-writer caches
  (the realistic two-agent topology), A pushes, B's stale push is correctly rejected
  (`version mismatch`, `[remote rejected] ... fetch first`), B's subsequent `git fetch` /
  `git pull --rebase` advances `refs/reposix/origin/main` with the OLD root commit intact
  (`git rev-list --max-parents=0` unchanged across the refetch) — no fresh disconnected
  history. Locked by a regression test (see below).
- **NEW finding, NOT the HIGH-1 mechanism:** `git pull --rebase`'s underlying rebase itself
  still fails on this exact scenario with `fatal: git upload-pack: not our ref <oid>` /
  `could not fetch <oid> from promisor remote` — the cache's delta-sync ("since" cursor)
  reports "0 changed (of 6)" even 2+ seconds after the conflicting writer's push landed, so
  the rebase's 3-way merge needs a blob the cache never lazily materialized. This blocks
  "step 6 completes" (SC1's literal wording) even though the ancestry/root-commit half is
  fixed. Filed to `GOOD-TO-HAVES.md`/`SURPRISES-INTAKE.md` (see NOTICED section of the
  executor's final report) — NOT fixed in this session (Rule 4: touches cache delta-sync
  cursor logic, a different root cause than cb630e5, needs its own investigation).
- **Separate, git-version-dependent finding:** stock Ubuntu 24.04 git (2.43.0) fails EVERY
  real single-backend `git push` outright (`stateless-connect git-receive-pack` rejected
  with a custom string instead of the protocol's `fallback` sentinel, per
  `git-remote-helpers(7)` — "just exiting with error message printed" means "don't bother
  trying to fall back"). CI's runner (git 2.54.0) and this dev box's system git (2.25.1,
  gated NOT-VERIFIED) don't hit it. Filed as a GOOD-TO-HAVES item — real user impact on a
  widely-deployed LTS git version, but orthogonal to T4/HIGH-1.
</executor_1_findings>
