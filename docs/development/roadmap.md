# Roadmap (v0.7+)

## What shipped

| Release | Highlights |
|---|---|
| **v0.1** | Simulator + `IssueBackend` trait + FUSE read-only mount + `git-remote-reposix` + 8 security guardrails. 17 Active Requirements delivered; 133 tests green. |
| **v0.2** | Real GitHub Issues adapter behind the same trait. |
| **v0.3** | Real Confluence Cloud adapter (live against `reuben-john.atlassian.net`); swarm harness; prebuilt release binaries. |
| **v0.4** | `pages/` + `tree/` nested mount layout with symlink overlay exposing Confluence's `parentId` hierarchy — the "hero image" folder structure. 272 tests green. |
| **v0.5** | Synthesized read-only `_INDEX.md` sitemap in each FUSE bucket directory (`mount/<bucket>/_INDEX.md`). 277 tests green. |
| **v0.6** | Confluence write path (`create_record`/`update_record`/`delete_or_close`); ADF↔Markdown converter; `labels/` read-only overlay; tree-recursive + mount-root `_INDEX.md`; `reposix refresh` subcommand. 317 tests green. |
| **v0.7** | Contention/truncation/chaos hardening; honest token benchmarks (89.1% reduction, real tokenizer); Confluence comments (`pages/<id>.comments/`), attachments (`.attachments/`), and whiteboards (`whiteboards/`); docs reorg (`docs/research/`). 317+ tests green. |

## Current open problems

> **All OP-1 through OP-11 items are CLOSED as of v0.7.0.** See [`HANDOFF.md`](https://github.com/reubenjohn/reposix/blob/main/HANDOFF.md) for the closed-item table and history.

The current next direction is v0.8:

- **JIRA Cloud read-only adapter** — same `IssueBackend` trait seam as GitHub and Confluence. JIRA's REST API uses OAuth 2.0 (not Basic auth); adapter named `reposix-jira`.
- **`BackendConnector` rename** — `IssueBackend` renamed to `BackendConnector` for accuracy (it connects any content store, not just issue trackers).
- **`Issue.extensions` field** — `serde_json::Value` map for backend-specific metadata not covered by the core schema (JIRA priority, Confluence space key, etc.).

### What shipped from the original v0.2 priority list

The v0.1-retrospective priority list has fully shipped:

- ✅ **Real-backend adapter for GitHub Issues** — v0.2.
- ✅ **Adversarial swarm harness** — Phase 9 (`reposix-swarm`).
- ✅ **CRLF + non-UTF-8 in `git-remote-reposix`** — fixed in v0.2.
- ✅ **Protocol error-line emission** — fixed in v0.2.
- ✅ **IPv6 allowlist parser** — covered in v0.1 follow-ups.
- ✅ **Real Confluence adapter** — v0.3.
- ✅ **Nested mount layout with `tree/` overlay** — v0.4.
- ✅ **Hardening probes (OP-7)** — contention/truncation/chaos harness shipped in v0.7.
- ✅ **Honest-tokenizer benchmarks (OP-8)** — `count_tokens` API integration shipped in v0.7.

Items deferred to v0.8+: `X-Reposix-Agent` HMAC signing, `DashMap` LRU cap, FUSE SIGTERM handler, audit-log PII redaction, conflict-aware git merging (north-star UX), simulator `/_audit` dashboard.

## Long-term north stars

- **macOS via macFUSE.** Requires a kernel extension + entitlement; architecturally similar to Linux FUSE. Re-evaluate when the Linux mount is battle-tested.
- **Windows via ProjFS / WinFsp.** A different abstraction; likely a parallel `reposix-projfs` crate instead of extending `reposix-fuse`.
- **A "real dark factory" deployment.** Simulated agents, a deliberately-broken real workflow, a large-scale exfil-surface test. This is the proof of the proof-of-usage.

## Known non-goals

- Web UI / dashboard as a primary user-facing surface. Agents don't need it; humans use the CLI + `git`.
- Support for a monolithic SaaS product (single hosted reposix). Local-first only.
- Picking a side between JSON-API-shaped backends and git-shaped ones. reposix is the impedance-matcher.

## How to extend

Start with `/gsd-add-phase` in the project root. The `.planning/ROADMAP.md` is the living scope — append there, then follow the plan → execute → review cycle. The [Contributing page](contributing.md) has the details.
