---
name: reposix-agent-flow
description: "Spawn the reposix dark-factory regression test — a fresh subprocess agent given only `reposix init` and a goal must complete clone+grep+edit+commit+push using pure git/POSIX, including the conflict-rebase and blob-limit recovery cycles. The StrongDM/dark-factory regression harness for the v0.9.0 architecture. Invoked from CI release-gate and from local dev (/reposix-agent-flow)."
argument-hint: "[sim|github|confluence|jira]"
allowed-tools:
  - Bash
  - Read
---

# reposix-agent-flow — dark-factory regression

This skill is the **StrongDM / dark-factory regression harness** for the v0.9.0
architecture pivot (FUSE → git-native partial clone). It encodes the central
thesis from `architecture-pivot-summary.md §4 "Agent UX: pure git, zero
in-context learning"`: a fresh subprocess agent — no reposix CLI awareness, no
in-context system-prompt instructions about reposix, no MCP tool schemas — must
be able to bootstrap a working tree with a single `reposix init` invocation and
then complete its task using only standard git and POSIX tools.

## When to use

- **CI release gate** — wired into `.github/workflows/ci.yml` as the
  `dark-factory` job (sim) and `integration-contract-{confluence,github,jira}-v09`
  jobs (real backends). Phase 36's deletion of the FUSE crate is only safe if
  this regression keeps passing.
- **Local dev** — `/reposix-agent-flow` (or the Skill tool from a coordinating
  agent) before any change touching `crates/reposix-remote/`,
  `crates/reposix-cache/`, or `crates/reposix-cli/src/init.rs`.
- **Pre-tag verification** — `bash scripts/green-gauntlet.sh --full` runs the
  sim variant; the real-backend variants run via the gated cargo tests.

## What it asserts

The architecture's "pure git, zero in-context learning" claim is the contract.
Concretely:

1. **`reposix init <backend>::<project> <path>`** produces a real partial-clone
   working tree: `extensions.partialClone=origin`, `remote.origin.promisor=true`,
   `remote.origin.partialclonefilter=blob:none`, `.git/` is a real git directory.
2. **Conflict path teaches the agent.** When `git push` rejects because the
   backend drifted, the helper's stderr names `git pull --rebase` as the
   recovery move — verbatim. An agent that reads stderr and follows the
   instruction recovers without prompt engineering.
3. **Blob-limit path teaches the agent.** When `command=fetch` would
   materialize more blobs than `REPOSIX_BLOB_LIMIT`, the helper refuses with a
   stderr error that names `git sparse-checkout` as the recovery move.

These are byte-identical regression tests — the teaching strings are the
contract, not implementation details.

## Invocation

```bash
# Default — sim backend, run from anywhere in the workspace.
bash scripts/dark-factory-test.sh sim

# Real-backend variants delegate to the gated cargo tests in
# crates/reposix-cli/tests/agent_flow_real.rs. Each requires the
# corresponding env-var pack (see docs/reference/testing-targets.md):
bash scripts/dark-factory-test.sh github       # → cargo test --ignored dark_factory_real_github
bash scripts/dark-factory-test.sh confluence   # → cargo test --ignored dark_factory_real_confluence
bash scripts/dark-factory-test.sh jira         # → cargo test --ignored dark_factory_real_jira
```

When this skill is invoked from the Skill tool with no argument, default to
`sim` — it's the only variant that runs without external credentials and is
the canonical regression target per project CLAUDE.md OP-1.

## Canonical test targets (real-backend variants)

Per project CLAUDE.md OP-6 and `docs/reference/testing-targets.md`:

- **Confluence space "TokenWorld"** (`reuben-john.atlassian.net`) — owner-
  sanctioned scratchpad. "Go crazy, it's safe."
- **GitHub repo `reubenjohn/reposix` issues** — ours; safe to create/close
  issues during tests. Cleanup is automatic via `gh issue close`.
- **JIRA project `TEST`** — default key, overridable via `JIRA_TEST_PROJECT`
  or `REPOSIX_JIRA_PROJECT`.

The simulator (`reposix-sim`) is always the default and is the only variant
that runs in autonomous mode without explicit credentials.

## How to interpret failures

If `dark-factory-test.sh sim` fails:

- **`reposix init` exit non-zero** — `crates/reposix-cli/src/init.rs`
  regression. Check `extensions.partialClone` config writes.
- **`extensions.partialClone != origin`** — git config not applied.
- **`BLOB_LIMIT teaching string regressed`** — someone changed the error
  message in `crates/reposix-remote/src/stateless_connect.rs`. The literal
  string `git sparse-checkout` is the contract; any rewrite must preserve it.
- **`conflict-rebase teaching string regressed`** — someone changed the
  message in `crates/reposix-remote/src/main.rs`. The literal `git pull
  --rebase` must remain.

## References

- `architecture-pivot-summary.md §4 "Agent UX: pure git, zero in-context
  learning"` — the central design claim this skill regression-tests.
- `scripts/dark-factory-test.sh` — the underlying script.
- `crates/reposix-cli/tests/agent_flow.rs` — the cargo test variant
  (regression-protects the teaching strings via file-content asserts).
- `crates/reposix-cli/tests/agent_flow_real.rs` — real-backend gated tests.
- `docs/reference/testing-targets.md` — env-var setup, cleanup procedure,
  owner permission statement.
- `.github/workflows/ci.yml` — `dark-factory` job + three
  `integration-contract-*-v09` jobs that wire this skill into CI.
