# Roadmap: reposix

## Overview

Seven-hour autonomous build of a git-backed FUSE filesystem that exposes a REST
issue tracker as a POSIX tree. Roadmap front-loads the simulator + shared core
types + security guardrails (the load-bearing Phase 1), then lets FUSE and the
CLI/audit layer build on that substrate in parallel, then wraps with a demo
that is required to show the guardrails firing. Write-path features
(`git-remote-reposix`, swarm harness, FUSE-in-CI) are STRETCH only — the hard
decision gate at T+3h (03:30 PDT) cuts them if the MVD isn't already green.

**Baseline already in tree (not re-planned):**
- 5-crate Cargo workspace (`reposix-core`, `-sim`, `-fuse`, `-remote`, `-cli`)
- `reposix-core` types: `Issue`, `IssueId`, `IssueStatus`, `ProjectSlug`,
  `Project`, `RemoteSpec`, `parse_remote_url`, `frontmatter::{render, parse,
  yaml_to_json_value}`, `Error` (7 unit tests passing)
- CI workflow at `.github/workflows/ci.yml` (fmt + clippy + test + integration
  + coverage jobs, green on paper)
- README, CLAUDE.md, LICENSE-MIT, first push to `reubenjohn/reposix`

**Granularity:** coarse (4 integer phases, 3 MVD + 1 stretch bucket)

**Requirement index:**

| Group | IDs | Source |
|-------|-----|--------|
| Functional core | FC-01 … FC-09 | PROJECT.md §Active → *Functional core* |
| Security guardrails | SG-01 … SG-08 | PROJECT.md §Active → *Security guardrails* |

Requirements come from PROJECT.md `### Active` directly (this project has no
separate REQUIREMENTS.md). Every `Active` bullet maps to exactly one phase
below.

## MVD vs STRETCH

- **MVD (Phases 1–3)** ≤4.5h wall clock with subagent parallelism. This is
  the credible minimum-viable demo the threat-model agent recommended:
  read-only mount + simulator GET + CLI + audit log + green CI + guardrails.
- **STRETCH (Phase S)** only runs if MVD lands ahead of schedule. The `git push`
  write-path, bulk-delete guard wired through a real push, swarm harness, and
  FUSE-in-CI all live here. At T+3h (03:30 PDT) the orchestrator MUST look at
  phase-1/2/3 status and commit to STRETCH or drop it.
- **Phase N (Demo, always runs)** is Phase 4. It demos whatever MVD shipped,
  and the guardrails it shipped. If STRETCH also shipped, Phase 4 extends the
  demo script to cover push + 409-merge-as-conflict and records those too.

## Parallelism map

Phase 1 is serial — it publishes the contracts the other phases link against.
After Phase 1 lands, Phase 2 (simulator + audit) and Phase 3 (FUSE + CLI) can
run in parallel on separate crates. Phase 4 (demo) is serial by nature — it
needs whatever shipped to demo against.

```
Phase 1 (serial, load-bearing)
   │
   ├──► Phase 2 (sim + audit)  ┐
   │                            ├──► Phase 4 (demo + README + recording)
   └──► Phase 3 (FUSE + CLI)   ┘
                    │
                    └──► [T+3h gate] ──► Phase S (STRETCH: push + swarm)
```

## Chapters

- [Early phases 1–S: phase list + phase details](./chapter-early-phases-1-s.md) — Phase 1 (core contracts), Phase 2 (simulator), Phase 3 (FUSE + CLI), Phase 4 (demo), Phase S (STRETCH write path + swarm).
- [Early phases: coverage, decision gates, progress, phases 8–15](./chapter-early-phases-coverage.md) — Requirements coverage table, T+3h/T+5h/T+7h decision gates, progress tracker, phases 8–15 (v0.2–v0.5 era).
- [Milestone v0.6.0 — Write Path + Full Sitemap](./chapter-v0.6.0.md) — Phases 16–20 (Confluence write path, swarm, OP-2 INDEX, labels, refresh) + Phase 26 (docs clarity overhaul).
- [Milestone v0.7.0 — Hardening + Confluence Expansion](./chapter-v0.7.0.md) — Phases 21–25 (hardening bundle, tokenizer benchmarks, comments, whiteboards, docs reorg).
- [Milestone v0.8.0 — JIRA Cloud Integration + Backlog](./chapter-v0.8.0.md) — Phases 27–29 (BackendConnector rename, JIRA read-only adapter, JIRA write path) + backlog.
