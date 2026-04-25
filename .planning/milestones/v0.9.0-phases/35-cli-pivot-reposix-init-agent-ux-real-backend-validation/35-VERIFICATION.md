---
phase: 35
status: human_needed
verifier: phase-runner (recovery)
date: 2026-04-24
---

# Phase 35 — Verification Report

## Status

**`human_needed`** — all sim-backed verification gates are green. Real-backend
exercise of the dark-factory + latency flow is gated on credential
availability and is currently `pending-secrets` for all three OP-6
targets (TokenWorld, `reubenjohn/reposix`, JIRA TEST). The test
infrastructure is in place; the only thing missing is a CI run with
the relevant secret packs decrypted.

## Gates

| # | Check | Status | Evidence |
|---|-------|--------|----------|
| 1 | `cargo build --workspace` clean | ✅ | `Finished dev profile … in 0.17s` |
| 2 | `cargo clippy --workspace --all-targets --exclude reposix-fuse -- -D warnings` clean | ✅ | `Finished dev profile … in 0.20s` |
| 3 | `reposix init sim::proj-1 /tmp/reposix-init-test` produces partial-clone working tree | ✅ | `git config remote.origin.url` → `reposix::http://127.0.0.1:7878/projects/proj-1`; `extensions.partialClone=origin`; `remote.origin.partialclonefilter=blob:none` |
| 4 | `reposix mount /tmp/anywhere` exits non-zero with migration message | ✅ | `Error: reposix mount has been removed in v0.9.0…`; exit=1 |
| 5 | `cargo test --test agent_flow -- --ignored` passes against sim | ✅ | `test dark_factory_sim_happy_path … ok`. The two non-ignored teaching-string regression tests also pass on default `cargo test`. |
| 6 | `bash scripts/dark-factory-test.sh sim` exit 0 | ✅ | `DARK-FACTORY DEMO COMPLETE — sim backend: agent UX is pure git.` |
| 7 | `docs/reference/testing-targets.md` exists, names all three targets | ✅ | TokenWorld, `reubenjohn/reposix`, JIRA `TEST` (with override resolution chain). 5 occurrences of "go crazy, it's safe"; 5 of `JIRA_TEST_PROJECT`/`REPOSIX_JIRA_PROJECT`. |
| 8 | `docs/benchmarks/v0.9.0-latency.md` exists with sim column populated | ✅ | init=24ms, list=9ms, get=8ms, patch=8ms, cap=5ms. All under 500ms soft threshold. |
| 9 | CHANGELOG `[Unreleased]` mentions breaking CLI change with migration | ✅ | `### Breaking — v0.9.0 architecture pivot` block + `### Added` cross-references. |
| 10 | README updated with `reposix init` quickstart | ✅ | New `## Quickstart (v0.9.0)` section above renamed historical section; v0.9 row in release table. |
| 11 | Workspace test count regressed? | ✅ | 436 tests pass, zero failures. (Phase 34 baseline was the same after the agent_flow + agent_flow_real additions; net +5 tests with the new files.) |
| 12 | Real-backend dark-factory test against ≥1 of {Confluence, GitHub, JIRA} | ⚠️ pending-secrets | `cargo test --test agent_flow_real -- --ignored` skips cleanly without creds. CI with secret packs will populate this. |

## Real-backend coverage

| Target | Test | Status | Reason |
|--------|------|--------|--------|
| Confluence TokenWorld | `dark_factory_real_confluence` | pending-secrets | `ATLASSIAN_API_KEY` / `ATLASSIAN_EMAIL` / `REPOSIX_CONFLUENCE_TENANT` not present in dev env |
| GitHub `reubenjohn/reposix` | `dark_factory_real_github` | pending-secrets | `GITHUB_TOKEN` not present in dev env |
| JIRA project `TEST` | `dark_factory_real_jira` | pending-secrets | `JIRA_EMAIL` / `JIRA_API_TOKEN` / `REPOSIX_JIRA_INSTANCE` not present in dev env |

Per the phase prompt: "If a real-backend live test cannot run because
creds are absent in the dev env: tag with `#[ignore]` + `skip_if_no_env!`,
mark the relevant ROADMAP success-criterion line with 'pending-secrets'
rather than 'passed', document it. Do NOT skip the work; ship the gated
test infrastructure."

This phase complies. Phase 36 wires the
`integration-contract-{confluence,github,jira}-v09` CI jobs that run
the same tests with secrets decrypted.

