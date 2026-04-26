# Roadmap

> **Source of truth.** Active planning lives in [`.planning/ROADMAP.md`](https://github.com/reubenjohn/reposix/blob/main/.planning/ROADMAP.md), which is driven by the GSD workflow (`/gsd-add-phase`, `/gsd-plan-phase`, `/gsd-execute-phase`). This page is a public-facing snapshot — it lags the planning ledger by design and is refreshed at milestone close.

## Shipped milestones

| Release | Shipped | Summary |
|---|---|---|
| **v0.1.0 MVD** | 2026-04-13 | Simulator, `IssueBackend` trait, FUSE read-only mount, `git-remote-reposix`, eight security guardrails. |
| **v0.2.0-alpha** | 2026-04-13 | GitHub Issues read-only adapter behind the same trait. |
| **v0.3.0** | 2026-04-14 | Confluence Cloud read-only adapter (live against `reuben-john.atlassian.net`); swarm harness; prebuilt release binaries. |
| **v0.4.0** | 2026-04-14 | Nested mount layout (`pages/` + `tree/`) with symlink overlay exposing Confluence's `parentId` hierarchy. |
| **v0.5.0** | 2026-04-14 | `IssueBackend` decoupling from FUSE plus per-bucket `_INDEX.md` sitemap. |
| **v0.6.0** | 2026-04-14 | Confluence write path (`create_record` / `update_record` / `delete_or_close`); ADF↔Markdown converter; `labels/` overlay; `reposix refresh` subcommand. |
| **v0.7.0** | 2026-04-16 | Contention / truncation / chaos hardening; honest token benchmarks; Confluence comments / attachments / whiteboards; docs reorg. |
| **v0.8.0** | 2026-04-16 | JIRA Cloud integration; `IssueBackend` → `BackendConnector` rename; `Issue.extensions` field for backend-specific metadata. |
| **v0.9.0** | 2026-04-24 | Architecture pivot — git-native partial clone. FUSE mount retired; `git-remote-reposix` now advertises `stateless-connect` + `export` against an on-disk bare-repo cache. Agent UX is upstream git from `init` onward. |
| **v0.10.0** | 2026-04-25 | Docs and narrative shine — landing page, tutorial set, mermaid diagrams, value-prop framing. |

## Active milestone

> **v0.11.0 Polish & Reproducibility** — PLANNING (Phases 50–55 scaffolded). Closes the v0.10.0 long tail: jargon glosses + glossary, mermaid render hygiene, fresh-clone tutorial runner, dist release pipeline, real-backend latency table, vision-innovations surface (`reposix doctor`, `reposix log --time-travel`, `reposix gc --orphans`, `reposix cost`, `reposix init --since`).
>
> **v0.11.1 Polish & Reproducibility (second pass)** — PLANNING (POLISH2-01..22). Carry-forward: crates.io publish, arm64-musl, JIRA cells, code-quality P1, machine-readability gaps, upstream mkdocs-material issue.

See [`.planning/ROADMAP.md`](https://github.com/reubenjohn/reposix/blob/main/.planning/ROADMAP.md) for current phase status and the per-milestone archives under [`.planning/milestones/`](https://github.com/reubenjohn/reposix/tree/main/.planning/milestones).

## Long-term north stars

- **A "real dark factory" deployment.** Simulated agents, a deliberately-broken real workflow, a large-scale exfil-surface test — proof of the proof-of-usage.
- **Windows + macOS parity** for the partial-clone working tree (today's flow runs on any platform with `git >= 2.34`; benchmark + CI coverage is Linux-first).
- **More backends** behind the `BackendConnector` trait. Today: GitHub, Confluence, JIRA, simulator. Future candidates: Linear, Notion, ServiceNow.

## Known non-goals

- Web UI / dashboard as a primary user-facing surface. Agents do not need it; humans use the CLI alongside `git`.
- A monolithic SaaS product (single hosted reposix). Local-first only.
- Picking a side between JSON-API-shaped backends and git-shaped ones. reposix is the impedance-matcher for the 80% workflow.

## How to extend

Start with `/gsd-add-phase` in the project root. The [Contributing page](contributing.md) has the details. Append new scope to `.planning/ROADMAP.md`, then follow the plan → execute → review cycle.
