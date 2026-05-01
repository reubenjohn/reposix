# reposix

## What This Is

A git-backed FUSE filesystem that exposes REST APIs (issue trackers, knowledge bases, ticketing systems) as a POSIX directory tree, so autonomous LLM agents can use `cat`, `grep`, `sed`, and `git` instead of MCP tool schemas. Targets the "dark factory" agent-engineering pattern: when no human reads the code, agents need substrates that match their pre-training distribution.

## Core Value

**An LLM agent can `ls`, `cat`, `grep`, edit, and `git push` issues in a remote tracker without ever seeing a JSON schema or REST endpoint.** Everything else (multi-backend support, simulators, RBAC, conflict resolution) is in service of that single experience.

## Requirements

### Validated

> Full validated-requirements log lives in `.planning/PROJECT-history.md` (RENAME-01, EXT-01, JIRA-01..06, ARCH-01..19, DOCS-01..11) plus the historical v0.1.0 MVD list. Per-milestone phase plans live in `.planning/milestones/v*-phases/`.

### Active

> See `.planning/REQUIREMENTS.md` `## v0.12.0 Requirements` for the active list (RELEASE-*, QG-*, STRUCT-*, DOCS-REPRO-*, DOCS-BUILD-*, SUBJ-*, ORG-*, MIGRATE-*).
>
> v0.12.0 thesis: v0.11.x bolted on a §0.8 SESSION-END-STATE framework that catches the regression class IT was designed for — but missed the curl-installer URL going dark for two releases. v0.12.0 builds the **Quality Gates** system: dimension-tagged checks (code / docs-build / docs-repro / release / structure / perf / security / agent-ux), cadence-routed runners (pre-push / pre-pr / weekly / pre-release / post-release / on-demand), unified catalog schema, mandatory subagent verifier grading per phase, waivers with TTL, and a `quality/PROTOCOL.md` runtime contract for autonomous execution. Once the framework lands, every future quality-miss is one catalog row + one verifier — never another bespoke script.

### Out of Scope

- **Real Jira/GitHub/Confluence credentials in v0.1** — Simulator first. Real backends bolt on once the substrate is proven. Avoids credential exposure during overnight autonomous build. *(JIRA credentials now planned for v0.8.0 — see JIRA-01…JIRA-06 above.)*
- **Windows / macOS support in v0.1** — FUSE on Linux only. macOS via macFUSE is a follow-up; Windows needs a different VFS layer entirely.
- **Web UI** — agents don't need it; humans use the CLI + the underlying git repo.
- **Multi-tenant hosted service** — local-first only. The whole point is the agent talks to the local FS.
- **Pickle/binary serialization** — JSON + YAML only. Per simon-willison-style auditability.
- **Eager full sync of remote state** — lazy, on-demand fetches with caching. A naïve `grep -r` must not melt API quotas (per `docs/research/initial-report.md` §rate-limiting).

## Context

- **Why this exists.** From `docs/research/agentic-engineering-reference.md`: MCP burns 100k+ tokens on schema discovery before the first useful operation. POSIX is in the model's pre-training. A `cat /mnt/jira/PROJ-123.md` operation is ~2k tokens of context vs ~150k for the equivalent MCP-mediated read.
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

## Current Milestone: v0.13.0 — DVCS over REST

**Goal:** Shift the project thesis from "VCS over REST" (one developer, one backend) to "DVCS over REST" — the SoT (e.g. confluence) remains authoritative, but a plain-git mirror on GitHub becomes the universal-read surface for everyone else. Devs `git clone git@github.com:org/repo.git` with vanilla git (zero reposix install), get all markdown, edit, commit. Install reposix only when they want to write back; `reposix attach` reconciles their existing checkout against the SoT, then `git push` via a bus remote fans out atomically to confluence (SoT-first) and the GH mirror. The litmus test: a vanilla-git clone, attach, edit, push round-trip + a webhook-driven mirror catch-up after a browser-side confluence edit, with conflict detection in both directions.

