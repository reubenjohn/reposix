# reposix

> **Status (2026-07-14):** v0.13.0 SHIPPED + tagged 2026-07-07 (`3423b18`); v0.13.1 hotfix
> SHIPPED 2026-07-08 (`04640d5`); v0.14.0 SHIPPED publicly (current GitHub "Latest" release,
> tag exists). **Arc D RATIFIED** at commit `6aa734a` under owner delegation (canonical
> record: the ADDENDUM in `.planning/milestones/audits/2026-07-12-reality-check.md`) —
> pipeline pause LIFTED, normal GSD gates apply. **Next / active milestone: v0.15.0
> "Floor"** (the launch-readiness floor) — this re-anchor starts it.

## What This Is

A git-native partial clone that exposes REST-backed systems of record (issue trackers, wikis, project trackers) as a bare git repo, so autonomous LLM agents can use `cat`, `grep`, `sed`, and `git` instead of MCP tool schemas. Targets the "dark factory" agent-engineering pattern: when no human reads the code, agents need substrates that match their pre-training distribution.

## Core Value

**An LLM agent can `ls`, `cat`, `grep`, edit, and `git push` issues in a remote tracker without ever seeing a JSON schema or REST endpoint.** Everything else (multi-backend support, simulators, RBAC, conflict resolution) is in service of that single experience.

## Requirements

### Validated

> Full validated-requirements log lives in `.planning/PROJECT-history.md` (RENAME-01, EXT-01, JIRA-01..06, ARCH-01..19, DOCS-01..11) plus the historical v0.1.0 MVD list. Per-milestone phase plans live in `.planning/milestones/v*-phases/`.

### Active

> v0.15.0 Floor requirements are being scoped via `/gsd-new-milestone` (this document is
> re-anchored in Steps 4–6 of that flow; formal REQUIREMENTS.md routing is a later step).
> Interim lane scaffold: `.planning/milestones/v0.15.0-phases/ROADMAP.md` (UX
> error-message audit phase already drafted) + `.planning/milestones/v0.15.0-phases/
> {GOOD-TO-HAVES,SURPRISES-INTAKE}.md` (pre-seeded intake rows to drain per OP-8). The
> cross-milestone index `.planning/REQUIREMENTS.md` is due for its own refresh — as of
> 2026-07-14 it still lists v0.14.0 as "Active milestone" and v0.13.0 with a stale
> shipped-date/phase-range (see Noticing).
>
> v0.12.0's requirements (RELEASE-*, QG-*, STRUCT-*, DOCS-REPRO-*, DOCS-BUILD-*, SUBJ-*,
> ORG-*, MIGRATE-*) SHIPPED 2026-04-28 — full detail in `.planning/PROJECT-history.md`.

### Out of Scope

- **Real Jira/GitHub/Confluence credentials in v0.1** — Simulator first. Real backends bolt on once the substrate is proven. Avoids credential exposure during overnight autonomous build. *(JIRA credentials now planned for v0.8.0 — see JIRA-01…JIRA-06 above.)*
- **Windows / macOS support in v0.1** — FUSE on Linux only. macOS via macFUSE is a follow-up; Windows needs a different VFS layer entirely.
- **Web UI** — agents don't need it; humans use the CLI + the underlying git repo.
- **Multi-tenant hosted service** — local-first only. The whole point is the agent talks to the local FS.
- **Pickle/binary serialization** — JSON + YAML only. Per simon-willison-style auditability.
- **Eager full sync of remote state** — lazy, on-demand fetches with caching. A naïve `grep -r` must not melt API quotas (per `docs/research/initial-report.md` §rate-limiting).

## Context

- **Why this exists.** From `docs/research/agentic-engineering-reference.md`: MCP burns 100k+ tokens on schema discovery before the first useful operation. POSIX is in the model's pre-training. A git-native read — `cat issues/<id>.md` inside the checkout — measured **~94% fewer output tokens** (1,213 vs 21,171) and **~75% lower cost per session** ($0.21 vs $0.83) than the equivalent GitHub-MCP read, with **~56% smaller total input-context** (244,556 vs 550,219 tokens) as the direct analog to the old "tokens of context" claim — P115's live-session benchmark against a real GitHub backend, offline-reproducible (`docs/benchmarks/token-economy.md`).
- **Reference materials.** `docs/research/initial-report.md` (architecture deep-dive on FUSE + git-remote-helper for agentic tooling) and `docs/research/agentic-engineering-reference.md` (Simon Willison interview distillation: dark factory pattern, lethal trifecta, simulator-first).
- **Inspiration projects.** `~/workspace/token_world` (Python, knowledge-graph as ground truth, CI discipline). `~/workspace/theact` (small-model RPG engine, observability tooling). `~/workspace/reeve_bot` (production Telegram bot stack).
- **Threat model.** This project is a textbook lethal trifecta: private remote data + untrusted ticket text + git-push exfiltration. Mitigations are first-class: tainted-content marking, audit log, no auto-push to unauthorized remotes, RBAC → POSIX permission translation.

