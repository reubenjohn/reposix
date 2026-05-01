# v0.13.0 — Open-question decisions

> **Audience.** The planner running `/gsd-plan-phase` for v0.13.0 phases. Read AFTER `vision-and-mental-model.md` + `architecture-sketch.md` + `kickoff-recommendations.md`.
>
> **Status.** Owner-ratified 2026-04-30 at v0.13.0 kickoff. Per kickoff-rec #1: each "Open question" from `architecture-sketch.md` is closed below either as a **DECIDED** call (carry into PLAN.md as ground truth) or **DEFERRED** with a named trigger that forces resolution inside the named phase.
>
> Decisions follow the sketch's "Probably X" guidance verbatim unless flagged otherwise.

## Phase-N (`reposix attach`) decisions

### Q1.1 — Cache path for attached checkout
**DECIDED:** Cache path derives from the **SoT URL passed to `attach`**, not from `remote.origin.url`. The standard derivation rule (`~/.cache/reposix/<host>/<project>` or equivalent) applies to `<sot-spec>`, not to whatever the local `origin` happens to be (which in the DVCS topology is the GH mirror). Document this contract explicitly in `attach`'s help text and in `docs/concepts/dvcs-topology.md` so `reposix sync`, `reposix list`, etc. find the same cache.

### Q1.2 — Re-attach with different SoT
**DECIDED:** **Reject.** Multi-SoT is the v0.14.0 origin-of-truth question. Error message: *"working tree already attached to <existing-sot>; multi-SoT not supported in v0.13.0. Run `reposix detach` first or pick the existing SoT."* (Tracks `reposix detach` as a follow-on; if not in v0.13.0 P-N scope, file as a v0.13.0 GOOD-TO-HAVE.)

### Q1.3 — Already-`reposix init`-ed checkouts
**DECIDED:** **Idempotent.** `attach` detects `extensions.partialClone` and proceeds anyway — re-attach refreshes cache state against the current backend. No special-casing of init-vs-attach origins; the operation is "make this checkout reposix-equipped against this SoT," whether or not it already was.

## Phase-N+1 (mirror-lag refs) decisions

### Q2.1 — `refs/mirrors/...` vs `refs/notes/reposix/...`
**DECIDED:** `refs/mirrors/...`. Better discoverability for plain-git users (`git for-each-ref refs/mirrors/`). Note-based metadata is filed as a v0.14.0 carry-forward if a tooling reason emerges.

### Q2.2 — Refs during the SoT-edit-to-webhook-sync gap
**DECIDED (clarification):** Nothing writes the refs in the gap window. The staleness window the refs measure **is** the gap. Document loudly in `docs/concepts/dvcs-topology.md` and `docs/guides/dvcs-mirror-setup.md`: *"refs/mirrors/confluence-synced-at is the timestamp the mirror last caught up to confluence — it is NOT a 'current SoT state' marker."*

### Q2.3 — Bus updates `refs/mirrors/confluence-head` only or both refs
**DECIDED:** Both. Webhook becomes a no-op refresh when the bus already touched them. Consistency > optimization.

## Phase-N+2 / N+3 (bus remote) decisions

### Q3.1 — Performance: today's `list_records` walk on every push
**DECIDED:** Ship **L1 migration in v0.13.0** per the sketch's strong recommendation. Phase decomposition reserves a dedicated phase (likely N+2 or N+3, before bus-write-fan-out) for replacing the unconditional `list_records` walk with `list_changed_since`-based conflict detection, plus a `reposix sync --reconcile` escape hatch for cache desync recovery. **L2 + L3 hardening defer to v0.14.0** per `architecture-sketch.md` § "Performance subtlety". Owner ratifies the scope expansion.

**Trigger for re-evaluation:** if v0.13.0 phase budget is over-tight by phase N+2, demote L1 to GOOD-TO-HAVE and document the inherited inefficiency per option (b) in the sketch — but only with explicit owner approval; the default is ship L1 inline.

### Q3.2 — 30s TTL cache for cheap GH precheck
**DECIDED:** **Defer.** Measure first; add only if push latency is hot. Filed as v0.13.0 GOOD-TO-HAVE candidate.

### Q3.3 — Bus URL scheme
**DECIDED:** `reposix::<sot-spec>?mirror=<mirror-url>`. Plays well with existing URL parsing; URL-safe in all contexts. Plus form (`+`) explicitly rejected. Single-word `bus` keyword in scheme reserved for v0.13.0+ (`reposix::bus://...`) is dropped — query-param form is the syntax.

**Trigger for revisit:** if a third bus endpoint enters scope (out-of-scope for v0.13.0 per vision doc), revisit the syntax.

