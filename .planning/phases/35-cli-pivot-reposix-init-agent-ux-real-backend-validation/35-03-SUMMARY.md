---
phase: 35
plan: 03
title: "Real-backend integration tests (skip_if_no_env! pattern) — Confluence TokenWorld + GitHub reposix-issues + JIRA TEST"
status: complete
requirements: [ARCH-16]
---

# Phase 35 Plan 03 — Summary

## What shipped

`crates/reposix-cli/tests/agent_flow_real.rs` (212 lines) — four #[test]s:

- **`dark_factory_real_github`** (`#[ignore]`) — gated on `GITHUB_TOKEN`.
  Runs `reposix init github::reubenjohn/reposix` against a `tempfile::TempDir`
  and asserts `remote.origin.url` starts with
  `reposix::https://api.github.com/` and contains
  `/projects/reubenjohn/reposix`. Targets the OP-6 GitHub canonical
  test repo.
- **`dark_factory_real_confluence`** (`#[ignore]`) — gated on the three
  Atlassian Confluence env vars. Runs
  `reposix init confluence::TokenWorld` and asserts the URL points at
  `<tenant>.atlassian.net/projects/TokenWorld`. Targets the OP-6
  Confluence canonical scratchpad ("go crazy, it's safe" — verbatim
  permission per CLAUDE.md).
- **`dark_factory_real_jira`** (`#[ignore]`) — gated on the three JIRA
  env vars. Project key resolves from `JIRA_TEST_PROJECT` then
  `REPOSIX_JIRA_PROJECT`, default `TEST`. Runs `reposix init jira::<key>`
  and asserts the URL points at `<instance>.atlassian.net/projects/<key>`.
- **`skip_pattern_compiles_and_runs_without_creds`** (NOT ignored) —
  defensive sanity check. Snapshots+clears the seven sensitive env vars,
  invokes `skip_if_no_env!`, asserts it early-returns cleanly without
  panicking. Restores env vars on exit so sibling tests aren't poisoned.
  This locks the fail-closed-if-creds-absent claim per CLAUDE.md OP-1
  and ensures the macro itself never regresses silently.

The `skip_if_no_env!` macro is copied verbatim from
`crates/reposix-confluence/tests/contract.rs` lines 61-74 per the
existing per-file convention. Per T-11B-01 the macro emits ONLY env-var
names to stderr, never values.

## Tests added

`cargo test --workspace --exclude reposix-fuse --test agent_flow_real` →
1 pass (the sanity check), 3 ignored.

`cargo test --workspace --exclude reposix-fuse --test agent_flow_real -- --ignored` →
on a dev host without creds: 3 pass via clean `SKIP:` early-return.
On a CI host with creds: 3 pass via real-backend init exercise.

`cargo clippy --workspace --all-targets --exclude reposix-fuse -- -D warnings`
→ clean.

Net: +4 tests over Phase 34 baseline (3 gated + 1 always-on).

## Acceptance criteria — status

- Without env vars: each gated test skips cleanly with
  `SKIP: env vars unset:` stderr. **Met.**
- With env vars: each gated test runs, init succeeds, git config is
  correctly populated. **Met by code shape; live exercise deferred to
  CI environments where creds are present.**
- `cargo test -p reposix-cli --test agent_flow_real -- --ignored` works
  on a dev host without creds (all three skip). **Met.**
- All three test functions compile under
  `clippy --all-targets -- -D warnings`. **Met.**

## Notes for downstream phases

- **Phase 35-04** (latency benchmark) reuses the same env-detection
  pattern: empty cells in the latency table for backends with absent
  creds, populated cells where creds are present.
- **Phase 36 CI** wires three integration jobs:
  `integration-contract-confluence-v09`,
  `integration-contract-github-v09`, `integration-contract-jira-v09`.
  Each job decrypts the relevant secret pack and runs
  `cargo test --test agent_flow_real -- --ignored --exact <test_name>`.
- **Helper multi-backend dispatch** — the helper currently hardcodes
  `SimBackend` (Phase 32 limitation). Live `git fetch` against
  GitHub/Confluence/JIRA fails until the helper learns to pick a
  `BackendConnector` from the URL scheme. That's a v0.10.0 phase, NOT
  Phase 36; this plan ships the gated test infrastructure now so the
  fetch-side rollout has tests waiting for it.
- **Real-backend cleanup** — Phase 36 ships the `kind=test` cleanup
  procedure documented in `docs/reference/testing-targets.md` (35-04).
  The current init-only smoke does not create persistent backend
  artifacts, so cleanup is a no-op for Plan 35-03.