## Constraints

- **Tech stack**: Rust 1.82+, Cargo workspace. Async via Tokio, HTTP via axum/reqwest (rustls-only — no openssl-sys), git via gix 0.82 (pinned `=` because pre-1.0). Runtime requires `git >= 2.34` for `extensions.partialClone` + `stateless-connect`.
- **Compatibility**: Linux + macOS + Windows after v0.11.0 dist binaries land; CI runs on `ubuntu-latest`. Pre-v0.11.0: Linux-only releases.
- **Security**: Autonomous mode never hits a real backend unless the user has put real creds in `.env` AND set a non-default `REPOSIX_ALLOWED_ORIGINS`. Simulator is the default for every demo / unit test / autonomous loop. Real-backend testing is sanctioned against three canonical targets: TokenWorld (Confluence), `reubenjohn/reposix` (GitHub), JIRA project `TEST` — see `docs/reference/testing-targets.md`.
- **Dependencies**: Only crates that compile without `pkg-config` or system dev headers. `rusqlite` with `bundled`. `reqwest` with `rustls-tls`.
- **Ground truth**: All state in committed artifacts. Cache state, simulator state, and helper state all live in committed-or-fixture artifacts. No "it works in my session" bugs.
- **Egress safety**: The single `reposix_core::http::client()` factory is the only legal way to construct an HTTP client in this workspace. Direct `reqwest::Client::new()` calls are denied by clippy lint (`clippy::disallowed_methods`). Every HTTP request honors `REPOSIX_ALLOWED_ORIGINS`.
- **Release gates**: Tag pushes are owner gates. Any docs-site work must be playwright-validated (POLISH-17 wires this into the pre-push hook).

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Rust over Python | Production substrate; FUSE perf matters under swarm load; user explicitly chose Rust | Validated |
| Simulator-first, not real backend | StrongDM pattern: real APIs rate-limit a swarm to death; simulator is free to hammer; also avoids credential risk in autonomous mode | Validated |
| `fuser` crate, `default-features = false` | No libfuse-dev / pkg-config available; runtime uses fusermount binary which is present | Validated |
| `rusqlite` with `bundled` | Avoids needing libsqlite3-dev | Validated |
| Workspace with 5 crates (`-core`, `-sim`, `-fuse`, `-remote`, `-cli`) | Clear separation of concerns; each crate independently testable; `-core` isolates types from binaries | Validated |
| Issues as `.md` + YAML frontmatter | Matches `docs/research/initial-report.md` §"Modeling Hierarchical Directory Structures"; agents already understand the format | Validated |
| `git-remote-helper` protocol over custom sync | Leverages git's conflict resolution (OP: ground truth, simon willison §5.2 lethal trifecta argues git semantics > JSON conflict synthesis) | Validated |
| Public GitHub repo `reubenjohn/reposix` | User authorized; CI must run; demo must be shareable | Validated |
| Auto/YOLO mode, coarse granularity, all workflow gates on | User asked for max autonomy + GSD discipline; coarse phases fit 7-hour window | Validated |
| Skip GSD discuss step | User instruction (~12:55 AM): "do all the gsd planning, exec, review, etc, just without the discuss steps" | Validated |
| Lethal-trifecta cuts are first-class requirements, not afterthoughts | Threat-model subagent flagged egress + bulk-delete + tainted typing as ship-blockers; safer to bake in than retrofit | Validated |

## Current Milestone: v0.15.0 Floor

**Goal:** Establish the launch-readiness floor — fix the t4 Confluence oid-drift product defect, purge doc-truth launch-blockers, harden every user-facing error to Rust-compiler-grade, re-measure live benchmarks, and simplify planning/docs — closing the known correctness and honesty gaps before the journey-slice milestones.

**Target features (lanes):**
- t4 Confluence oid-drift fix-first (`crates/reposix-cache/src/builder.rs` `read_blob`) + `sync --reconcile` oid-drift audit
- Doc-truth launch-blocker purge (6 blockers) + docs/planning simplification (delete legacy outright; git history is the archive)
- User-facing error hardening to Rust-compiler-grade (folds in the prior narrow v0.15 UX-error-message goal as one lane)
- Live MCP benchmark re-measurement (hero-number doc-alignment waivers expire 2026-08-15 — schedule EARLY)
- ADR-010 mirror-fanout decision packet (produce options+tradeoffs for owner/manager ruling; do NOT implement pre-ruling)
- Intake / good-to-have drain (OP-8): SURPRISES-INTAKE items B/C/E/F + the GTH-V15-* rows

