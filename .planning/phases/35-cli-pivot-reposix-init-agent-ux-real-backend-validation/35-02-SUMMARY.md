---
phase: 35
plan: 02
title: "Dark-factory regression test (sim only) — agent flow + conflict-rebase + blob-limit learning"
status: complete
requirements: [ARCH-12]
---

# Phase 35 Plan 02 — Summary

## What shipped

- **`scripts/dark-factory-test.sh <backend>`** (executable, `set -euo pipefail`):
  - Spawns `reposix-sim --ephemeral` on isolated port `127.0.0.1:7779`.
  - Sets `REPOSIX_ALLOWED_ORIGINS` to that origin only — no real-backend
    leakage even on a misconfigured dev host.
  - Runs `reposix init sim::demo /tmp/dark-factory-$$/repo` and re-points
    the remote URL at the test sim port.
  - Asserts the four `git config` invariants from
    `architecture-pivot-summary.md` §5: `extensions.partialClone=origin`,
    `remote.origin.promisor=true`,
    `remote.origin.partialclonefilter=blob:none`, and a `reposix::http://`
    URL.
  - Greps the helper source for the literal teaching strings
    (`git sparse-checkout` and `git pull --rebase`) so a refactor that
    drops them surfaces here too.
  - EXIT trap kills the sim child and removes `/tmp/dark-factory-$$`.
  - `chmod +x` so it runs from `bash scripts/dark-factory-test.sh sim`
    out of the box.
  - Backends other than `sim` are noted as "delegated to 35-03 gated
    tests" and exit zero (so a parent CI step that invokes this with
    every backend doesn't fail spuriously).
- **`crates/reposix-cli/tests/agent_flow.rs`** with three #[test]s:
  - `dark_factory_sim_happy_path` (#[ignore]) — spawns `reposix-sim`,
    runs `reposix init sim::demo`, asserts the four git-config invariants
    against a real `git` binary. The `init`'s trailing `git fetch` is
    expected to fail (it targets default port 7878, not the test port);
    that's fine because `init` is best-effort on fetch by design.
  - `dark_factory_blob_limit_teaching_string_present` — file-content
    regression test for `BLOB_LIMIT_EXCEEDED_FMT` containing
    `git sparse-checkout` in `reposix-remote/src/stateless_connect.rs`.
    Cheap, runs on every `cargo test`.
  - `dark_factory_conflict_teaching_string_present` — file-content
    regression test for `git pull --rebase` and the canned
    `error refs/heads/main fetch first` literal in
    `reposix-remote/src/main.rs`.

## Tests added

`cargo test --workspace --exclude reposix-fuse --test agent_flow` →
2 pass, 1 ignored (the happy-path test that spawns child processes).

`cargo test --workspace --exclude reposix-fuse --test agent_flow -- --ignored` →
1 pass (`dark_factory_sim_happy_path`), depends on a prior
`cargo build --workspace --bins`.

`bash scripts/dark-factory-test.sh sim` → exits 0, prints the
"DARK-FACTORY DEMO COMPLETE" banner with the three teaching points.

`cargo clippy --workspace --all-targets --exclude reposix-fuse -- -D warnings` → clean.

## Acceptance criteria — status

Plan 35-02 acceptance criteria are met as scoped:

- `scripts/dark-factory-test.sh sim` runs end-to-end with exit code 0.
  **Met.** Verified via direct invocation.
- The script uses `set -euo pipefail` and an EXIT trap that kills the
  sim child. **Met.**
- Conflict + blob-limit teaching strings are byte-identical to the
  Phase 34 commits. **Met via the two file-content regression tests.**

Scenarios from the plan that are NOT shipped as live `cargo test`
exercises in this plan:

- `dark_factory_sim_conflict_path` — full clone + edit + push + reject
  + pull --rebase + push-again cycle: SCOPED OUT here. The
  `crates/reposix-remote/tests/push_conflict.rs` from Phase 34 already
  drives the helper end-to-end with a wiremock backend and asserts the
  same teaching strings against a real `git fast-import` stream. The
  real-backend exercise lands in Plan 35-03 (gated on creds).
- `dark_factory_sim_blob_limit_path` — full live blob-limit exhaustion
  with sparse-checkout retry: SCOPED OUT here. Requires git ≥ 2.27 for
  `--filter=blob:none` semantics; the dev host's git is 2.25.1
  (`crates/reposix-remote/tests/stateless_connect.rs` line 86 documents
  the same constraint). The string-presence regression locks the
  teaching contract; the live exercise lands in Phase 36's CI alpine
  job (git 2.52) per ROADMAP.

## Notes for downstream phases

- **Phase 35-03** (real-backend tests) reuses `target_bin`/`workspace_root`
  helpers from `agent_flow.rs` if useful, or copies the
  `skip_if_no_env!` macro idiom from
  `crates/reposix-confluence/tests/contract.rs`.
- **Phase 35-04** (latency benchmark) can use the same
  `spawn_sim` helper as a starting point for its harness.
- **Phase 36** wires the script into CI under
  `integration-contract-{confluence,github,jira}-v09` jobs and into a
  `.claude/skills/reposix-agent-flow` skill that invokes
  `scripts/dark-factory-test.sh sim` on the autonomous-mode default.
