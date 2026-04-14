# Roadmap (v0.4+)

## What shipped

| Release | Highlights |
|---|---|
| **v0.1** | Simulator + `IssueBackend` trait + FUSE read-only mount + `git-remote-reposix` + 8 security guardrails. 17 Active Requirements delivered; 133 tests green. |
| **v0.2** | Real GitHub Issues adapter behind the same trait. |
| **v0.3** | Real Confluence Cloud adapter (live against `reuben-john.atlassian.net`); swarm harness; prebuilt release binaries. |
| **v0.4** | `pages/` + `tree/` nested mount layout with symlink overlay exposing Confluence's `parentId` hierarchy — the "hero image" folder structure. 264 tests green. |

## Current open problems

The canonical list of next-session work lives in [`HANDOFF.md`](https://github.com/reubenjohn/reposix/blob/main/HANDOFF.md) as "OP-1 … OP-N" open problems. Highlights still outstanding after v0.4:

- **OP-2** — dynamically generated `INDEX.md` per directory (composes with `tree/`).
- **OP-3** — cache refresh via `git pull` semantics; the mount becomes a time machine over the backend.
- **OP-7** — hardening probes (concurrent-write contention swarm, FUSE under real-backend load, 500-page pagination cap, tenant-name leakage in tracing, chaos audit-log restart).
- **OP-8** — honest-tokenizer benchmarks using Anthropic's `count_tokens` API.
- **OP-9** — Confluence beyond pages (whiteboards, comments at `pages/<id>.comments/`, attachments, live docs, folders, multi-space).
- **OP-10** — eject 3rd-party adapter crates out of the core repo (user-gated).
- **OP-11** — repo-root reorg (user-gated).

### What shipped from the original v0.2 priority list

The v0.1-retrospective priority list that used to occupy this section has substantially shipped:

- ✅ **Real-backend adapter for GitHub Issues** — v0.2.
- ✅ **Adversarial swarm harness** — Phase 9 (`reposix-swarm`).
- ✅ **CRLF + non-UTF-8 in `git-remote-reposix`** — fixed in v0.2.
- ✅ **Protocol error-line emission** — fixed in v0.2.
- ✅ **IPv6 allowlist parser** — covered in v0.1 follow-ups.
- ✅ **Real Confluence adapter** — v0.3.
- ✅ **Nested mount layout with `tree/` overlay** — v0.4.

Items still outstanding (and now rolled into `OP-7` hardening unless otherwise noted): `X-Reposix-Agent` HMAC signing, `DashMap` LRU cap, FUSE SIGTERM handler, audit-log PII redaction, deterministic blob bytes, FUSE `create()` id divergence, marks-based incremental import, FUSE attribute cache invalidation, conflict-aware git merging (north-star UX), simulator `/_audit` dashboard, per-phase CI timing + coverage trend, token-economy benchmark with the real `count_tokens` API (OP-8).

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