**Mental model.** Three roles: SoT-holder (reposix-equipped, writes via bus); mirror-only consumer (vanilla git, read-only); round-tripper (reposix-equipped after `attach`, writes via bus). Bus remote: precheck-then-SoT-first-write — cheap network checks (`ls-remote` mirror, `list_changed_since` on SoT) bail before reading stdin; on success, REST-write to SoT then `git push` to mirror; mirror-write failure leaves "mirror lag" recoverable on next push, not data loss. Mirror-lag observability via plain-git refs (`refs/mirrors/confluence-head`, `refs/mirrors/confluence-synced-at`) — vanilla `git fetch` brings them along; `git log` shows staleness.

**Target features:**
- **`reposix attach <backend>::<project>`** — bootstrap a working tree NOT created by `reposix init`; reconcile cache OIDs against current HEAD by `id`-in-frontmatter; reject re-attach with different SoT (multi-SoT is v0.14.0); idempotent re-attach against same SoT.
- **Bus remote** — `reposix::<sot-spec>?mirror=<mirror-url>` URL scheme; precheck + SoT-first-write algorithm with `list_changed_since`-based conflict detection; PUSH-only (read goes to SoT directly).
- **Mirror-lag refs** — `refs/mirrors/confluence-head` (SoT SHA at last sync) + `refs/mirrors/confluence-synced-at` (annotated tag with timestamp); written by webhook sync AND bus push; bus-remote reject messages cite them in hints.
- **Webhook-driven mirror sync** — reference GitHub Action workflow at `.github/workflows/reposix-mirror-sync.yml` shipping with `docs/guides/dvcs-mirror-setup.md`; `repository_dispatch` trigger from confluence webhook + cron safety net; `--force-with-lease` race protection against concurrent bus pushes.
- **L1 perf migration** — replace today's unconditional `list_records` walk in `handle_export` with `list_changed_since`-based conflict detection (single REST call on success path); add `reposix sync --reconcile` escape hatch for cache-desync recovery. L2/L3 hardening defer to v0.14.0.
- **DVCS docs** — `docs/concepts/dvcs-topology.md` (three roles + diagram + when to choose each pattern) + `docs/guides/dvcs-mirror-setup.md` (webhook + Action setup) + troubleshooting matrix entries; cold-reader pass via `doc-clarity-review` against fresh reader who has read only `docs/index.md` + `docs/concepts/mental-model-in-60-seconds.md`.
- **Dark-factory regression — third arm** — extend `scripts/dark-factory-test.sh` so a fresh subprocess agent given only the GH mirror URL completes vanilla-clone + `reposix attach` + bus-push end-to-end with zero in-context learning beyond what the helper's stderr teaches.