**Context this milestone starts from.** v0.15.0 is the first PLANNED milestone of ratified
**Arc D (ratchet-first)** — see the ADDENDUM in
`.planning/milestones/audits/2026-07-12-reality-check.md` for the full owner-decision
record (Q3 real-backend launch gate, Q4 subjective-runner fix funded, Q5/Q7 aggressive
docs/planning simplification mandate, Q9 six-planned/six-stub scale). Arc D's shape:
v0.15 floor (this milestone) → v0.17 meta-milestone (the five gate shapes that would have
caught the reality-check findings) → v0.19 truth purge + IA rebuild → v0.21 benchmark
honesty → v0.23 journey slices → v0.25 launch kit + Show-HN, with stub milestones
interleaved (v0.16, v0.18, …) draining surprises/good-to-haves as they surface.
Interim lane scaffold already seeded: `.planning/milestones/v0.15.0-phases/{ROADMAP,
GOOD-TO-HAVES,SURPRISES-INTAKE}.md`. Formal phase numbering + REQUIREMENTS.md routing
happen in the next steps of this `/gsd-new-milestone` run, not in this re-anchor.

**Public roadmap.** The bird's-eye capability map for this arc — shipped → Floor → future
arcs converging on the OD-4 launch — is published at [`../docs/roadmap.md`](../docs/roadmap.md) <!-- SYNC: paired with docs/roadmap.md § Roadmap. Edit either side → update the other; re-color the arcs (shipped / active / future) at milestone close. -->

---

## Previously Validated Milestones

Full per-milestone narrative (goal, target features, framing principles, carry-forward) lives in `.planning/PROJECT-history.md`. Cross-milestone retrospective lessons live in `.planning/RETROSPECTIVE.md`. Per-milestone phase artifacts live under `.planning/milestones/v*-phases/`.

| Milestone | Status | Theme | Phases |
|---|---|---|---|
| v0.14.0 | SHIPPED + Latest 2026-07-14 | Wave-2 hardening (D2 dark-factory regression, RBF-LR-03 rebase recovery, OD-4 launch-readiness scope stub) | P102–P112 + P113 |
| v0.13.1 | SHIPPED 2026-07-08 | Hotfix release | — |
| v0.13.0 | SHIPPED 2026-07-07 (`3423b18`) | DVCS over REST (extended) — `attach`, bus remote, mirror-lag refs, webhook mirror sync | P78–P97 |
| v0.12.x | SHIPPED 2026-04-30 | Quality Gates + carry-forward drain | 56–65, 72–77 |
| v0.12.0 | SHIPPED 2026-04-28 | Quality Gates framework (dimension / cadence / kind) | 56–63 |
| v0.11.x | SHIPPED 2026-04-27 | Polish & Reproducibility (mkdocs site live; §0.8 SESSION-END-STATE framework) | 50–55 + POLISH2-* |
| v0.10.0 | SHIPPED 2026-04-25 | Docs & Narrative Shine (Diátaxis IA, banned-words linter, cold-reader audit) | 40–45 |
| v0.9.0 | SHIPPED 2026-04-24 | Architecture Pivot — git-native partial clone (FUSE deleted) | 31–36 |
| v0.1.0..v0.8.0 | SHIPPED earlier | MVD + JIRA write path + connector matrix; see history file | see archives |

v0.13.2 (Cross-link fidelity, 10th quality-gate dimension) is **QUEUED, not shipped** —
scaffolded at P98–P107 (placeholder numbering, pending renumber-on-insertion) but zero
phases executed as of 2026-07-14; resequenced behind v0.14.0 and the launch-readiness arc
per OD-4 (see STATE.md `workstreams.workstream_b`).

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `/gsd-transition`):
1. Requirements invalidated? → Move to Out of Scope with reason
2. Requirements validated? → Move to Validated with phase reference
3. New requirements emerged? → Add to Active
4. Decisions to log? → Add to Key Decisions
5. "What This Is" still accurate? → Update if drifted

**After each milestone** (via `/gsd-complete-milestone`):
1. Full review of all sections
2. Core Value check — still the right priority?
3. Audit Out of Scope — reasons still valid?
4. Update Context with current state

---
*Last updated: 2026-07-14 — `/gsd-new-milestone` re-anchor: v0.13.0/v0.13.1/v0.14.0 moved
to Previously Validated (all SHIPPED); v0.13.2 noted QUEUED not shipped; Arc D ratified
(`6aa734a`) and v0.15.0 "Floor" opened as Current Milestone. Historical milestone
narrative lives in `.planning/PROJECT-history.md` to keep this file under the
file-size-limits budget (one logical change per session).*
