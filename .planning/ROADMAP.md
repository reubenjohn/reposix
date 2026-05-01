# Roadmap: reposix

## Milestones

- ✅ **v0.1.0 MVD** — Phases 1-4, S (shipped 2026-04-13) · [archive](milestones/v0.8.0-phases/ROADMAP.md)
- ✅ **v0.2.0-alpha** — Phase 8: GitHub read-only adapter (shipped 2026-04-13)
- ✅ **v0.3.0** — Phase 11: Confluence Cloud read-only adapter (shipped 2026-04-14)
- ✅ **v0.4.0** — Phase 13: Nested mount layout pages/+tree/ (shipped 2026-04-14)
- ✅ **v0.5.0** — Phases 14-15: IssueBackend decoupling + bucket _INDEX.md (shipped 2026-04-14)
- ✅ **v0.6.0** — Phases 16-20: Write path + full sitemap (shipped 2026-04-14)
- ✅ **v0.7.0** — Phases 21-26: Hardening + Confluence expansion + docs (shipped 2026-04-16)
- ✅ **v0.8.0 JIRA Cloud Integration** — Phases 27-29 (shipped 2026-04-16)
- ✅ **v0.9.0 Architecture Pivot — Git-Native Partial Clone** — Phases 31–36 (shipped 2026-04-24) · [archive](milestones/v0.9.0-phases/ROADMAP.md)
- ✅ **v0.10.0 Docs & Narrative Shine** — Phases 40–45 (shipped 2026-04-25) · [archive](milestones/v0.10.0-phases/ROADMAP.md)
- ✅ **v0.11.x Polish & Reproducibility** — Phases 50–55 + POLISH2-* polish passes (v0.11.0 shipped 2026-04-25; v0.11.1 + v0.11.2 polish passes shipped 2026-04-26 / 2026-04-27 via release-plz; all 8 crates published to crates.io at v0.11.2)
- ✅ **v0.12.0 Quality Gates** — Phases 56–65 (shipped 2026-04-29) · [archive](milestones/v0.12.0-phases/ROADMAP.md)
- ✅ **v0.12.1 Polish** — Phases 72–77 (shipped 2026-04-30) · [archive](milestones/v0.12.1-phases/ARCHIVE.md)
- 🚧 **v0.13.0 DVCS over REST** — Phases 78–88 (in flight; scoped 2026-04-30) · [milestone roadmap](milestones/v0.13.0-phases/ROADMAP.md)

## Phases

## v0.13.0 DVCS over REST (PLANNING)

> **Status:** scoped 2026-04-30. The full milestone roadmap (thesis, mental model, recurring success criteria, and per-phase detail for P78–P88) lives at `.planning/milestones/v0.13.0-phases/ROADMAP.md`. The index below is a navigable summary; click each phase for its `*-PLAN-OVERVIEW.md` (where it exists) or read the milestone roadmap for full per-phase scope, success criteria, and context anchors.
>
> **Source-of-truth handover bundle:** `.planning/research/v0.13.0-dvcs/{vision-and-mental-model,architecture-sketch,kickoff-recommendations,decisions}.md` + `.planning/milestones/v0.13.0-phases/CARRY-FORWARD.md` + `.planning/research/v0.14.0-observability-and-multi-repo/vision-and-mental-model.md` (so v0.13.0 knows what NOT to absorb).
>
> **Recurring per-phase success criteria** (catalog-first, CLAUDE.md-in-same-PR, per-phase push, unbiased verifier subagent, eager-resolution preference, simulator-first, tainted-by-default, dual-table audit) — see milestone ROADMAP.md § "Recurring success criteria for EVERY phase (P78–P88)".

### Phase index (P78–P88)

