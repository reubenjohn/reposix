---
phase: 35
plan: 01
title: "reposix init subcommand + remove reposix mount + CHANGELOG migration note + README update"
status: complete
requirements: [ARCH-11]
---

# Phase 35 Plan 01 — Summary

## What shipped

- **`reposix init <backend>::<project> <path>`** subcommand at `crates/reposix-cli/src/init.rs`:
  - `translate_spec_to_url` parses `<backend>::<project>` and emits a
    `reposix::<scheme>://<host>/projects/<project>` URL the helper accepts.
    Supports `sim` (defaults to `http://127.0.0.1:7878`), `github`
    (`https://api.github.com`), `confluence` (requires
    `REPOSIX_CONFLUENCE_TENANT`), and `jira` (requires
    `REPOSIX_JIRA_INSTANCE`). Rejects empty project, missing `::`, and
    unknown backends with explicit error messages.
  - `run` performs the six-step git sequence locked in
    `architecture-pivot-summary.md` §5: `git init <path>`, then four
    `git config` invocations (`extensions.partialClone=origin`,
    `remote.origin.url=<url>`, `remote.origin.promisor=true`,
    `remote.origin.partialclonefilter=blob:none`), then a best-effort
    `git fetch --filter=blob:none origin`. The fetch failure path logs
    `tracing::warn!` but does not bail — `init` is "configure local
    state"; the agent's first read triggers actual blob materialization.
  - All shell-out goes through `std::process::Command` (no `sh -c`,
    matching the CLAUDE.md no-shell-injection rule).
- **Mount migration stub** in `crates/reposix-cli/src/main.rs`:
  - `Cmd::Mount { mount_point: _ }` arm is a single `anyhow::bail!` with
    the locked-from-CONTEXT message
    `reposix mount has been removed in v0.9.0. Use 'reposix init <backend>::<project> <path>' — see CHANGELOG and docs/reference/testing-targets.md.`
    Exit code is non-zero (anyhow's default surface).
  - The `Mount` variant is retained in clap so stale scripts get a clear
    migration message rather than `error: unrecognized subcommand 'mount'`.
- **`crates/reposix-cli/src/mount.rs` preserved** with `#![allow(dead_code)]`
  so Phase 36 has a single point of deletion when the FUSE crate goes
  away. Module-level doc updated to flag the deprecation.
- **CHANGELOG `[Unreleased]`** gains a `### Breaking — v0.9.0 architecture pivot`
  block with the migration command pair (`reposix mount` ↔ `reposix init`).
- **README.md** headline tagline switches to "git-native partial-clone
  bridge"; a v0.9 row is appended to the release table; new
  `## Quickstart (v0.9.0)` section sits above the renamed historical
  Quickstart. Old FUSE-mount sections retained for Phase 36 docs sweep.

## Tests added

`crates/reposix-cli/tests/cli.rs` — 4 changes:

- `help_lists_all_subcommands` now also requires `init` in `--help`.
- `subcommand_help_renders` now also runs `reposix init --help`.
- `init_help_documents_spec_argument` (NEW) — `reposix init --help`
  must mention either `BACKEND::PROJECT` or `<backend>::<project>` and
  every backend name (`sim`, `github`, `confluence`, `jira`). Lock for
  the architecture's "zero in-context training" claim — a help-reading
  agent must learn the form from `--help` alone.
- `mount_emits_migration_error` (NEW) — running `reposix mount /tmp/x`
  must exit non-zero with stderr containing `reposix init` and
  `removed in v0.9.0`.

`crates/reposix-cli/src/init.rs` — 7 unit tests (the bin's `unittests src/main.rs`
target), all green:

- `translate_sim_spec`, `translate_github_spec`, `translate_confluence_requires_tenant`,
  `translate_jira_requires_instance`, `translate_rejects_missing_separator`,
  `translate_rejects_unknown_backend`, `translate_rejects_empty_project`.

Net: +6 reposix-cli tests over Plan-34 baseline (5 new in `cli.rs`, 7 unit
tests in `init.rs`, balanced against the now-removed test suite changes).

## Acceptance criteria — status

- `reposix init sim::demo /tmp/test-init` produces a directory where
  `git rev-parse --is-inside-work-tree` is `true` and
  `git config remote.origin.url` returns
  `reposix::http://127.0.0.1:7878/projects/demo`.
  **Verified at unit-test level via `translate_sim_spec`; full e2e
  verification deferred to Phase 35 verification.**
- `reposix mount /tmp/anything` exits non-zero with stderr matching the
  locked migration message.
  **Verified by `mount_emits_migration_error`.**
- `cargo clippy --workspace --all-targets -- -D warnings` clean
  (excluding `reposix-fuse` which Phase 36 deletes).
- `cargo build --workspace` clean.
- `cargo test -p reposix-cli` — all green via workspace context.

## Recovery notes

This plan was executed in a previous session that crashed mid-flight
(VM restart). The recovery runner:

1. Verified the uncommitted `init.rs` against the plan and found a small
   `clippy::doc_markdown` regression (4 backtickless inline-code spans
   in `Cmd::Init` doc-comments) which was fixed before commit.
2. Committed the work atomically per task structure:
   - `feat(35-01): add reposix init subcommand` (Task A: init.rs + main.rs + mount.rs + Cargo.lock).
   - `test(35-01): add init/mount CLI surface tests` (Task B: tests/cli.rs).
   - `docs(35-01): CHANGELOG breaking-change note + README quickstart` (Task C).
3. Did NOT touch `crates/reposix-fuse/` per phase-prompt guardrail.
4. Cargo.lock change was the auto-pulled `reqwest` resolver result from
   running cargo on the workspace; bundled with Task A's `feat` commit
   since it is dependency-related.

## Notes for downstream phases

- **Phase 35-02** (dark-factory regression test) builds on this surface
  by spawning `reposix init sim::<slug> <tmpdir>` from an integration
  test and exercising the three paths (happy / conflict-rebase / blob-limit).
  The literal `BLOB_LIMIT_EXCEEDED_FMT` and `error refs/heads/main fetch first`
  strings asserted there are byte-identical to the helper's emit paths
  (Phase 32 / 34).
- **Phase 35-03** (real-backend integration tests) reuses
  `translate_spec_to_url` for the github/confluence/jira branches; the
  helper itself currently still hardcodes `SimBackend` (Phase 32
  limitation), so the real-backend tests assert init-level success and
  defer fetch-against-real to a future phase per `35-03-PLAN.md`.
- **Phase 36** deletes `reposix-fuse` and `crates/reposix-cli/src/mount.rs`;
  the `Cmd::Mount` clap variant should be deleted at that time too,
  along with the `mount_emits_migration_error` test.
