## v0.13.0 DVCS over REST (PLANNING)

> **Status:** scoped 2026-04-30. Phases 78–88 derive from `.planning/research/v0.13.0-dvcs/architecture-sketch.md` § "Phase decomposition" + 15 ratified open-question decisions in `decisions.md` + 4 carry-forward items in `CARRY-FORWARD.md`. Handover bundle: `.planning/research/v0.13.0-dvcs/{vision-and-mental-model,architecture-sketch,kickoff-recommendations,decisions}.md` + `.planning/milestones/v0.13.0-phases/CARRY-FORWARD.md` + `.planning/research/v0.14.0-observability-and-multi-repo/vision-and-mental-model.md`.

**Thesis + mental model.** See `.planning/research/v0.13.0-dvcs/vision-and-mental-model.md` for the full statement. Two-sentence summary: shift from "VCS over REST" to "DVCS over REST" — confluence (or any one issues backend) remains the source of truth, but a plain-git mirror on GitHub becomes the universal-read surface; devs `git clone` vanilla, install reposix only to write back, and `git push` via a bus remote that fans out SoT-first then mirror-best-effort. Three roles: SoT-holder (reposix-equipped, writes via bus); mirror-only consumer (vanilla git, read-only); round-tripper (reposix-equipped after `attach`, writes via bus). Mirror-lag observability via plain-git refs (`refs/mirrors/<sot>-head`, `<sot>-synced-at`) — vanilla `git fetch` brings them along.

**Recurring success criteria for EVERY phase (P78–P88)** — non-negotiable per CLAUDE.md Operating Principles + the v0.12.0 autonomous-execution protocol; NOT separate REQ-IDs:

1. **Catalog-first** — phase's FIRST commit writes catalog rows under `quality/catalogs/<file>.json` BEFORE any implementation commit.
2. **CLAUDE.md updated in same PR** (QG-07) — every phase that introduces a new file/convention/gate revises the relevant CLAUDE.md section in the same PR.
3. **Per-phase push** (codified 2026-04-30, closes backlog 999.4) — `git push origin main` BEFORE verifier-subagent dispatch; pre-push gate-passing is part of phase-close criterion.
4. **Phase close = unbiased verifier subagent dispatch (OP-7)** — isolated subagent grades all catalog rows for the phase against artifacts under `quality/reports/verifications/`; verdict at `quality/reports/verdicts/p<N>/VERDICT.md`; phase does not close on RED.
5. **Eager-resolution preference (OP-8)** — items < 1hr / no new dependency get fixed in the discovering phase; else appended to `SURPRISES-INTAKE.md` or `GOOD-TO-HAVES.md`. The +2 reservation (P87 + P88) drains them.
6. **Simulator-first (OP-1)** — all phases run end-to-end against the simulator. Two simulator instances serve as "confluence-shaped SoT" + "GitHub-shaped mirror." Real-backend tests gate milestone close, not individual phase closes.
7. **Tainted-by-default (OP-2)** — mirror writes carry tainted bytes from the SoT; the `attach` cache marks all materialized blobs as tainted.
8. **Audit log non-optional (OP-3)** — every bus-remote push writes audit rows to BOTH tables (cache audit + backend audit); mirror push writes a cache-audit row noting mirror-lag delta; webhook-driven syncs write cache-audit rows too.

### Phase 78: Pre-DVCS hygiene — gix bump, WAIVED-row verifiers, multi-source walker