- **P78** Pre-DVCS hygiene — gix bump, WAIVED-row verifiers, multi-source walker · [78-PLAN-OVERVIEW](phases/78-pre-dvcs-hygiene/78-PLAN-OVERVIEW.md)
- **P79** POC + `reposix attach` core · [79-PLAN-OVERVIEW](phases/79-poc-reposix-attach-core/79-PLAN-OVERVIEW.md)
- **P80** Mirror-lag refs (`refs/mirrors/<sot>-head`, `<sot>-synced-at`) · [80-PLAN-OVERVIEW](phases/80-mirror-lag-refs/) (TBD)
- **P81** L1 perf migration — `list_changed_since`-based conflict detection · [81-PLAN-OVERVIEW](phases/81-l1-perf-migration/) (TBD)
- **P82** Bus remote — URL parser, prechecks, fetch dispatch · [82-PLAN-OVERVIEW](phases/82-bus-remote-url-parser/) (TBD)
- **P83** Bus remote — write fan-out (SoT-first, mirror-best-effort, fault injection) · [83-PLAN-OVERVIEW](phases/83-bus-write-fan-out/83-PLAN-OVERVIEW.md)
- **P84** Webhook-driven mirror sync — GH Action workflow + setup guide · [84-PLAN-OVERVIEW](phases/84-webhook-mirror-sync/) (TBD)
- **P85** DVCS docs — topology, mirror setup, troubleshooting, cold-reader pass · [85-PLAN-OVERVIEW](phases/85-dvcs-docs/) (TBD)
- **P86** Dark-factory regression — third arm (vanilla-clone + attach + bus-push) · [86-PLAN-OVERVIEW](phases/86-dark-factory-third-arm/) (TBD)
- **P87** Surprises absorption (+2 reservation slot 1) · [87-PLAN-OVERVIEW](phases/87-surprises-absorption/) (TBD)
- **P88** Good-to-haves polish (+2 reservation slot 2) — milestone close · [88-PLAN-OVERVIEW](phases/88-good-to-haves-polish/) (TBD)

For full per-phase goals, requirements, dependencies, success criteria, and context anchors, see `.planning/milestones/v0.13.0-phases/ROADMAP.md` per CLAUDE.md §0.5 / Workspace layout (verbose per-phase blocks live in the milestone roadmap; this top-level file is the navigable index).

## Previously planned milestones

Per CLAUDE.md §0.5 / Workspace layout, each shipped/historical milestone's
ROADMAP.md lives inside its `*-phases/` directory. Top-level ROADMAP.md
holds ONLY the active milestone (currently v0.13.0) + this index.

- **v0.12.1** Polish — see `.planning/milestones/v0.12.1-phases/ARCHIVE.md` (Phases 72–77, SHIPPED 2026-04-30).
- **v0.12.0** Quality Gates — `.planning/milestones/v0.12.0-phases/ROADMAP.md` (Phases 56–65, SHIPPED 2026-04-29).
- **v0.11.0** Polish & Reproducibility — `.planning/milestones/v0.11.0-phases/ROADMAP.md` (Phases 50–55, SHIPPED 2026-04-25 → 2026-04-27).
- **v0.10.0** Docs & Narrative Shine — `.planning/milestones/v0.10.0-phases/ROADMAP.md` (Phases 40–45, SHIPPED 2026-04-25).
- **v0.9.0** Architecture Pivot — `.planning/milestones/v0.9.0-phases/ROADMAP.md` (Phases 31–36, SHIPPED 2026-04-24).
- v0.8.0 and earlier — see `.planning/milestones/v0.X.0-phases/ARCHIVE.md` per the POLISH2-21 condensation (8 archives, v0.1.0 → v0.8.0).

## Backlog

### Phase 999.1: Follow-up — missing SUMMARY.md files from prior phases (BACKLOG)

