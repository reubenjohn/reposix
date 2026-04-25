# Phase 35: CLI pivot — `reposix init` replacing `reposix mount` + agent UX validation - Context

**Gathered:** 2026-04-24
**Status:** Ready for planning
**Mode:** Auto-generated (discuss skipped via workflow.skip_discuss=true)

<domain>
## Phase Boundary

Replace the `reposix mount` command with `reposix init <backend>::<project> <path>` and run the dark-factory acceptance test. After this phase, a fresh subprocess agent with no reposix CLI awareness completes a clone → grep → edit → commit → push → conflict → pull --rebase → push cycle against the simulator AND at least one real backend (Confluence "TokenWorld" space, GitHub `reubenjohn/reposix` issues, or JIRA project `TEST`) using only standard git/POSIX tools.

This is the headline "pure git, zero in-context learning" milestone. The CLI surface shrinks (one new command, one removed command), but the test surface is the largest in the v0.9.0 sequence: the dark-factory regression test must be **executable outside the reposix CLI** so an autonomous Claude can be spawned in an empty directory, handed only `reposix init` + a goal, and observed completing the task.

Latency for each golden-path step is captured and committed under `docs/benchmarks/v0.9.0-latency.md` as a sales asset. Soft thresholds documented (sim cold clone < 500ms, real backend < 3s); regressions flagged but not CI-blocking.

`docs/reference/testing-targets.md` is created here, documenting the three canonical test targets (TokenWorld, `reubenjohn/reposix`, JIRA `TEST`) with env-var setup and the explicit "go crazy, it's safe" permission statement from the owner.

</domain>

<decisions>
## Implementation Decisions

### Operating-principle hooks (non-negotiable — per project CLAUDE.md)

- **Agent UX = pure git, zero in-context learning (architecture-pivot-summary §4).** The dark-factory regression test is the operationalization of this claim. If the test fails, the architecture's central thesis fails.
- **Close the feedback loop (OP-1).** Acceptance test runs in CI and on local dev (the Phase 36 skill invokes it). The test transcript is committed as a fixture so any regression that breaks the dark-factory flow shows up in `git diff`.
- **Ground truth obsession (OP-6) — real-backend testing.** Per project CLAUDE.md OP-6: simulator-only coverage does NOT satisfy this requirement. At least one real backend MUST be exercised under the same suite for v0.9.0 to ship. Canonical targets:
  - **Confluence "TokenWorld" space** — owner-sanctioned scratchpad; "go crazy, it's safe" permission documented.
  - **GitHub `reubenjohn/reposix` issues** — the project's own issue tracker; eat-our-own-dogfood signal.
  - **JIRA project `TEST`** — overridable via `JIRA_TEST_PROJECT` / `REPOSIX_JIRA_PROJECT`.
- **Ground-truth artifact (OP-6).** The agent's transcript is captured as a test fixture (committed under `tests/fixtures/dark-factory-transcript.txt` or similar). Latency rows likewise committed under `docs/benchmarks/v0.9.0-latency.md`.
- **Audit log non-optional (OP-3).** Real-backend test runs write to the audit log. Test asserts the audit log contains the expected sequence: `(materialize, materialize, ..., push_accept)` or `(materialize, ..., push_reject, materialize, ..., push_accept)` for the conflict-rebase path.

### New CLI surface (locked)

```
reposix init <backend>::<project> <path>
```

Behaviour (architecture-pivot-summary §5):