**Goal:** Land three mutually-parallelizable hygiene items as a single light kickoff phase BEFORE the POC + attach work begins — (a) bump `gix` off the yanked `=0.82.0` baseline (closes #29 + #30); (b) land verifier scripts for the 3 currently-WAIVED structure rows in `quality/catalogs/freshness-invariants.json` BEFORE waivers expire 2026-05-15; (c) schema-migrate the docs-alignment walker to watch every source on `Source::Multi` rows (carry-forward `MULTI-SOURCE-WATCH-01` from v0.12.1 P75 via path-(b) `source_hashes: Vec<String>`).

**Requirements:** HYGIENE-01, HYGIENE-02, MULTI-SOURCE-WATCH-01 · **Depends on:** — (entry-point phase; v0.12.1 SHIPPED 2026-04-30 is the precondition) · **Plan:** [78-PLAN-OVERVIEW](../../phases/78-pre-dvcs-hygiene/78-PLAN-OVERVIEW.md)

### Phase 79: POC + `reposix attach` core

**Goal:** Ship the POC first (`research/v0.13.0-dvcs/poc/`) to surface design surprises, then implement `reposix attach <backend>::<project>` — builds a fresh cache from REST against an existing checkout, reconciles by `id` in frontmatter, populates the cache reconciliation table, and adds remote `reposix::<sot>?mirror=<existing-origin-url>`. Reconciliation cases (match / backend-deleted via `--orphan-policy` / no-id / duplicate-id / mirror-lag) per `architecture-sketch.md`. Re-attach to different SoT REJECTED (Q1.2); same-SoT IDEMPOTENT (Q1.3). Blobs `Tainted<Vec<u8>>` (OP-2).

**Requirements:** POC-01, DVCS-ATTACH-01..04 · **Depends on:** P78 GREEN · **Plan:** [79-PLAN-OVERVIEW](../../phases/79-poc-reposix-attach-core/79-PLAN-OVERVIEW.md)

**Plans status:**
- **79-01** — POC-01 (throwaway POC at `research/v0.13.0-dvcs/poc/`). **SHIPPED 2026-05-01** (5 commits 660bae0..4e6de2b; SUMMARY at `.planning/phases/79-poc-reposix-attach-core/79-01-SUMMARY.md`; FINDINGS at `research/v0.13.0-dvcs/poc/POC-FINDINGS.md` — 5 INFO + 2 REVISE + 0 SPLIT routing tags).
- **79-02** — DVCS-ATTACH-01..02 (scaffold + cache reconciliation module). PENDING orchestrator's POC-FINDINGS re-engagement decision.
- **79-03** — DVCS-ATTACH-02..04 (tests + idempotency + close). PENDING 79-02.

### Phase 80: Mirror-lag refs (`refs/mirrors/<sot>-head`, `<sot>-synced-at`)

**Goal:** Wire mirror-lag observability into plain-git refs that vanilla `git fetch` brings along, so `git log refs/mirrors/<sot>-synced-at` reveals staleness to readers who never installed reposix. Two refs in the `refs/mirrors/...` namespace (Q2.1): `<sot>-head` records the SHA of the SoT's `main` at last sync; `<sot>-synced-at` is an annotated tag with a timestamp message. Helper APIs land in `crates/reposix-cache/`. The existing single-backend push (today's `handle_export`) is wired to update both refs on success — pre-bus integration point so observability is in place BEFORE bus phases. Webhook sync also writes both refs (P84). Bus push will update both refs (P83). Q2.3: bus updates both refs (consistency over optimization); webhook becomes a no-op refresh when bus already touched them.

**Requirements:** DVCS-MIRROR-REFS-01..03 · **Depends on:** P79 GREEN · **Plan:** [80-PLAN-OVERVIEW](../../phases/80-mirror-lag-refs/80-PLAN-OVERVIEW.md)

### Phase 81: L1 perf migration — `list_changed_since`-based conflict detection

**Goal:** Replace `handle_export`'s unconditional `list_records` walk with `list_changed_since`-based conflict detection so the bus remote (P82–P83) inherits the cheap path (Q3.1). Net REST cost on success path: one call (`list_changed_since`) + actual REST writes. L1 trusts cache as the prior; `reposix sync --reconcile` is the on-demand escape hatch. L2/L3 hardening defers to v0.14.0.

**Requirements:** DVCS-PERF-L1-01..03 · **Depends on:** P80 GREEN · **Plan:** [81-PLAN-OVERVIEW](../../phases/81-l1-perf-migration/81-PLAN-OVERVIEW.md)

### Phase 82: Bus remote — URL parser, prechecks, fetch dispatch

**Goal:** Stand up the bus remote's read/dispatch surface. URL parser recognizes `reposix::<sot>?mirror=<url>` (Q3.3); `+`-delimited form rejected. CHEAP PRECHECK A (mirror drift via `git ls-remote`) + CHEAP PRECHECK B (SoT drift via `list_changed_since`) run BEFORE stdin read. Bus does NOT advertise `stateless-connect` for fetch (Q3.4). Dispatch-only; WRITE fan-out (steps 4–9) lands in P83.

**Requirements:** DVCS-BUS-URL-01, DVCS-BUS-PRECHECK-01..02, DVCS-BUS-FETCH-01 · **Depends on:** P81 GREEN · **Plan:** [82-PLAN-OVERVIEW](../../phases/82-bus-remote-url-parser/82-PLAN-OVERVIEW.md)

### Phase 83: Bus remote — write fan-out (SoT-first, mirror-best-effort, fault injection)