**Pre-DVCS hygiene (P0):**
- **Bump `gix` off yanked `=0.82.0`** (closes #29 + #30; `gix-actor` 0.40.1 also yanked) — the `=`-pin is load-bearing per Tech stack.
- **Land 3 WAIVED structure-row verifier scripts** — `no-loose-top-level-planning-audits`, `no-pre-pivot-doc-stubs`, `repo-org-audit-artifact-present` (waivers expire 2026-05-15).
- **POC** — throwaway end-to-end demo in `research/v0.13.0-dvcs/poc/` BEFORE Phase 1 PLAN.md drafted; ~1 day budget; surfaces algorithm-shape decisions cheaply (v0.9.0 precedent).

**Non-negotiable framing principles** (carried from project CLAUDE.md Operating Principles):
- **OP-1 Simulator-first.** All v0.13.0 phases run end-to-end against the simulator. Two simulator instances in one process serve as "confluence-shaped SoT" + "GitHub-shaped mirror" for tests. Real-backend tests (TokenWorld + reubenjohn/reposix) gate the milestone close, not individual phase closes.
- **OP-2 Tainted-by-default.** Mirror writes carry tainted bytes from the SoT. The GH mirror's frontmatter must preserve `Tainted<T>` semantics — a downstream agent reading from the mirror gets the same trifecta protection as one reading SoT directly. The `attach` cache marks all materialized blobs as tainted.
- **OP-3 Audit log non-optional.** Every bus-remote push writes audit rows to BOTH tables — cache audit (helper RPC turn) + backend audit (SoT REST mutation). The mirror push writes a cache-audit row noting "mirror lag now zero" or "mirror lag now N." Webhook-driven syncs write cache-audit rows too.
- **OP-7 Verifier subagent dispatch on every phase close.** Per `quality/PROTOCOL.md`. The DVCS round-trip test is a catalog row in dimension `agent-ux`, kind `subagent-graded`, cadence `pre-pr`.
- **OP-8 +2 phase practice.** v0.13.0 reserves last 2 phases for surprises absorption + good-to-haves polish. The DVCS scope is large enough that something will surface; do not omit the +2 reservation.
- **Per-phase push cadence (codified 2026-04-30).** Every phase closes with `git push origin main` BEFORE verifier-subagent dispatch. Pre-push gate-passing is part of phase-close criterion. Closes backlog 999.4.

**Source-of-truth handover bundle (read these before planning Phase 1):**
- `.planning/research/v0.13.0-dvcs/vision-and-mental-model.md` (the thesis + success gates)
- `.planning/research/v0.13.0-dvcs/architecture-sketch.md` (technical design + open questions)
- `.planning/research/v0.13.0-dvcs/kickoff-recommendations.md` (pre-kickoff readiness moves)
- `.planning/research/v0.13.0-dvcs/decisions.md` (ratified open-question decisions)
- `.planning/milestones/v0.13.0-phases/CARRY-FORWARD.md` (carry-forward items + P0 hygiene)
- `.planning/research/v0.14.0-observability-and-multi-repo/vision-and-mental-model.md` (so the v0.13.0 ROADMAP knows what NOT to absorb when surprises surface)

**Carry-forward (rolled into v0.13.0 phases):**
- `MULTI-SOURCE-WATCH-01` from v0.12.1 P75 — walker hashes every source on `Source::Multi` rows.
- `GIX-YANKED-PIN-01` (P0 hygiene) — bump gix off yanked baseline.
- `WAIVED-STRUCTURE-ROWS-03` (P0/P1 hygiene) — 3 verifier scripts before waiver expires 2026-05-15.
- `POC-DVCS-01` (pre-Phase-1) — end-to-end exploration in `research/v0.13.0-dvcs/poc/`.

**Explicitly NOT in scope (deferred to v0.14.0):**
- OTel / `reposix tail` / multi-project helper (operational maturity for an existing thesis).
- Origin-of-truth frontmatter enforcement (only matters with multi-issues-backend bus; v0.13.0 is 1+1).
- L2/L3 cache-desync hardening (background reconcile, transactional cache writes).
- RETROSPECTIVE.md backfill for v0.9.0 → v0.12.0.

---

## Previously Validated Milestones

Full per-milestone narrative (goal, target features, framing principles, carry-forward) lives in `.planning/PROJECT-history.md`. Cross-milestone retrospective lessons live in `.planning/RETROSPECTIVE.md`. Per-milestone phase artifacts live under `.planning/milestones/v*-phases/`.

| Milestone | Status | Theme | Phases |
|---|---|---|---|
| v0.12.x | SHIPPED 2026-04-30 | Quality Gates + carry-forward drain | 56–65, 72–77 |
| v0.12.0 | SHIPPED 2026-04-28 | Quality Gates framework (dimension / cadence / kind) | 56–63 |
| v0.11.x | SHIPPED 2026-04-27 | Polish & Reproducibility (mkdocs site live; §0.8 SESSION-END-STATE framework) | 50–55 + POLISH2-* |
| v0.10.0 | SHIPPED 2026-04-25 | Docs & Narrative Shine (Diátaxis IA, banned-words linter, cold-reader audit) | 40–45 |
| v0.9.0 | SHIPPED 2026-04-24 | Architecture Pivot — git-native partial clone (FUSE deleted) | 31–36 |
| v0.1.0..v0.8.0 | SHIPPED earlier | MVD + JIRA write path + connector matrix; see history file | see archives |

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
*Last updated: 2026-05-01 — Milestone v0.12.x SHIPPED 2026-04-30; v0.13.0 DVCS over REST in flight. Historical milestone narrative split out to `.planning/PROJECT-history.md` to keep this file under the file-size-limits budget (one logical change per session).*
