# P92 T4 prove-before-fix — repro notes (DP-2 discipline)

**Author:** P92 Executor 1. **Date:** 2026-07-05. **Method:** container-based (`docker run
ubuntu:24.04` + `git-core` PPA to match CI's runner git ~2.54.0, `--network host` to reach a
`reposix-sim` spawned on the host at `127.0.0.1:7878`, workspace binaries built once on the
host and bind-mounted read-only). Full raw transcripts are session-scratchpad-only (not
committed — they contain no secrets but are pure debug noise); this file is the citable
distillation the PLAN.md's `<executor_1_findings>` section points at.

## Why a container

This dev box's system git is 2.25.1 — below the project's `git >= 2.34` requirement
(`CLAUDE.md` Tech stack), so `reposix init`'s partial-clone fetch (`stateless-connect`)
cannot even complete natively here (matches the existing `agent-ux/real-git-push-e2e`
catalog row's own NOT-VERIFIED artifact, `git_too_old`, same box). CI's runner (`gh run view
28726703296`, `actions/checkout@v7` step) reports `git version 2.54.0`. The `git-core` PPA
installed into a throwaway `ubuntu:24.04` container reproduces that exact version, giving an
apples-to-apples proof environment instead of a NOT-VERIFIED short-circuit.

## Repro topology

Two independent working trees (`A`, `B`), each with its OWN `REPOSIX_CACHE_DIR` (two
separate bare caches) — the realistic two-agent/two-machine topology, matching the original
May-02 T4 dark-factory test's structure (two independent `/tmp/conflict-A` /
`/tmp/conflict-B` checkouts, no shared cache). An earlier attempt shared ONE cache between A
and B and found conflict detection did not fire (the shared cache's own delta-sync
absorbed A's write before B's precheck ran) — that is a real but DIFFERENT scenario (single
machine, one cache, two working trees) from the T4 finding's intent, so the regression test
targets the two-cache topology.

## Sequence + observed results (git 2.54.0, two independent caches)

1. `reposix init sim::demo A` / `reposix init sim::demo B` (separate `REPOSIX_CACHE_DIR`
   each) — both succeed, both land on the same root commit (`sync(sim:demo): 6 issues`).
2. A edits `issues/1.md`, commits, `git push origin main` — **exit 0**, `[new branch] main
   -> main`.
3. B edits the SAME file (stale base — B's cache never saw A's write), commits, `git push
   origin main` — **exit 1**, `error: patch issue 1: version mismatch: current=2
   requested=1`, `! [remote rejected] main -> main (some-actions-failed)`. Matches the
   documented "WIN" half of the original T4 test exactly.
4. B recovers: `git pull --rebase origin main`. The FETCH leg succeeds and ADVANCES
   `refs/reposix/origin/main` (`094eb7f..9d35135 main -> refs/reposix/origin/main` in one
   run). **Key assertion: `git rev-list --max-parents=0 refs/reposix/origin/main` before and
   after this fetch is IDENTICAL** — the original HIGH-1 symptom ("helper mints a fresh root
   commit per fetch, no ancestry to the prior tip") does NOT reproduce. Confirmed in TWO
   independent runs (git 2.43.0 single-cache topology, and git 2.54.0 two-cache topology).
5. The rebase step ITSELF then fails with `fatal: git upload-pack: not our ref <oid>` /
   `could not fetch <oid> from promisor remote` — a SEPARATE, newly-discovered bug (cache
   delta-sync reports "0 changed (of 6)" even 2+ seconds after the conflicting write landed,
   so the blob the 3-way merge needs was never lazily materialized into B's cache). This
   blocks "step 6 completes" in SC1's literal wording even though the HIGH-1 ancestry
   mechanism is fixed. NOT fixed in this session — filed (see PLAN.md
   `<executor_1_findings>` + the executor's NOTICED/RAISE-LIST report to L1).

## DP-2 verdict

**GREEN on the narrow HIGH-1 claim** ("does a fresh root commit appear after the helper
refetch") — ancestry is preserved, confirmed twice, across two topologies and two git
versions (2.43.0, 2.54.0). Per the prove-before-fix protocol this is locked as a regression
gate (`quality/gates/agent-ux/t4-conflict-rebase-ancestry.sh`) asserting NO fresh root
commit after a refetch that follows a real conflicting push — proven to BITE by
temporarily reintroducing the pre-`cb630e5` `ensure_hide_sync_refs` shell-out (no env scrub)
and confirming the gate fails (RED) against that reverted code, then reverting back to
confirm GREEN against current `main`. See `.planning/CONSULT-DECISIONS.md` `[SELF]` entry
for the full DP-2 disposition.

**Separately (not part of the locked regression, filed as new findings, NOT designed a fix
for either — Rule 4, different root causes than cb630e5):**
- Cache delta-sync "0 changed" false-negative blocking the rebase's blob materialization.
- git 2.43.0 (stock Ubuntu 24.04) fails ALL single-backend real pushes outright because the
  helper answers a `stateless-connect git-receive-pack` probe with a custom error string
  instead of the `git-remote-helpers(7)`-mandated `fallback` sentinel — per the spec, any
  reply other than `fallback` means "don't bother trying to fall back," so git never
  attempts the `export` capability that push actually needs. Git 2.54.0 (this project's CI
  version) does not hit it, nor does an old-enough git (< the version that started trying
  `connect`-family capabilities for push) — this is a real, version-windowed compatibility
  gap on a currently-supported LTS git, not a regression this phase introduced.