**Goal:** Implement the riskiest part of the bus remote — the SoT-first-write algorithm with mirror-best-effort fallback and full fault-injection coverage. Algorithm: read fast-import stream from stdin and buffer; apply REST writes to SoT (confluence); on success, write audit rows to BOTH tables (cache + backend) and update `last_fetched_at`; then `git push` to GH mirror; on mirror failure, write mirror-lag audit row, update `refs/mirrors/<sot>-head` to new SoT SHA but NOT `<sot>-synced-at`, return ok to git (SoT contract satisfied — recoverable on next push). On mirror success, update `<sot>-synced-at` to now and ack. NO helper-side retry on transient mirror-write failures (Q3.6) — surface, audit, let user retry. Fault-injection tests cover kill-GH-push-between-confluence-write-and-ack / kill-confluence-write-mid-stream / simulate-confluence-409-after-precheck.

**Requirements:** DVCS-BUS-WRITE-01..06 · **Depends on:** P82 GREEN · **Plan:** [83-PLAN-OVERVIEW](../../phases/83-bus-write-fan-out/83-PLAN-OVERVIEW.md) (P83-01 + P83-02 SHIPPED)

### Phase 84: Webhook-driven mirror sync — GH Action workflow + setup guide

**Goal:** Ship the reference GitHub Action workflow that keeps the GH mirror current with confluence-side edits — the pull side of the DVCS topology. Workflow at `.github/workflows/reposix-mirror-sync.yml` triggers on `repository_dispatch` (event type `reposix-mirror-sync`) plus a cron safety net (default `*/30`, configurable via workflow `vars` per Q4.1). Workflow runs `reposix init confluence + git push <mirror>` and updates `refs/mirrors/...` refs. Uses `--force-with-lease` against last known mirror ref so a concurrent bus-push's race doesn't corrupt mirror state. First-run handling (no existing mirror refs, empty mirror) is graceful per Q4.3. Latency target: < 60s p95 from confluence edit to GH ref update. Backends without webhooks (Q4.2): cron path is the only sync mechanism.

**Requirements:** DVCS-WEBHOOK-01..04 · **Depends on:** P80 + P83 GREEN · **Plan:** [84-PLAN-OVERVIEW](../../phases/84-webhook-mirror-sync/84-PLAN-OVERVIEW.md)

### Phase 85: DVCS docs — topology, mirror setup, troubleshooting, cold-reader pass

**Goal:** Make v0.13.0 legible to a cold reader. Three new doc surfaces ship: `docs/concepts/dvcs-topology.md` (three roles + diagram from `vision-and-mental-model.md` + when-to-choose-which-pattern guidance + the verbatim Q2.2 clarification *"`refs/mirrors/<sot>-synced-at` is the timestamp the mirror last caught up to <sot> — it is NOT a 'current SoT state' marker"*); `docs/guides/dvcs-mirror-setup.md` (walk-through of webhook + Action setup; backends-without-webhooks fallback per Q4.2; cleanup procedure); troubleshooting matrix entries in `docs/guides/troubleshooting.md` covering bus-remote `fetch first` rejection messages, attach reconciliation warnings, webhook race conditions, cache-desync recovery via `reposix sync --reconcile`. Cold-reader pass via `doc-clarity-review`. Banned-words enforced (no FUSE residue; no jargon-above-Layer-3 leaks).

**Requirements:** DVCS-DOCS-01..04 · **Depends on:** P79 + P80 + P81 + P82 + P83 + P84 ALL GREEN · **Plan:** TBD (P85 plan-overview not yet authored)

**Success criteria:**
1. `docs/concepts/dvcs-topology.md` ships with three roles + diagram + Q2.2 clarification + when-to-choose guidance; banned-words clean.
2. `docs/guides/dvcs-mirror-setup.md` ships with end-to-end webhook + Action walk-through, cron-only fallback (Q4.2), cleanup procedure.
3. Troubleshooting matrix entries land for bus-remote rejection / attach reconciliation / webhook race / cache-desync recovery.
4. `doc-clarity-review` cold-reader pass against a reader who has read only `docs/index.md` + `docs/concepts/mental-model-in-60-seconds.md`; zero critical-friction findings.
5. mkdocs nav + banned-words clean (`bash scripts/check-docs-site.sh` + `scripts/banned-words-lint.sh` GREEN; mermaid diagrams render).
6. Catalog rows land first (docs-alignment binding the three new docs to verifier tests; subjective-rubric for cold-reader pass with `freshness_ttl: 30d`); CLAUDE.md updated.
7. Phase close: `git push origin main`; verifier subagent grades GREEN; verdict at `quality/reports/verdicts/p85/VERDICT.md`.