1. `git init <path>` — creates a regular working tree.
2. `git -C <path> config extensions.partialClone origin` — enables partial-clone semantics.
3. `git -C <path> config remote.origin.url reposix::<backend>/<project>` — points origin at the helper.
4. `git -C <path> config remote.origin.promisor true` — marks origin as the promisor.
5. `git -C <path> config remote.origin.partialclonefilter blob:none` — sets the filter default.
6. `git -C <path> fetch --filter=blob:none origin` — bootstraps the partial clone.
7. (Optional) leaves the working tree empty for sparse-first agents; does NOT auto-`git checkout` (sparse-checkout setup is the agent's call).

`reposix mount` is **removed** (not deprecated) in this same release. Running it emits a one-line migration message: `reposix mount has been removed in v0.9.0. Use 'reposix init <backend>::<project> <path>' — see CHANGELOG and docs/reference/testing-targets.md.`

### Dark-factory regression test (locked)

The headline acceptance test. Architecture:

- A fresh subprocess Claude (or a scripted shell agent acting as one) is spawned inside an empty directory.
- It is given ONLY a `reposix init` command + a natural-language goal: "find issues mentioning 'database' and add a TODO comment to each."
- It MUST complete the task using pure git/POSIX tools: `cd /tmp/repo && cat issues/<id>.md && grep -r database . && <edit> && git add . && git commit && git push`.
- The test exercises three paths:
  1. **Happy path** — clone → grep → edit → commit → push → success.
  2. **Conflict path** — second writer mutates one of the agent's target issues mid-flight; agent observes `! [remote rejected]`, runs `git pull --rebase`, retries `git push`, succeeds.
  3. **Blob-limit path** — naive `git grep` triggers the Phase 34 blob-limit error; agent reads the error message, runs `git sparse-checkout set issues/PROJ-24*`, retries, succeeds.
- The transcript is committed as a fixture so regressions show up in `git diff`.
- The test is **executable outside the reposix CLI** — a wrapper script under `scripts/dark-factory-test.sh` (or similar) that takes a backend target as argument: `scripts/dark-factory-test.sh sim` for the autonomous-mode default, `scripts/dark-factory-test.sh github` / `confluence` / `jira` for real-backend validation.

### Latency capture (locked)

- A committed benchmark script (Claude's discretion: `scripts/bench-v0.9.0-latency.sh` or a Rust binary under `crates/reposix-cli/src/bin/`) runs the golden path and captures wall-clock latency per step.
- Output: `docs/benchmarks/v0.9.0-latency.md` — a Markdown table with columns for sim + each real backend exercised. Soft thresholds documented inline; regressions flagged inline.
- Steps benchmarked: clone, first-blob fetch, sparse-batched checkout (10 blobs), single-issue edit, push, conflict reject, pull-rebase, push-again.
- Soft thresholds: sim cold clone < 500ms, real backend < 3s. Not CI-blocking.

### Real-backend integration run (locked)

- Pass against ≥1 of {TokenWorld, `reubenjohn/reposix`, JIRA `TEST`}.
- Falls back to `#[ignore]` skip when credentials absent, with a clear WARN on stdout: `WARN: v0.9.0 acceptance unverified for backend <X> — credentials missing. See docs/reference/testing-targets.md.`
- Phase 36 wires three CI jobs (`integration-contract-{confluence,github,jira}-v09`) per ARCH-19; Phase 35 only requires that the test infrastructure is in place and at least one backend run is recorded in the SUMMARY.

### `docs/reference/testing-targets.md` (created here, finalized in Phase 36)

Required content per ARCH-18:

- Enumerate the three targets.
- Env-var setup for each: `GITHUB_TOKEN`, `ATLASSIAN_API_KEY` + `ATLASSIAN_EMAIL` + `REPOSIX_CONFLUENCE_TENANT`, JIRA equivalent.
- Rate-limit notes per backend.
- Cleanup procedure ("do not leave junk issues lying around — clear `kind=test` after each run").
- Owner's explicit permission statement: "TokenWorld is for testing — go crazy, it's safe."
- Cross-link from CLAUDE.md's "Commands you'll actually use" section (Phase 36 wires this link).

### Test surface

- `reposix_init_produces_partial_clone` — `reposix init sim::proj-1 /tmp/repo` produces a directory where `git rev-parse --is-inside-work-tree` is true, `git config remote.origin.url` returns `reposix::sim/proj-1`, `extensions.partialClone` is set, and `.git/objects` has tree objects but no blob objects until `git checkout` runs.
- `reposix_mount_emits_migration` — `reposix mount /path` exits non-zero with the migration message.
- `dark_factory_sim_happy_path` — full transcript runs against simulator.
- `dark_factory_sim_conflict_path` — full transcript with mid-flight second writer.
- `dark_factory_sim_blob_limit_path` — full transcript with naive `git grep` recovery via sparse-checkout.
- `dark_factory_real_backend` — same harness pointed at one of the three real backends (gated by env vars).
- `latency_artifact_exists_and_has_sim_column` — `docs/benchmarks/v0.9.0-latency.md` exists and includes a sim column.
- `testing_targets_doc_exists` — `docs/reference/testing-targets.md` exists with the three target sections.

### Claude's Discretion

The exact agent harness (subprocess Claude with system prompt vs. scripted shell agent) is at Claude's discretion. The acceptance criterion is "no reposix CLI knowledge required beyond `reposix init`." A simpler scripted agent that demonstrates the same path is acceptable for v0.9.0 — the spawned-Claude harness can land in v0.10.0 if the simpler version proves the architecture.

CHANGELOG `[v0.9.0]` content is at Claude's discretion as long as the breaking-change migration note is present (`reposix mount /path` → `reposix init <backend>::<project> /path`).

</decisions>

<code_context>
## Existing Code Insights

### Reusable assets

- `crates/reposix-cli/src/main.rs` — existing top-level CLI dispatch (clap-based). Add `init` subcommand alongside the existing `mount`/`demo`/`list`/`refresh`. Mark `mount` as removed-with-message.
- `crates/reposix-cli/src/mount.rs` — current mount handler. Phase 35 replaces its body with the migration-message error; Phase 36 deletes the file outright when FUSE is removed.
- `crates/reposix-cli/src/demo.rs` — canonical end-to-end demo against the simulator. Phase 35 updates this to use `reposix init` instead of `reposix mount`. The benchmarks script can leverage demo's existing setup harness.
- `crates/reposix-cli/src/sim.rs` — embedded simulator startup; reused by the dark-factory test harness.
- Existing real-backend integration tests (Phase 11 Confluence, Phase 28 JIRA, Phase 8 GitHub) — gated `#[ignore]` patterns; reuse the gating idiom.
- `scripts/demo.sh`, `scripts/demos/*` — existing scripted demo patterns; Phase 35 adds `scripts/dark-factory-test.sh` in the same idiom.

### Established patterns

- CLI subcommand error messages exit non-zero with stderr text (no panics).
- Real-backend tests guard on env vars: `if std::env::var("GITHUB_TOKEN").is_err() { skip!() }`.
- Scripts are bash with `set -euo pipefail` and verified by green-gauntlet.

### Integration points

- Phase 31's cache + Phase 32's helper + Phase 33's delta sync + Phase 34's push guardrails all converge here. Phase 35 is the first phase that exercises all four together end-to-end.
- Phase 36 wires CI jobs and the `reposix-agent-flow` skill that *invokes* this phase's dark-factory test. Phase 35 ships the test; Phase 36 ships the harness/CI/skill that runs it.

</code_context>

<specifics>
## Specific Ideas

- The `reposix init` subcommand may delegate to a thin Rust wrapper around `git` (using `std::process::Command` to invoke `git init`, `git config`, `git fetch`) — no need for `git2` or `gix` here, the CLI surface is six shell-out steps.
- The migration message for `reposix mount` is one line, no panic, exit code 2 (CLI usage error).
- `JIRA_TEST_PROJECT` / `REPOSIX_JIRA_PROJECT` overrides are Phase 35's responsibility — wire them through the existing `crates/reposix-jira` config layer.
- Latency rows use `chrono::Duration` formatted as milliseconds (e.g. `42ms`, `1.4s`). Markdown table format is human-readable.
- The dark-factory test fixture is committed as plain text under `tests/fixtures/dark-factory-<backend>-transcript.txt`; per backend exercised. The fixture is the test's *expected output*, not its *input* — diffs are reviewable.
- "Go crazy, it's safe" permission statement is a literal quote in `docs/reference/testing-targets.md` to short-circuit any future agent's "should I really write to this?" hesitation.
- v0.9.0 `Cargo.toml` workspace version bump is part of this phase OR Phase 36 — Claude's discretion, but the CHANGELOG entry must land in Phase 35 alongside the breaking CLI change.

</specifics>

<deferred>
## Deferred Ideas

- Spawned-subprocess-Claude agent harness (vs. scripted shell agent) — v0.10.0 if a simpler harness suffices for v0.9.0 acceptance.
- Three CI jobs `integration-contract-{confluence,github,jira}-v09` — Phase 36 (ARCH-19).
- Per-field merge on conflict — v0.10.0+ (also deferred from Phase 34).
- Cleanup automation for real-backend test artifacts (auto-deleting test issues after a run) — v0.10.0+.
- `reposix init --sparse <pathspec>` flag to set sparse-checkout in one shot — v0.10.0 UX polish.

</deferred>
</content>