**Goal:** Resolve plans that ran without producing summaries during earlier phase executions
**Deferred at:** 2026-04-16 during /gsd-next advancement to /gsd-verify-work (Phase 29 → milestone completion)
**Plans:**
- [ ] Phase 16: 16-D-docs-and-release (ran, no SUMMARY.md)
- [ ] Phase 17: 17-A-workload-and-cli (ran, no SUMMARY.md)
- [ ] Phase 17: 17-B-tests-and-docs (ran, no SUMMARY.md)
- [ ] Phase 18: 18-02 (ran, no SUMMARY.md)
- [ ] Phase 21: 21-A-audit (ran, no SUMMARY.md)
- [ ] Phase 21: 21-B-contention (ran, no SUMMARY.md)
- [ ] Phase 21: 21-C-truncation (ran, no SUMMARY.md)
- [ ] Phase 21: 21-D-chaos (ran, no SUMMARY.md)
- [ ] Phase 21: 21-E-macos (ran, no SUMMARY.md)
- [ ] Phase 22: 22-A-bench-upgrade (ran, no SUMMARY.md)
- [ ] Phase 22: 22-B-fixtures-and-table (ran, no SUMMARY.md)
- [ ] Phase 22: 22-C-wire-docs-ship (ran, no SUMMARY.md)
- [ ] Phase 25: 25-02 (ran, no SUMMARY.md)
- [ ] Phase 27: 27-02 (ran, no SUMMARY.md)

### Phase 999.2: `confirm-retire --all-proposed` batch flag (BACKLOG)

**Goal:** Eliminate ad-hoc bash loops when draining RETIRE_PROPOSED rows
**Source:** 2026-04-30 session — 27-row drain required hand-rolled `jq | while read | call CLI per id` loop. OP #4: ad-hoc bash is a missing-tool signal.
**Plans:**
- [ ] Add `--all-proposed` (and/or `--ids-from-file`) flag to `reposix-quality doc-alignment confirm-retire`
- [ ] Preserve `--i-am-human` semantics + per-row audit trail entry
- [ ] Test on a fresh propose-retire fixture

### Phase 999.3: Pre-push runner — separate `timed_out` from `asserts_failed` (BACKLOG)

**Goal:** Stop network-flake timeouts from being recorded as gate FAIL when assertions actually passed
**Source:** 2026-04-30 session — `release/crates-io-max-version/reposix-confluence` recorded `status: FAIL` despite `asserts_passed: [4]`, `asserts_failed: []`, `timed_out: true`. False positive every weekly run.
**Plans:**
- [ ] Audit `quality/runners/run.py` status-derivation logic
- [ ] Distinguish `TIMEOUT` (preserve last semantic verdict, surface as PARTIAL?) from `FAIL` (asserts truly failed)
- [ ] Backfill any rows currently FAIL-by-timeout

### Phase 999.4: Autonomous-run push cadence — RESOLVED 2026-04-30

**Resolution:** Per-phase push. Codified in `CLAUDE.md` § "GSD workflow" → "Push cadence — per-phase" (this commit). Phase-close subagent issues `git push origin main` BEFORE verifier dispatch; pre-push gate-passing is part of the close criterion. Pre-commit fmt hook (a25f6ff) stays on as secondary safety net. Decision made at v0.13.0 kickoff per `.planning/research/v0.13.0-dvcs/kickoff-recommendations.md` rec #3.

### Phase 999.5: `docs/reference/crates.md` — zero claim-to-test coverage (BACKLOG)

**Goal:** Bind the most-uncovered docs file to verifier rows
**Source:** 2026-04-30 session — `doc-alignment status` shows 0 rows / 147 eligible lines on `docs/reference/crates.md`. Largest single uncovered surface in the catalog.
**Plans:**
- [ ] Extract claims via `/reposix-quality-backfill` scoped to this doc
- [ ] Bind tests; retire-propose any qualitative-only claims
- [ ] Re-walk; confirm coverage_ratio bump

### Phase 999.6: Docs-alignment coverage climb (BACKLOG)

**Goal:** Raise overall `coverage_ratio` from 0.2031 toward the next milestone target
**Source:** 2026-04-30 session — current ratio is 2× above floor (0.10) but headroom is large. Natural next dimension target after retire-backlog drained.
**Plans:**
- [ ] Set milestone-level coverage target (e.g., 0.30 or 0.40)
- [ ] Identify worst-covered docs (`status` per-file table)
- [ ] Allocate 2-3 phases of binding work per worst offender
- [ ] Track via `claims_bound` and `coverage_ratio` headline numbers
