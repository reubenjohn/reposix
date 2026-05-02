---
phase: 79
title: "POC + `reposix attach` core"
milestone: v0.13.0
requirements: [POC-01, DVCS-ATTACH-01, DVCS-ATTACH-02, DVCS-ATTACH-03, DVCS-ATTACH-04]
depends_on: [78]
plans:
  - 79-01-PLAN.md  # POC-01: throwaway end-to-end POC at research/v0.13.0-dvcs/poc/
  - 79-02-PLAN.md  # DVCS-ATTACH-01..02 (scaffold + cache module): catalog + clap surface + reconciliation module
  - 79-03-PLAN.md  # DVCS-ATTACH-02..04 (tests + close): integration tests + idempotency/reject + docs + push
waves:
  1: [79-01]   # POC ships first; findings inform 79-02 + 79-03
  2: [79-02]   # scaffold + cache module; absorbs POC findings
  3: [79-03]   # tests + idempotency + reject + docs + close
---

# Phase 79 — POC + `reposix attach` core (overview)

This is the FIRST DVCS-substantive phase of milestone v0.13.0. **Three sequential
plans** (was 2 — split per checker B1 to keep each plan under the context budget):

- **79-01 / POC-01 — End-to-end POC.** Throwaway code in
  `research/v0.13.0-dvcs/poc/` exercising three integration paths against the
  simulator BEFORE the production attach subcommand is designed:
  (a) `reposix attach`-shaped flow against a deliberately-mangled checkout
  (mixed `id`-bearing + `id`-less files, plus a duplicate-`id` and a
  deleted-on-backend case);
  (b) bus-remote-shaped push observing mirror lag (SoT writes succeed, mirror
  trailing — verifies the SoT-first sequencing is sound);
  (c) cheap-precheck path refusing fast on SoT version mismatch (no stdin
  read, no REST writes).
  Ships with `POC-FINDINGS.md` listing algorithm-shape decisions, integration
  friction, and design questions the architecture sketch did not anticipate.
  Time budget: ~1 day; if exceeding 2 days, surface as a SURPRISES-INTAKE
  candidate before continuing (per OP-8 and CARRY-FORWARD POC-DVCS-01).
  POC scope is throwaway — code lives at `research/v0.13.0-dvcs/poc/`, NOT
  inside `crates/`.

- **79-02 / DVCS-ATTACH-01..02 (scaffold + cache module) — `reposix attach`
  subcommand surface + cache reconciliation module.** Lands the catalog row
  (T01), the `reposix attach` clap subcommand surface in `crates/reposix-cli/`
  (T02), and the cache reconciliation table + module + audit hook in
  `crates/reposix-cache/` (T03 — incl. the load-bearing `Cache::log_attach_walk`
  audit write per OP-3). T03 also adds the small set of new public Cache APIs
  (`list_record_ids`, `find_oid_for_record`, `connection_mut`) needed by the
  walker — confirmed not-yet-extant via grep at planning time. NO integration
  tests yet (those are 79-03). Plan terminates with `git push origin main`.

- **79-03 / DVCS-ATTACH-02..04 (tests + idempotency + close) — Behavior
  coverage + multi-SoT reject + Tainted contract + docs.** Lands the 6
  reconciliation-case integration tests (T01), the re-attach idempotency +
  multi-SoT-reject + audit-row + Tainted-materialization tests (T02), and
  the CLAUDE.md update + per-phase push that flips the catalog row from
  FAIL → PASS (T03). Plan terminates with `git push origin main`.

Sequential. POC findings feed into 79-02 (and possibly 79-03) via the
orchestrator's re-engagement protocol; 79-02 lands the scaffold; 79-03
lands behavior coverage. The split keeps each plan ≤ 4 cargo-heavy
tasks per checker B1.

POC findings feed directly into 79-02's implementation — the orchestrator
SHALL surface `POC-FINDINGS.md` back to a planner subagent BEFORE 79-02
execution begins if findings warrant a 79-02 plan revision (see § "POC
findings → planner re-engagement" below).

## Chapters

- [Wave plan](./wave-plan.md)
- [Reframe of DVCS-ATTACH-04](./reframe-dvcs-attach-04.md)
- [New GOOD-TO-HAVES entry](./good-to-haves.md)
- [POC findings → planner re-engagement protocol](./poc-findings-protocol.md)
- [Phase-close protocol](./phase-close-protocol.md)
- [Risks + mitigations](./risks.md)
- [+2 reservation and subagent delegation](./scope-and-delegation.md)

## Plan summary table

| Plan  | Goal                                            | Tasks | Cargo? | Catalog rows minted        | Tests added                     | Files modified (count) |
|-------|-------------------------------------------------|-------|--------|----------------------------|---------------------------------|------------------------|
| 79-01 | POC of three integration paths                  | 5     | partial | 0 (POC is throwaway)       | 0 production tests              | ~6 (all under research/) |
| 79-02 | `reposix attach` scaffold + reconciliation module + audit hook | 3 | YES | 1 (`agent-ux/reposix-attach-against-vanilla-clone`, status FAIL) | 4 unit tests (idempotent CREATE, walk-collects-id-map, duplicate-id-aborts, type-system Tainted assertion) | ~9 (crates/ + catalog + verifier) |
| 79-03 | Reconciliation behavior tests + idempotency/reject + Tainted integration test + audit-row test + docs + close | 3 | YES | 0 (catalog row status flipped FAIL → PASS) | 8 integration tests (6 reconciliation + 2 idempotency/reject) + 2 (Tainted-materialization integration + audit-row) | ~3 (1 new test file + CLAUDE.md + catalog rewrite) |

Total: ~11 tasks across 3 plans (was 11 across 2; one task moved from
the original 79-02 T01-T06 distribution to a 3-task 79-02 + 3-task 79-03
split). Wave plan: sequential (1 wave each).

Test count: 4 unit (79-02) + 10 integration (79-03) = 14 total — increased
by 2 (Tainted-materialization integration test + concrete type assertion)
from the original 11 to deliver the reframed DVCS-ATTACH-04 acceptance.