**Context anchor:** `.planning/research/v0.13.0-dvcs/vision-and-mental-model.md` § "Mental model"; `.planning/research/v0.13.0-dvcs/architecture-sketch.md` § "Webhook-driven mirror sync"; `.planning/research/v0.13.0-dvcs/decisions.md` § Q2.2 + § Q4.2 + § "POC scope"; existing `docs/guides/troubleshooting.md`; `.claude/skills/doc-clarity-review/SKILL.md`.

### Phase 86: Dark-factory regression — third arm (vanilla-clone + attach + bus-push)

**Goal:** Extend `quality/gates/agent-ux/dark-factory.sh` (formerly `scripts/dark-factory-test.sh`, migrated in v0.12.0 P59 SIMPLIFY-07) to add a third subprocess-agent transcript: a fresh agent given only the GH mirror URL + a goal completes vanilla-clone + `reposix attach` + edit + bus-push end-to-end with zero in-context learning beyond what the helper's stderr teaches. Reuses the existing dark-factory test harness; no in-prompt instruction beyond the goal statement. The transcript proves the DVCS thesis: a curious developer who has never read a reposix doc can still complete the round-trip because the helper teaches itself via stderr (blob-limit error names `git sparse-checkout`; bus reject names mirror-lag refs; attach errors name `--orphan-policy`).

**Requirements:** DVCS-DARKFACTORY-01..02 · **Depends on:** P79 + P80 + P81 + P82 + P83 + P84 + P85 ALL GREEN · **Plan:** TBD (P86 plan-overview not yet authored)

**Success criteria:**
1. Third arm in dark-factory harness (`dvcs-third-arm` scenario; existing two arms unchanged); subprocess agent prompt: *"The repo at <GH-mirror-url> mirrors a confluence backend. Install reposix, attach, fix the bug in `issues/0001.md` (typo on line 3), push your fix back. You have 10 minutes."*
2. Zero in-context learning required: agent did NOT receive the bus URL syntax / `attach` spelling / `--orphan-policy` / mirror-lag ref namespace; all four are recovered from helper stderr or `--help`.
3. End-to-end success: typo fix lands in confluence (REST GET) AND GH mirror (`git fetch && git log`) AND `refs/mirrors/<sot>-synced-at` advanced; audit rows present in both tables.
4. Catalog row in `agent-ux` dimension (`dvcs-third-arm`, `kind: subagent-graded`, `cadence: pre-pr`, `freshness_ttl: 30d`).
5. Sim AND TokenWorld coverage (CI default + secrets-gated real-backend; milestone-close gate per OP-1).
6. Catalog rows + helper-stderr docs-alignment rows land first; CLAUDE.md updated.
7. Phase close: `git push origin main`; verifier GREEN; verdict at `quality/reports/verdicts/p86/VERDICT.md`.

**Context anchor:** `.planning/research/v0.13.0-dvcs/vision-and-mental-model.md` § "Success gates" #6; `quality/gates/agent-ux/dark-factory.sh`; `.claude/skills/reposix-agent-flow/SKILL.md`; `docs/reference/testing-targets.md`; `quality/PROTOCOL.md` § "Verifier subagent prompt template".

### Phase 87: Surprises absorption (+2 reservation slot 1)

**Goal:** Drain `.planning/milestones/v0.13.0-phases/SURPRISES-INTAKE.md` per OP-8. Each entry → RESOLVED | DEFERRED | WONTFIX with commit SHA or rationale. Verifier honesty spot-check on previous phases (P78–P86) plans + verdicts asks: *"did this phase honestly look for out-of-scope items?"* Empty intake is acceptable IF phases produced explicit `Eager-resolution` decisions in their plans; empty intake when verdicts show skipped findings is RED. The +2 reservation is in addition to P78–P86 planned phases per CLAUDE.md OP-8 — eager-resolution preference is the default; surprises absorption picks up only what eager-resolution couldn't.

**Requirements:** DVCS-SURPRISES-01 · **Depends on:** P78 + P79 + P80 + P81 + P82 + P83 + P84 + P85 + P86 ALL GREEN · **Plan:** TBD (P87 plan-overview not yet authored)

