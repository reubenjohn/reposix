---
phase: 104-github-helper-path-404-fix
slug: S-260707-gh404
milestone: v0.14.0 (wave-2)
verified: 2026-07-12T07:13:26Z
status: passed
verdict: GREEN
score: 1/1 gradeable row PASS (+ 1 honest NOT-VERIFIED real-backend row, fail-closed)
verifier: unbiased phase-close verifier (no session context — graded from real execution)
head: 03942f8
rows:
  - id: agent-ux/github-helper-path-slug-not-sanitized
    status: PASS
    source: real gate execution (cargo exit 0)
  - id: agent-ux/github-front-door-real-backend
    status: NOT-VERIFIED
    honest: true  # real-backend, owner-gated, verifier script intentionally absent; fail-closed
constraint_notes:
  - "ONE-cargo budget honored: a concurrent herdr on-demand --persist runner (PID 351077) held the cargo slot mid-verification; I waited it out, then ran the single scoped gate. Independent re-run + its minted artifact agree."
  - "NO real-GitHub call made. Front-door row confirmed honest NOT-VERIFIED, not faked."
  - "Did not touch .planning/ (concurrent herdr-manager writer). Verdict artifact only."
---

# Phase 104 "GitHub helper-path 404 fix" (S-260707-gh404) — Verification Report

Graded against reality, not any executor's word. No pre-written artifact was
trusted — the PASS came from a real `cargo test` exit code.

## Row 1 — agent-ux/github-helper-path-slug-not-sanitized — PASS

- **Gate ran for real.** `quality/gates/agent-ux/github-helper-path-slug-not-sanitized.sh`
  → exit 0, invoking `cargo test -p reposix-cache --test github_project_slug_not_sanitized`
  → `1 passed`. Independently re-run by me (exit 0) AND minted by the concurrent
  runner into `quality/reports/verifications/agent-ux/github-helper-path-slug-not-sanitized.json`
  (ts `2026-07-12T07:10:58Z`, `exit_code: 0`, real cargo stdout embedded).
- **Artifact synthesized from the exit code, not pre-written.** No artifact
  existed before this session; the JSON carries the live cargo transcript
  (`test ... ok`, `finished in 0.04s`) — provably a fresh synthesis.
- **Row was received honest.** On origin/main (HEAD 03942f8) the row was minted
  `NOT-VERIFIED` by the coordinator's remediation commit `dec3aa4` (not
  self-declared PASS) — exactly so an unbiased verifier grades it. It flipped to
  PASS only after real execution.
- **Regression genuinely pinned (not tautological / over-mocked).** The test's
  `RecordingMock` captures the exact `project` string reaching
  `list_records_complete`, then asserts BOTH halves of the split: (1) the backend
  receives the RAW `owner/repo` (never the sanitized `owner-repo` — the 404 bug),
  and (2) `cache.repo_path()` is the flat `github-owner-repo.git`. The module doc
  documents a real FAILS-THEN-PASSES reproduction (pre-fix `resolve_cache_path`
  → nested `github-owner/repo.git` → assertion 2 fails). The gate also greps the
  test body so it can't be gutted to an always-green stub.
- **Root cause genuinely fixed.** No production call site pre-sanitizes before
  `Cache::open`: the three real callers — `sync.rs:105`, `attach.rs:164`,
  `main.rs:293` — all pass the RAW `parsed.project` / `state.project` with explicit
  "pass the RAW project slug" comments. `sanitize_project_for_cache` has exactly
  ONE production caller, `resolve_cache_path` (`reposix-cache/src/path.rs:40`);
  otherwise only the canonical impl (`reposix-core::path`) + the `backend_dispatch.rs`
  re-export + tests. Zero pre-sanitize leaks into the backend-facing slug.
- **CacheCollision teaches the remedy.** `error.rs:35-41` names the S-260707-gh404
  owner/repo identity migration as the cause AND the fix: "Delete the stale cache
  dir and re-run `reposix init`/`reposix attach` to rebuild it from the backend."

## Row 2 — agent-ux/github-front-door-real-backend — NOT-VERIFIED (honest, fail-closed)

- Status `NOT-VERIFIED`; `transport_claim: true`, `coverage_kind: real-backend`,
  `cadences: [pre-release-real-backend]`, `skip_reason: env-missing`,
  `last_real_grade: null`. Verifier script intentionally absent (catalog-first SPEC
  mint) → runner demotes unconditionally to NOT-VERIFIED. NOT faked to PASS. Correct
  per OD-2 — the live GitHub 200 is owner-gated and verified at milestone-close's
  9th probe. I made no api.github.com call.

## Phase-close integrity

- **`ci.yml:263 continue-on-error: true` STILL PRESENT** on
  `integration-contract-github-v09` (KNOWN-LIMITATION comment names S-260707-gh404).
  Correct: the fix is proven at sim/unit only; the real-GitHub transport claim is
  not yet owner-verified, so the marker must NOT be removed. No premature promotion.

## Noticed

- **WARNING #3 already filed (GOOD-TO-HAVES-03, commit 03942f8).** The unit test
  pins `Cache::open` behavior but there is no caller-level gate stopping a future
  caller from re-sanitizing before `Cache::open`. Honestly filed, not a blocker.
- The row's verification artifact has empty `asserts_passed`/`asserts_failed` (it's
  a `mechanical` exit-code row, graded on cargo exit 0 — congruence not applicable).
  Fine, but worth noting the PASS rests entirely on the gate's internal greps +
  cargo exit, which I confirmed hold.
- A concurrent herdr on-demand `--persist` runner was minting the shared catalog
  mid-verification. It agreed with my independent grade, but two `--persist`
  runners on one catalog is a live race hazard — see raise list.

## Raise list

1. **Concurrent `--persist` runners race the shared catalog.** PID 351077 (herdr
   on-demand `--persist`) ran alongside this verification, minting
   `quality/catalogs/agent-ux.json`. Two persisting runners on one catalog file can
   interleave writes. Recommend a catalog-write lock or single-persist-lane
   discipline. (Not this phase's bug; surfaced for the coordinator.)
2. **Caller-path raw-slug guard** — already filed as GOOD-TO-HAVES-03; re-flag for
   scheduling so the anti-404 contract has a caller-level gate, not just a unit test.

---

**VERDICT: GREEN.** The sim/unit contract row is PASS from real cargo execution;
the real-backend row is honest fail-closed NOT-VERIFIED; `ci.yml:263 continue-on-error`
remains present; root cause genuinely fixed with no production pre-sanitize leak.

_Verifier: Claude (unbiased phase-close). Real execution only._
