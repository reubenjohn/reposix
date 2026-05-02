# Phase 83: Bus remote — write fan-out (SoT-first, mirror-best-effort, fault injection) — Research

**Researched:** 2026-05-01
**Domain:** git remote helper protocol; SoT-first fan-out write; mirror-best-effort with audit; fault injection via wiremock + file:// mirror fixtures
**Confidence:** HIGH (reuse path is fully shipped; fault-injection donors exist in `tests/push_conflict.rs` + `tests/bus_precheck_*.rs`)

## Summary

Phase 83 implements the **riskiest** part of the bus remote — steps 4-9 of the architecture-sketch's `§ 3` algorithm: read fast-import from stdin, write to SoT, fan out to mirror, audit-row both tables, update the two `refs/mirrors/<sot>-*` refs differently depending on which leg of the fan-out succeeded. Per `decisions.md` Q3.6 (RATIFIED 2026-04-30): **no helper-side retry** — surface failures, audit them, let the user retry the whole push.

The structural insight, validated against the live source: **the SoT-write half of the algorithm is `handle_export` lines 360-606 verbatim** — parse stdin, run the L1 precheck, plan, execute create/update/delete, write `helper_push_accepted` cache audit row, write `last_fetched_at` cursor, write the two mirror refs, write `mirror_sync_written` audit row, ack `ok refs/heads/main` to git. P83's job is NOT to rewrite this loop. It is to (a) lift the post-`bus_handler::handle_bus_export`-precheck portion of `handle_export` into a shared `apply_writes` function with a narrow signature (mirroring P81's `precheck_export_against_changed_set` narrow-deps refactor), (b) interpose a `git push <mirror_remote_name> main` shell-out between SoT write and the synced-at ref write, and (c) handle the new partial-failure end-state where SoT writes land but the mirror push fails. The mirror-push subprocess works in the bus_handler's `cwd` (the working tree where `mirror_remote_name` was resolved during P82's STEP 0) — that cwd is preserved across the function call.

**Primary recommendation:** **Split into P83a + P83b.** P83a delivers the write-fan-out core: refactor `handle_export` write-loop into `apply_writes(...)` with narrow deps, wire `bus_handler::handle_bus_export` to call it, add the `git push <mirror>` subprocess + ref/audit branching, ship 4 happy-path/no-mirror-remote integration tests. P83b delivers the 3 fault-injection scenarios + audit-completeness verification + P82↔P83 integration smoke. This split aligns with the ROADMAP's explicit *"may want to split"* carve-out (P83 §147 last sentence) and the build-memory-budget constraint (CLAUDE.md §"Build memory budget" — fault-injection tests are heavy linkage; serializing them into a second phase keeps each phase's cargo budget tight). A single P83 is doable but compounds risk: any architectural ambiguity caught during fault-injection forces a re-plan of the core, doubling cost.

## Chapters

- [**01-Architectural Responsibility Map**](./01-architectural-responsibility.md) — Maps capabilities to primary/secondary tiers with rationale; shows which code modules own what.
- [**02-User Constraints**](./02-user-constraints.md) — Locked decisions from decisions.md, Claude's discretion, and deferred ideas; user-facing constraints.
- [**03-Phase Requirements**](./03-phase-requirements.md) — 6 DVCS-BUS-WRITE requirements (01–06) with research support.
- [**04-Standard Stack**](./04-standard-stack.md) — Dependencies, versions, library table, verification notes.
- [**05-Architecture Patterns**](./05-architecture-patterns.md) — System architecture diagram, recommended project structure, key patterns.
- [**06-Mirror Write Algorithm**](./06-mirror-write-algorithm.md) — Exact state transitions for mirror push + ref writes; fault paths.
- [**07-Audit Design**](./07-audit-design.md) — Mirror-lag audit row shape, dual-table completeness contract, catalog rows.
- [**08-Fault Injection**](./08-fault-injection.md) — Test infrastructure, 4 test scenarios, common donor patterns.
- [**09-Pitfalls & Security**](./09-pitfalls-security.md) — Common pitfalls (6), risks summary, ASVS security domain, threat patterns.
- [**10-Validation & Sources**](./10-validation-sources.md) — Validation architecture, primary/secondary/tertiary sources, assumptions log.
- [**11-Metadata**](./11-metadata.md) — Confidence breakdown, validity window, resolved open questions.

## Key Insight

**The core refactor is not a rewrite.** The SoT-write portion of `handle_export` (lines 360-606) lifts verbatim into `apply_writes()`. P83's job is to (1) thread that function into bus_handler, (2) add the mirror-push shell-out between SoT and synced-at writes, and (3) handle the new partial-failure case. The complexity is in the branching and audit, not in the algorithm.