**Success criteria:**
1. Every entry in `SURPRISES-INTAKE.md` has terminal STATUS (RESOLVED + commit SHA / DEFERRED + target milestone + rationale / WONTFIX + rationale). No `STATUS: TBD` at phase close.
2. Verifier honesty spot-check samples ≥3 P78–P86 plan/verdict pairs; spot-check report at `quality/reports/verdicts/p87/honesty-spot-check.md`. Empty intake acceptable IF phases produced explicit `Eager-resolution` decisions; empty intake when verdicts show skipped findings → RED.
3. Catalog deltas computed: any SURPRISES entries that flip catalog rows update cleanly; alignment_ratio + coverage_ratio deltas reported in the phase verdict.
4. No silent scope creep: P78–P86 verdicts must have flagged out-of-scope findings via Eager-resolution OR SURPRISES-INTAKE.
5. Catalog rows for v0.14.0 carry-forwards land before TBD → DEFERRED flips; CLAUDE.md updated if any v0.14.0 carry-forward sentences land.
6. Phase close: `git push origin main`; verifier GREEN AND signs honesty spot-check; verdict at `quality/reports/verdicts/p87/VERDICT.md`.

**Context anchor:** CLAUDE.md § "Operating Principles" OP-8; `.planning/milestones/v0.13.0-phases/SURPRISES-INTAKE.md`; `.planning/milestones/v0.12.1-phases/` P76 verdict (precedent for honesty-check execution).

### Phase 88: Good-to-haves polish (+2 reservation slot 2) — milestone close

**Goal:** Drain `.planning/milestones/v0.13.0-phases/GOOD-TO-HAVES.md` per OP-8 sizing rules — XS items always close in-phase; S items close if budget; M items default-defer to v0.14.0. After polish, finalize milestone-close artifacts: CHANGELOG `[v0.13.0]` entry; tag-script at `.planning/milestones/v0.13.0-phases/tag-v0.13.0.sh` (≥6 safety guards mirroring v0.12.0 precedent); RETROSPECTIVE.md v0.13.0 section distilled per OP-9 ritual; milestone-close verifier subagent dispatched and GREEN. Owner runs `tag-v0.13.0.sh` and pushes the tag — orchestrator does NOT push the tag.

**Requirements:** DVCS-GOOD-TO-HAVES-01 · **Depends on:** P87 GREEN · **Plan:** TBD (P88 plan-overview not yet authored)

**Success criteria:**
1. GOOD-TO-HAVES.md drained: every entry terminal STATUS — XS closed (commit SHA), S closed-or-deferred (rationale), M default-deferred to v0.14.0 (carry-forward target named).
2. CHANGELOG `[v0.13.0]` finalized: summarizes P78–P88 + lists every shipped REQ-ID by category + names v0.14.0 carry-forward.
3. Tag-script authored at `.planning/milestones/v0.13.0-phases/tag-v0.13.0.sh` with ≥6 safety guards (clean tree, on `main`, version match, CHANGELOG entry exists, tests green, signed tag); tag-gate guards re-run cleanly post-P88.
4. RETROSPECTIVE.md v0.13.0 section distilled (OP-9) BEFORE archive: What Was Built / What Worked / What Was Inefficient / Patterns Established / Key Lessons. Source: SURPRISES-INTAKE + GOOD-TO-HAVES + per-phase verdicts + autonomous-run findings.
5. Milestone-close verifier dispatched and GREEN at `quality/reports/verdicts/milestone-v0.13.0/VERDICT.md`: P78–P88 catalog rows all GREEN-or-WAIVED; dark-factory three-arm transcript GREEN against sim AND TokenWorld; no expired waivers without follow-up; RETROSPECTIVE v0.13.0 section exists; +2 reservation operational.
6. STOP at tag boundary: orchestrator does NOT push the tag. STATE.md cursor updated to "v0.13.0 ready-to-tag; owner pushes tag."
7. Catalog rows + CLAUDE.md v0.13.0-shipped historical-milestone subsection land first.
8. Phase close: `git push origin main`; milestone-close verifier GREEN; verdict at `quality/reports/verdicts/p88/VERDICT.md` + `quality/reports/verdicts/milestone-v0.13.0/VERDICT.md`.

**Context anchor:** CLAUDE.md § "Operating Principles" OP-8 + OP-9; `.planning/milestones/v0.13.0-phases/GOOD-TO-HAVES.md`; `.planning/RETROSPECTIVE.md`; `.planning/milestones/v0.12.0-phases/tag-v0.12.0.sh` + `.planning/milestones/v0.11.0-phases/tag-v0.11.0.sh` (tag-script precedents); `.planning/research/v0.14.0-observability-and-multi-repo/vision-and-mental-model.md` (carry-forward target).
