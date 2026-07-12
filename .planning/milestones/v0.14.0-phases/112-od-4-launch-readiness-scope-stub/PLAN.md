---
phase: 112
plan: 01
title: OD-4 launch-readiness — SCOPE-BUT-DO-NOT-START stub
type: scope-stub
severity: none
autonomous: false
requirements: [D2-OD4-STUB-01]
depends_on: [P111]  # P111 is GREEN (milestone-close grade c259718). Execution against this scope was blocked until P111 GREEN — now satisfied — but the milestone itself is DEFERRED past the owner-cut v0.14.0 tag (see banner).
provides: [launch-readiness-milestone-scope]
affects: []  # no code, no gates, no catalog rows — planning artifact only
---

# Phase 112: OD-4 launch-readiness — SCOPE-BUT-DO-NOT-START stub

> ## DO NOT START THE LAUNCH-READINESS MILESTONE IN THIS SESSION
>
> **This phase SCOPES the OD-4 launch-readiness milestone; it does NOT execute it.**
> No asciinema recording, no demo script, no headline-number measurement, no
> README / positioning copy, no install-path work, no code, no catalog rows are
> produced or started here. Execution is **deferred to a post-tag session** that
> authors the real milestone via `/gsd-new-milestone`. Writing any implementation
> against these four pillars in THIS phase is out of scope.

## Objective

Scope the OD-4 launch-readiness milestone — name its four pillars, its sequencing, and
its owner mandate — and mark it **deferred** behind the owner-cut v0.14.0 tag. This
artifact is a durable pointer for the next session to pick up cold; it is deliberately
NOT the milestone itself (no decomposition, no per-pillar plan, no REQ-IDs beyond the
single stub requirement).

## OD-4 reference

Owner decision **OD-4** — "Delegation tiering, quality-convergence-before-features,
launch-readiness resequence" — item 3 (EXECUTIVE RESEQUENCE), at
`.planning/phases/89-framework-fixes-cadence-shell-kind/89-OWNER-DECISIONS.md`
**lines 137–164**. The launch-readiness milestone is "pulled forward from the
v0.10.0-post-pivot v1.0 vision," goal: **global-map adoption**. Owner quote: *"make
executive autonomous decisions and tangential pivots that ultimately puts this project
on the global map."*

## The four launch-readiness pillars (OD-4)

1. **asciinema hero demo** — a recorded terminal cast of the pure-git agent UX
   (init/attach → `cat`/`grep`/`sed`/`git push`) as the landing-page hero.
2. **CI-verified honest headline numbers** — headline latency/throughput claims
   measured and asserted in CI, so the weekly badge is honestly green (no hand-waved
   numbers).
3. **install-path excellence** — a frictionless install story (package-manager-first
   path, `reposix doctor`, verified badge/link freshness) for a skeptical first-time dev.
4. **positioning / Show-HN kit** — the launch narrative plus Show-HN assets (positioning
   copy, comparison framing, submission-ready kit) for adoption.

No phase decomposition, REQ-IDs, or execution detail per pillar — those are authored by
the dedicated OD-4 scoping session (`/gsd-new-milestone`), not here.

## Sequencing

The launch-readiness milestone runs **AFTER the owner-cut aggregate v0.14.0 tag** and
**BEFORE workstream B (v0.13.2 cross-link fidelity, P98)**. Per OD-4 item 3, v0.13.2
(P98–P107) was resequenced to follow launch-readiness; P98's dependency additionally
now includes launch-readiness GREEN.

## Next-session pointer

To author the real launch-readiness milestone, run **`/gsd-new-milestone`** — decompose
the four pillars into phases, mint REQ-IDs, and write the execution plans there. THIS
stub is only the pointer; it is intentionally not that milestone. Do not begin any pillar
work until the owner has cut the v0.14.0 tag.

## Phase-close

This stub's close is a **"scope-artifact-present, zero-implementation"** check: the PLAN
scopes OD-4 (name + file path + line range + four pillars), marks DO-NOT-START, and
contains no code, no gates, and no catalog row. Per the ROADMAP P112 success criteria,
**no verifier-subagent dispatch is required** — a lightweight owner acknowledgment that
the stub correctly says "do not start" suffices.
