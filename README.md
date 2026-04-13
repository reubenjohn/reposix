# reposix

> A git-backed FUSE filesystem that exposes REST APIs as POSIX directories so autonomous LLM agents can use `cat`, `grep`, `sed`, and `git` instead of MCP tool schemas.

[![CI](https://github.com/reubenjohn/reposix/actions/workflows/ci.yml/badge.svg)](https://github.com/reubenjohn/reposix/actions/workflows/ci.yml)
[![Docs](https://github.com/reubenjohn/reposix/actions/workflows/docs.yml/badge.svg)](https://reubenjohn.github.io/reposix/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE-MIT)
[![Rust](https://img.shields.io/badge/rust-stable-orange.svg)](rust-toolchain.toml)

:book: **Full docs with architecture diagrams:** <https://reubenjohn.github.io/reposix/>

## Why

Modern coding agents have ingested vast amounts of Unix shell scripting and `git` workflows during pre-training. Asking them to use a `cat` + `git commit` workflow is asking them to do what they already know how to do. Asking them to use Model Context Protocol (MCP) is asking them to load 100k+ tokens of JSON schemas before doing anything useful.

reposix takes the second problem and reduces it to the first. A Jira board, a GitHub Issues repo, or a Confluence space becomes a directory of Markdown files with YAML frontmatter, with native `git push` synchronization and merge-conflict-as-API-conflict semantics.

See [`InitialReport.md`](InitialReport.md) for the full architectural argument and [`benchmarks/RESULTS.md`](benchmarks/RESULTS.md) for the measured **92.3% reduction** in input-context tokens (reposix vs MCP for the same task).

## Status

**v0.1 alpha.** Built autonomously overnight on 2026-04-13 as an experiment in whether a single coding agent can ship a complete Rust substrate in ~7 hours. Treat as alpha per Simon Willison's "proof of usage, not proof of concept" rule. ~133 workspace tests pass; `cargo clippy --workspace --all-targets -- -D warnings` is clean.

| Phase                                   | Outcome                                                                                                  |
|-----------------------------------------|----------------------------------------------------------------------------------------------------------|
| Phase 1 — Core contracts + guardrails   | shipped: `http::client()` factory + allowlist, `Tainted<T>`/`sanitize`, audit-log triggers, path validator |
| Phase 2 — Simulator + audit log         | shipped: axum sim with rate limit + 409 + RBAC, append-only SQLite audit                                |
| Phase 3 — FUSE read path + CLI          | shipped: getattr/readdir/read/write/create/unlink, 5s timeout watchdog, `reposix sim/mount/demo`         |
| Phase S — Write path + git-remote-reposix | shipped: full FUSE write, `git-remote-reposix` PATCH/POST/DELETE, SG-02 bulk-delete cap                 |
| Phase 4 — Demo + docs                   | shipped: `scripts/demo.sh` + recorded `script(1)` typescript + walkthrough + README polish               |

Tracking artifacts live in [`.planning/`](.planning/).

## Demo

End-to-end recording: [`docs/demo.md`](docs/demo.md) (walkthrough),
[`docs/demo.typescript`](docs/demo.typescript) (raw `script(1)`),
[`docs/demo.transcript.txt`](docs/demo.transcript.txt) (ANSI-stripped).

The recording captures the full 9-step narrative — sim startup, FUSE mount, agent-style `ls`/`cat`/`grep`, FUSE write through, `git push` round-trip — and three guardrails firing **on camera**:

1. **Outbound HTTP allowlist refusal (SG-01).** A second mount with `REPOSIX_ALLOWED_ORIGINS` mismatched against the configured backend; every fetch refuses, surfaces as `Permission denied` on `ls`.
2. **Bulk-delete cap (SG-02).** `git rm` of 6 issues + push is refused; commit message tag `[allow-bulk-delete]` overrides.
3. **Server-controlled-frontmatter strip (SG-03).** A client write whose body contains `version: 999` does not update the server's authoritative version — `Tainted<T> → sanitize()` strips server-controlled fields before egress.

## Quickstart

Prereqs (Linux only for v0.1):

- Rust stable 1.82+ (tested with 1.94.1).
- `fusermount3` (Ubuntu: `sudo apt install fuse3`).
- `jq`, `sqlite3`, `curl`, `git` (>= 2.20) on `$PATH`.

Then:

```bash
git clone https://github.com/reubenjohn/reposix
cd reposix
bash scripts/demo.sh
```

For the per-step explanation see [`docs/demo.md#walkthrough`](docs/demo.md#walkthrough).

## Architecture

```
┌──────────┐   git    ┌──────────────────┐  HTTP   ┌──────────────────┐
│  agent   │ ───────▶ │ git-remote-      │ ──────▶ │ reposix-sim      │
│ (shell)  │          │ reposix          │         │ (or real Jira)   │
└──────────┘          └──────────────────┘         └──────────────────┘
     │                         │                            ▲
     │ POSIX                   │ tokio                      │
     ▼                         │                            │
┌──────────┐                   │                            │
│ FUSE     │ ──────────────────┴────────────────────────────┘
│ mount    │   reqwest (HTTP allowlist enforced)
└──────────┘
     ▲
     │ fusermount3
     ▼
┌──────────┐
│ kernel   │
│  VFS     │
└──────────┘
```

## Security

reposix is a textbook **lethal trifecta** (Simon Willison's framing): private remote data + untrusted ticket text + `git push` exfiltration. The full red-team gap analysis is in [`.planning/research/threat-model-and-critique.md`](.planning/research/threat-model-and-critique.md). The mitigations below are the v0.1 commitments — every one has a test or a clippy lint that asserts it.

### Threat model — what's enforced in v0.1

| ID    | Mitigation                                                       | Enforcement                                                                                              |
|-------|------------------------------------------------------------------|----------------------------------------------------------------------------------------------------------|
| SG-01 | Outbound HTTP allowlist (`REPOSIX_ALLOWED_ORIGINS`)              | Single `reposix_core::http::client()` factory + clippy `disallowed-methods` lint on `reqwest::Client::new` |
| SG-02 | Bulk-delete cap (push deleting > 5 issues refused)               | `git-remote-reposix` `diff::plan` returns `BulkDeleteRefused`; integration tests with 5 vs 6 deletes      |
| SG-03 | Server-controlled frontmatter immutable from clients             | `Tainted<T> → sanitize()` strips `id`/`version`/`created_at`/`updated_at` on every PATCH/POST egress     |
| SG-04 | Filename = `<id>.md`; path validation rejects `/`, `\0`, `..`    | `validate_issue_filename` invoked at every FUSE path-bearing op                                          |
| SG-05 | Tainted-content typing                                           | `Tainted<T>`/`Untainted<T>` newtype pair; `trybuild` compile-fail test for misuse                        |
| SG-06 | Audit log append-only                                            | SQLite `BEFORE UPDATE` and `BEFORE DELETE` triggers on `audit_events`; pragma test asserts they exist     |
| SG-07 | FUSE never blocks the kernel forever                             | All upstream HTTP via `with_timeout(5s)` wrapper; on timeout returns EIO                                 |
| SG-08 | Demo recording shows guardrails firing                           | `docs/demo.typescript` contains SG-02 refusal + allowlist refusal markers; verified by grep              |

### Deferred to v0.2

- **M-* findings from the red-team report.** Several medium-severity findings in the threat-model document are deferred — for example, fully sandboxed `git-remote-reposix` execution (currently runs as the invoking user with full FS access), and TTY-confirmation on `git remote add reposix::...`.
- **Real-backend credentials.** v0.1 does **not** authenticate to any real backend. Simulator-only. Real Jira/GitHub/Confluence integration ships with explicit user opt-in, allowlist scoping per origin, and credential isolation in v0.2.
- **Signed recording attestation.** `script(1)` timestamps are trusted-by-invocation. We do not claim cryptographic provenance on `docs/demo.typescript`.
- **Workflow rule enforcement.** v0.1's transitions endpoint reports all 5 statuses as legal from any state. v0.2 will model real workflow constraints (e.g. "must pass through `in_progress` before `done`").
- **Swarm harness + FUSE-in-CI.** Stretch items listed in PROJECT.md but cut from v0.1 to keep the build window honest.

v0.1 does **not** authenticate to any real backend. Simulator-only. Treat this codebase as alpha.

## Honest scope

This project is the output of ~7 hours of autonomous coding-agent work on the night of 2026-04-13 — single agent, GSD planning workflow, no human in the loop after kickoff. SG-01 through SG-08 are mechanically enforced by tests + lints, but it's still alpha — only run it against the in-process simulator, don't hand it credentials to anything you care about, and read [`threat-model-and-critique.md`](.planning/research/threat-model-and-critique.md) end-to-end before considering v0.2.

Proof of usage, not proof of concept.

## License

Dual-licensed under MIT or Apache-2.0, at your option.
