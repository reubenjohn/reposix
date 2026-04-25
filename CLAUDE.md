# CLAUDE.md — reposix project guide

This file is read by every agent (Claude Code, Codex, Cursor, etc.) that opens this repo. It's the local extension of the user's global CLAUDE.md (`~/.claude/CLAUDE.md`) and overrides nothing — it adds project-specific rules.

## Project elevator pitch

reposix exposes REST-based issue trackers (and similar SaaS systems) as a standard git repo via a git remote helper and partial clone. Agents use `cat`, `grep`, `sed`, and `git` on real workflows — no MCP tool schemas, no custom CLI. See `docs/research/initial-report.md` for the architectural argument and `docs/research/agentic-engineering-reference.md` for the dark-factory pattern that motivates the simulator-first approach.

## Architecture transition (v0.9.0 — in progress)

> **Read `.planning/research/v0.9-fuse-to-git-native/architecture-pivot-summary.md` for the full design.**

reposix is migrating from a FUSE virtual filesystem to git-native partial clone. Key changes:

- **DELETE `crates/reposix-fuse`** — no more FUSE daemon, fusermount3, or /dev/fuse dependency.
- **`git-remote-reposix` becomes a promisor remote** via `stateless-connect` capability — blobs are lazy-fetched on demand.
- **Push uses the existing `export` capability** — hybrid confirmed working in POC.
- **Agent UX is pure git** — `git clone`, `cat`, `git push`. Zero reposix CLI awareness.
- **Push-time conflict detection** — helper checks backend state at push time, rejects with standard git errors.
- **Blob limit guardrail** — helper refuses to serve >N blobs, error message teaches agent to narrow scope via sparse-checkout.

References below marked "(pre-v0.9.0)" describe the FUSE architecture being replaced. Code references to `reposix-fuse` will be deleted during v0.9.0 phases.

## Operating Principles (project-specific)

The user's global Operating Principles in `~/.claude/CLAUDE.md` are bible. The following are project-specific reinforcements, not replacements:

1. **Simulator is the default / testing backend.** The simulator at `crates/reposix-sim/` is the default backend for every demo, unit test, and autonomous agent loop. Real backends (GitHub via `reposix-github`, Confluence via `reposix-confluence`) are guarded by the `REPOSIX_ALLOWED_ORIGINS` egress allowlist and require explicit credential env vars (`GITHUB_TOKEN`, `ATLASSIAN_API_KEY` + `ATLASSIAN_EMAIL` + `REPOSIX_CONFLUENCE_TENANT`). Autonomous mode never hits a real backend unless the user has put real creds in `.env` AND set a non-default allowlist. This is both a security constraint (fail-closed by default) and the StrongDM dark-factory pattern.
2. **Tainted by default.** Any byte that came from a remote (simulator counts) is tainted. Tainted content must not be routed into actions with side effects on other systems (e.g. don't echo issue bodies into `git push` to remotes outside an explicit allowlist). The lethal-trifecta mitigation matters even against the simulator, because the simulator is *seeded* by an agent and seed data is itself attacker-influenced.
3. **Audit log is non-optional.** Every network-touching action gets a row in the simulator's SQLite audit table. If a feature can't write to the audit log, it's not done.
4. **No hidden state.** Mount state, simulator state, and git remote helper state all live in committed-or-fixture artifacts. No "it works in my session" bugs.
5. **Working tree = git repo.** The working tree must always be a real git checkout. The whole point of the design is `git diff` is the change set. (Pre-v0.9.0 this was a FUSE mount; post-v0.9.0 it's a partial-clone git repo.)
6. **Real backends are first-class test targets.** Three canonical targets are sanctioned for aggressive testing: **Confluence space "TokenWorld"** (owned by the user; safe to mutate freely), **GitHub repo `reubenjohn/reposix` issues** (ours; safe to create/close issues during tests), and **JIRA project `TEST`** (default key; overridable via `JIRA_TEST_PROJECT` or `REPOSIX_JIRA_PROJECT`). See `docs/reference/testing-targets.md` (created in Phase 36) for env-var setup. Simulator remains the default (OP-1), but "simulator-only coverage" does NOT satisfy acceptance for transport-layer or performance claims.

## Workspace layout

```
crates/
├── reposix-core/    # Shared types: Issue, Project, RemoteSpec, Error.
├── reposix-sim/     # In-process axum HTTP simulator.
├── reposix-fuse/    # FUSE daemon (BEING DELETED in v0.9.0 — see architecture transition above).
├── reposix-remote/  # git-remote-reposix binary.
└── reposix-cli/     # Top-level `reposix` CLI (orchestrator).

.planning/           # GSD project state. Do not hand-edit; use /gsd-* commands.
research/            # Long-form research notes + red-team reports.
docs/                # User-facing docs.
runtime/             # gitignored — local sim DB, mount points.
```

## Tech stack

- Rust stable (1.82+ via `rust-toolchain.toml`).
- Async: `tokio` 1.
- Web: `axum` 0.7 + `reqwest` 0.12 (rustls only, never openssl-sys).
- FUSE: `fuser` 0.17 with `default-features = false`. **(Pre-v0.9.0 — being deleted. See architecture transition above.)**
- Storage: `rusqlite` 0.32 with `bundled` feature (no system libsqlite3).
- Errors: `thiserror` for typed crate errors, `anyhow` only at binary boundaries.

## Commands you'll actually use

```bash
# Local dev loop
cargo check --workspace                                   # fast type check
cargo test --workspace                                    # unit tests
cargo clippy --workspace --all-targets -- -D warnings     # CI lint
cargo fmt --all                                           # CI fmt

# Run the stack
cargo run -p reposix-sim                                  # start simulator on :7777
cargo run -p reposix-fuse -- /tmp/reposix-mnt             # mount (when phase 3 lands)
cargo run -p reposix-cli -- demo                          # canonical end-to-end demo

# FUSE integration tests — require fusermount3; NEVER run without the feature flag.
# `cargo test --workspace` intentionally excludes these (unsafe in WSL2, requires /dev/fuse).
# The feature gate is compile-time: without it, FUSE test code is not in the binary at all.
# NOTE: This entire block is removed in v0.9.0 Phase 36 — `crates/reposix-fuse/` is deleted there.
cargo test -p reposix-fuse --release --features fuse-mount-tests -- --test-threads=1

# Testing against real backends (v0.9.0+)
# Confluence — TokenWorld space (safe to mutate)
export ATLASSIAN_API_KEY=… ATLASSIAN_EMAIL=… REPOSIX_CONFLUENCE_TENANT=reuben-john
export REPOSIX_ALLOWED_ORIGINS='https://reuben-john.atlassian.net'
cargo test -p reposix-confluence --features live -- --ignored

# GitHub — reubenjohn/reposix issues (safe to mutate)
export GITHUB_TOKEN=…
export REPOSIX_ALLOWED_ORIGINS='https://api.github.com'
cargo test -p reposix-github --features live -- --ignored

# JIRA — default project key TEST (overridable)
export JIRA_EMAIL=… JIRA_API_TOKEN=… REPOSIX_JIRA_INSTANCE=…
export JIRA_TEST_PROJECT=TEST        # or set REPOSIX_JIRA_PROJECT
cargo test -p reposix-jira --features live -- --ignored
```

## GSD workflow

This project uses GSD (`get-shit-done`) for planning and execution. Workflow rule:

> **Always enter through a GSD command.** Never edit code outside a GSD-tracked phase or quick.

Entry points:

- `/gsd-quick` — small fix or doc tweak.
- `/gsd-execute-phase <n>` — run a planned phase end-to-end with subagents.
- `/gsd-debug` — investigate a bug.
- `/gsd-progress` — what's the state of the project right now.

The auto-mode bootstrap from 2026-04-13 set `mode: yolo`, `granularity: coarse`, and enabled all workflow gates (research / plan_check / verifier / nyquist / code_review). Do not silently downgrade these.

## Coding conventions

- `#![forbid(unsafe_code)]` in every crate. The `fuser` callbacks themselves are safe Rust.
- `#![warn(clippy::pedantic)]` in every crate. Allow-list specific lints with rationale; never blanket-allow `pedantic`.
- All public items documented; missing-doc lint is on for `reposix-core`.
- All `Result`-returning functions have a `# Errors` doc section.
- Tests live next to the code (`#[cfg(test)] mod tests`). Integration tests in `tests/`.
- Frontmatter uses `serde_yaml` 0.9 + Markdown body. Never JSON-on-disk for issues.
- Times are `chrono::DateTime<Utc>`. No `SystemTime` in serialized form.

## Subagent delegation rules

Per the user's global OP #2: "Aggressive subagent delegation." Specifics for this project:

- `gsd-phase-researcher` for any "how do I implement X" question that would consume >100 lines of orchestrator context.
- `gsd-planner` for phase planning. Do not write `PLAN.md` by hand.
- `gsd-executor` for phase execution unless the work is trivially small.
- `gsd-code-reviewer` after every phase ships, before declaring done.
- Run multiple subagents in parallel whenever they're operating on disjoint files.

The orchestrator's job is to route, decide, and integrate — not to type code that a subagent could type.

## Threat model

This project is a textbook lethal-trifecta machine:

| Leg of trifecta | Where it shows up here |
| --- | --- |
| Private data | Mounted FUSE exposes issue bodies, internal field values, attachments. |
| Untrusted input | Every issue body / comment / title is attacker-influenced text. |
| Exfiltration | `git push` can target arbitrary remotes; the FUSE daemon makes outbound HTTP. |

Cuts that are mandatory and tested:

- **Outbound HTTP allowlist.** The FUSE daemon and remote helper refuse to talk to any origin not in `REPOSIX_ALLOWED_ORIGINS` (env var, defaults to `http://127.0.0.1:*` only).
- **No shell escape from FUSE writes.** Writes are bytes-in-bytes-out; no rendering, no template expansion.
- **Frontmatter field allowlist.** Server-controlled fields (`id`, `created_at`, `version`) cannot be overridden by client writes; they are stripped on the inbound path before serialization.
- **Audit log is append-only.** SQLite WAL, no UPDATE/DELETE on the audit table.

See `research/threat-model-and-critique.md` (produced by red-team subagent) for the full analysis.

## What to do when context fills

If you (the agent) notice this CLAUDE.md getting hard to keep in working memory:

1. Read `.planning/STATE.md` first — it's the entry point.
2. Read the most recent `.planning/phases/*/PLAN.md`.
3. Skim `git log --oneline -20` to know what's recently shipped.
4. Don't read this file linearly; grep for the section you need.

## Quick links

- `docs/research/initial-report.md` — full architectural argument for FUSE + git-remote-helper.
- `docs/research/agentic-engineering-reference.md` — dark-factory pattern, lethal trifecta, simulator-first.
- `.planning/PROJECT.md` — current scope.
- `.planning/STATE.md` — current cursor.