## ROADMAP success criteria — status

(Inferred from phase prompt + ARCH-11/12/16/17/18/19 references; project
owner should reconcile against the canonical ROADMAP.md when convenient.)

1. **`reposix init <backend>::<project> <path>` replaces `reposix mount`** — ✅ passed.
2. **Mount stub emits migration error** — ✅ passed.
3. **CHANGELOG breaking-change note** — ✅ passed.
4. **README quickstart reflects new flow** — ✅ passed.
5. **Dark-factory regression test against sim** — ✅ passed.
6. **Conflict + blob-limit teaching strings byte-identical to Phase 34** — ✅ passed (regression-protected via file-content tests).
7. **Latency artifact at `docs/benchmarks/v0.9.0-latency.md`** — ✅ passed (sim column populated, real columns gated on creds).
8. **`docs/reference/testing-targets.md` with three sanctioned targets** — ✅ passed.
9. **Real-backend dark-factory exercise against ≥1 of {Confluence, GitHub, JIRA}** — ⚠️ pending-secrets.

## Recovery context

This phase was originally executed in a previous session that crashed
mid-flight (VM restart). Plans 35-01 was partially complete with
uncommitted code; 35-02/03/04 were planned but not executed.

The recovery runner:
1. **Stage 1 — Recovery:** Validated the uncommitted 35-01 code (init.rs +
   main.rs edits + tests/cli.rs + mount.rs deprecation), fixed a small
   `clippy::doc_markdown` regression, and committed the work atomically
   per task structure (`feat(35-01)`, `test(35-01)`, `docs(35-01)`).
   Committed Phase 35 plans + research + validation scaffolds. Wrote
   `35-01-SUMMARY.md`. Bumped STATE.md cursor.
2. **Stage 2 — Sequential plan execution:**
   - 35-02: `scripts/dark-factory-test.sh` + `tests/agent_flow.rs` (3 tests, 1 ignored).
   - 35-03: `tests/agent_flow_real.rs` (4 tests, 3 ignored + 1 sanity check).
   - 35-04: `scripts/v0.9.0-latency.sh` + `docs/benchmarks/v0.9.0-latency.md` + `docs/reference/testing-targets.md` + CHANGELOG cross-links.
3. **Stage 3 — Verification:** This document.

## Notes for Phase 36

- **No new crates** were added; the v0.9.0 pivot continues to be a
  single-crate refactor at the CLI layer + per-existing-crate test
  additions. Phase 36's deletion of `reposix-fuse` doesn't need to
  update the workspace `Cargo.toml` for any 35-introduced crate.
- **CLAUDE.md commands block** is unchanged in this phase — Phase 36's
  docs sweep should add the cross-link to
  `docs/reference/testing-targets.md` per the doc's "Linked from"
  footer.
- **Skill-creation hooks:** Phase 36's
  `.claude/skills/reposix-agent-flow` should invoke
  `bash scripts/dark-factory-test.sh sim` as the autonomous default.
  The script accepts `github`/`confluence`/`jira` arguments but
  delegates to the gated 35-03 cargo tests — Phase 36 may want to
  flip that delegation if the helper has multi-backend dispatch by then.
- **Tag-script template:** `scripts/tag-v0.8.0.sh` exists in repo;
  Phase 36 should clone it to `scripts/tag-v0.9.0.sh`.
- **Stub deletion:** Phase 36's deletion of `reposix-fuse` should also
  delete:
  - `crates/reposix-cli/src/mount.rs` (already `#![allow(dead_code)]`).
  - The `Cmd::Mount` variant in `crates/reposix-cli/src/main.rs`.
  - The `mount_emits_migration_error` test in `crates/reposix-cli/tests/cli.rs`.
  - The `mount` entry in `subcommand_help_renders` and `help_lists_all_subcommands`.

## Approval

`status: human_needed` — the project owner should confirm:
- Whether `pending-secrets` real-backend coverage is acceptable for
  v0.9.0 ship, or whether Phase 36 must run all three CI integration
  jobs green before tag.
- Whether the lean dark-factory test (file-content regression for
  conflict + blob-limit teaching strings, plus a single live happy-path)
  is sufficient for the architecture's "pure git, zero in-context
  learning" claim, or whether Phase 36 should expand to live
  conflict-rebase + blob-limit retry loops once the dev host's git is
  upgraded to ≥2.27 (currently 2.25.1, which is the documented
  blocker for `--filter=blob:none` integration tests).