### Q3.4 — Bus handles FETCH
**DECIDED:** **PUSH only** for v0.13.0. Fetch goes to the SoT directly via existing single-backend code path. Bus is a write-fan-out construct.

### Q3.5 — Bus URL with no local `git remote` for the mirror
**DECIDED:** **Fail with hint.** *"configure the mirror remote first: `git remote add <name> <mirror-url>`."* No auto-mutation of user's git config.

### Q3.6 — Retry on transient mirror-write failures (step 7 of bus algo)
**DECIDED:** **Surface, no helper-side retry.** User retries the whole push. Helper-side retry would hide signal and complicate the audit trail. The audit row records the partial failure; next push catches up the mirror.

## Phase-N+4 (webhook sync) decisions

### Q4.1 — Cron fallback frequency
**DEFERRED to Phase N+4 implementation.** Make the workflow `vars`-configurable; default to `*/30` minutes. Trigger: real-backend test against TokenWorld validates that 30-min cron + webhook combination doesn't double-write. If the dual-trigger creates spurious force-with-lease failures, tune.

### Q4.2 — Backends without webhooks
**DECIDED:** Document in `docs/guides/dvcs-mirror-setup.md` § "Backends without webhooks": cron path becomes the only sync mechanism; no v0.13.0 implementation work required (the workflow already supports cron-only mode by omitting the `repository_dispatch` trigger).

### Q4.3 — First-run handling (no existing mirror refs)
**DEFERRED to Phase N+4 implementation.** Acceptance criterion in PLAN.md: *"the workflow runs cleanly against an empty GH mirror (no refs/heads/main, no refs/mirrors/...) and populates them on first run."* If the simulator+wiremock combo doesn't cover first-run gracefully, that's a SURPRISES-INTAKE candidate.

## Cross-cutting decisions

### POC scope (kickoff-rec #2)
**RATIFIED 2026-04-30 (owner):** Build POC first. Lives in `research/v0.13.0-dvcs/poc/` as throwaway code. Exercises three integration paths against the simulator: `reposix attach` against deliberately-mangled checkout (with mixed `id`-bearing + `id`-less files), bus-remote push that observes mirror lag, cheap-precheck path that refuses fast on SoT version mismatch. **Acceptance:** the POC ships as a runnable shell script + scratch Rust crate; it is NOT the v0.13.0 implementation. Findings feed back into the planner via a POC-FINDINGS.md sibling. Time budget: ~1 day; if exceeding 2 days, surface as a SURPRISES-INTAKE candidate before continuing.

### Push cadence (kickoff-rec #3)
**RESOLVED 2026-04-30:** per-phase push, codified in `CLAUDE.md` § "GSD workflow" → "Push cadence — per-phase". Closes backlog 999.4.

### `/gsd-review` (kickoff-rec #4)
**SCHEDULED:** runs after v0.13.0 ROADMAP + first PLAN.md drafted, before execution starts.

### WAIVED structure rows (3 rows expire 2026-05-15)
**RATIFIED 2026-04-30 (owner):** Verifier scripts land in **v0.13.0 P0/P1** (alongside hygiene + attach work, before bus-remote phases). Three rows in `quality/catalogs/freshness-invariants.json` — `no-loose-top-level-planning-audits`, `no-pre-pivot-doc-stubs`, `repo-org-audit-artifact-present`. Each gets a verifier under `quality/gates/structure/` (mirroring docs-alignment dim shape); rows flip from WAIVED → PASS without renewing. Waiver auto-renewal is the failure mode this avoids — leaving them waived would defeat the catalog-first principle (rows defining a green contract whose verifier doesn't exist).

### gix yanked-pin (issues #29, #30 — surfaced 2026-04-30 by CI-monitor subagent)
**RATIFIED 2026-04-30 (owner):** Bump gix off `=0.82.0` (yanked from crates.io 2026-04-28) as **v0.13.0 P0** (before attach + bus work). gix-actor 0.40.1 also yanked. The `=`-pin is load-bearing per `CLAUDE.md` § Tech stack — gix is pre-1.0 and the pin protects against semver surprises. Action: bump to latest non-yanked release, re-pin, re-run cargo workspace check + nextest, update tech-stack note in CLAUDE.md if the version changes. Closes #29 + #30.

**Trigger for re-evaluation:** if the bump introduces breakage in cache materialization or partial-clone paths beyond a one-line API rename, file as v0.13.0 SURPRISES-INTAKE entry; do NOT rollback to the yanked pin.

## Lineage

- `architecture-sketch.md` — open-question source of truth.
- `kickoff-recommendations.md` § "Pre-kickoff checklist" — the items this file closes.
- `CLAUDE.md` § "GSD workflow" → "Push cadence — per-phase" — push cadence codification.
- `.planning/ROADMAP.md` § Backlog 999.4 — RESOLVED row.
