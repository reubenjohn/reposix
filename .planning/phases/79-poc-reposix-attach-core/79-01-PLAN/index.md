---
phase: 79
plan: 01
title: "POC-01 — end-to-end POC of the three v0.13.0 innovations against the simulator"
wave: 1
depends_on: []
requirements: [POC-01]
files_modified:
  - research/v0.13.0-dvcs/poc/run.sh
  - research/v0.13.0-dvcs/poc/POC-FINDINGS.md
  - research/v0.13.0-dvcs/poc/scratch/Cargo.toml
  - research/v0.13.0-dvcs/poc/scratch/src/main.rs
  - research/v0.13.0-dvcs/poc/fixtures/mangled-checkout/**
  - research/v0.13.0-dvcs/poc/logs/**
autonomous: true
mode: standard
---

# Phase 79 Plan 01 — POC of the three innovations (POC-01)

<objective>
Build a throwaway end-to-end POC at `research/v0.13.0-dvcs/poc/` that
exercises three integration paths against the simulator BEFORE the production
`reposix attach` subcommand is designed:

(a) **`reposix attach`-shaped flow** against a deliberately-mangled
    checkout — mixed `id`-bearing + `id`-less files, plus a duplicate-`id`
    case and a deleted-on-backend case. Exercises the reconciliation table
    from `architecture-sketch.md` § "Reconciliation cases" rows 1-5.

(b) **Bus-remote-shaped push** observing mirror lag — SoT writes succeed,
    mirror trailing. Exercises the SoT-first sequencing from `architecture-sketch.md`
    § "Algorithm (export path)" steps 6-8 (without committing to the production
    bus URL parser; we just need to verify that the sequencing is sound and
    surfaces interesting findings).

(c) **Cheap-precheck path** refusing fast on SoT version mismatch — no
    stdin read, no REST writes. Exercises CHEAP PRECHECK B from the same
    section (the one that becomes `list_changed_since`-based after the L1
    migration in P81).

The POC ships with `POC-FINDINGS.md` listing algorithm-shape decisions,
integration friction, and design questions the architecture sketch did not
anticipate. Findings feed directly into 79-02's plan via the orchestrator's
re-engagement protocol (see 79-PLAN-OVERVIEW.md § "POC findings → planner
re-engagement protocol").

**Time budget: ~1 day; if exceeding 2 days, surface as a SURPRISES-INTAKE
candidate before continuing.** Per CARRY-FORWARD POC-DVCS-01 + decisions.md
§ "POC scope".

**Throwaway code only.** This POC lives at `research/v0.13.0-dvcs/poc/`,
NOT inside `crates/`. It is NOT v0.13.0 implementation. Throwaway means:

- No new workspace member (no `crates/<poc>` entry in workspace `Cargo.toml`).
- If a Rust scratch helper is needed, it lives at
  `research/v0.13.0-dvcs/poc/scratch/Cargo.toml` as a STANDALONE crate
  (its own `Cargo.toml`, NOT a workspace member). It can `path = "../../crates/reposix-core"`
  etc. to depend on workspace crates without joining the workspace.
- No catalog rows for the POC itself; the production catalog row lands
  in 79-02.
- The POC's `run.sh` is the success contract — it must exit 0 against
  the simulator; no PR-level pre-push gate is added.
</objective>

<must_haves>
- Directory `research/v0.13.0-dvcs/poc/` exists and is git-tracked.
- File `research/v0.13.0-dvcs/poc/run.sh` exists, executable, exercises the
  three paths end-to-end against the simulator. Exits 0 on success.
- File `research/v0.13.0-dvcs/poc/POC-FINDINGS.md` exists, contains:
  - **Header** — POC scope + simulator version + date.
  - **§ Path (a) — Reconciliation against mangled checkout** — observations,
    surprises, design questions for 79-02.
  - **§ Path (b) — Bus-remote SoT-first sequencing** — observations,
    surprises, design questions for 79-02 / P82 / P83.
  - **§ Path (c) — Cheap-precheck on SoT mismatch** — observations,
    surprises, design questions for 79-02 / P82.
  - **§ Implications for 79-02** — the routing tag block; 0-N items each
    tagged exactly one of `INFO | REVISE | SPLIT`. Highest-severity tag
    drives orchestrator re-engagement (see 79-PLAN-OVERVIEW.md).
  - **§ Time spent** — wall-clock note (e.g., "started 2026-MM-DD HH:MM
    UTC, finished 2026-MM-DD HH:MM UTC, total ~Nh"). If >2 days, the
    SURPRISES-INTAKE entry is also referenced.
- A fixtures directory at `research/v0.13.0-dvcs/poc/fixtures/mangled-checkout/`
  with at least 5 `.md` files exercising the 5 reconciliation cases:
  - `issues/0001.md` with valid `id: 1` matching a backend record (case 1: match).
  - `issues/0099.md` with valid `id: 99` for a record that does NOT exist
    on the backend (case 2: backend-deleted).
  - `notes/freeform.md` with NO `id` frontmatter field (case 3: no-id).
  - `issues/0042-a.md` AND `issues/0042-b.md` both claiming `id: 42` (case 4:
    duplicate-id hard error).
  - Comments in each fixture name which reconciliation case it exercises.
  - The simulator seed file (or test setup in `run.sh`) ensures backend
    record `id: 99` exists in cache state but NOT in the working tree
    (case 5: mirror-lag — record present on backend, absent locally).
- `research/v0.13.0-dvcs/poc/logs/` directory containing at least one
  transcript per path (a/b/c) — captured stdout/stderr from the POC's
  end-to-end runs. These are the verifier's evidence trail.
- If a Rust scratch crate is created, it lives at
  `research/v0.13.0-dvcs/poc/scratch/Cargo.toml` as a STANDALONE crate
  (NOT joining the workspace). The crate name is `reposix-poc-79`.
- The POC does NOT introduce any catalog rows — production rows land in
  79-02.
- POC commits do NOT touch `crates/` (NOT v0.13.0 implementation). Verified
  by `git diff --stat HEAD~N..HEAD -- crates/` returning empty for each
  of this plan's commits.
- Per-phase push: the plan's terminal task pushes to origin/main with
  pre-push GREEN.
- Time annotation in `POC-FINDINGS.md § Time spent` — the wall-clock
  budget signal that triggers SURPRISES if >2d.
</must_haves>

<canonical_refs>
- `.planning/milestones/v0.13.0-phases/CARRY-FORWARD.md` § "POC-DVCS-01" —
  verbatim acceptance criteria.
- `.planning/research/v0.13.0-dvcs/decisions.md` § "POC scope" — owner
  ratification 2026-04-30.
- `.planning/REQUIREMENTS.md` POC-01 — verbatim acceptance.
- `.planning/research/v0.13.0-dvcs/architecture-sketch.md` § "Reconciliation
  cases" (table; the 5 rows the POC exercises).
- `.planning/research/v0.13.0-dvcs/architecture-sketch.md` § "Algorithm
  (export path)" steps 1-9 — the POC's path (b) and (c) shape source.
- `.planning/research/v0.13.0-dvcs/vision-and-mental-model.md` § "Risks and
  how we'll know early" — the >5-rule reconciliation-count early signal.
- `.planning/research/v0.13.0-dvcs/kickoff-recommendations.md` § rec #2 —
  v0.9.0 precedent (saved 3-4 days; ratifies the 1-day budget).
- `crates/reposix-cli/src/init.rs:45-96` — `translate_spec_to_url` — POC
  may call this directly from scratch crate (depends on `reposix-cli`
  crate via local path).
- `crates/reposix-core/src/record.rs:99-200` — `frontmatter::parse` /
  `render` API for the `id`-matching walk in path (a).
- `crates/reposix-cache/src/path.rs:22-38` — `resolve_cache_path` (Q1.1
  contract — derives from SoT, not origin).
- `crates/reposix-sim/` — the simulator the POC runs against. Default
  bind `127.0.0.1:7878`; default seed has records.
- `scripts/dark-factory-test.sh` — precedent for shell-driven end-to-end
  tests against the local cargo workspace; the POC's `run.sh` mirrors
  this shape.
- `CLAUDE.md` § "Build memory budget" — strict serial cargo (only one
  cargo invocation at a time across the POC's run.sh).
- `CLAUDE.md` § "Push cadence — per-phase" — terminal `git push origin
  main` in T05.
- `CLAUDE.md` Operating Principles #1 (Verify against reality) — each
  POC path must produce a real artifact (transcript log) the verifier
  can grade.
- `CLAUDE.md` Operating Principles #4 (Self-improving infrastructure;
  ad-hoc bash is a missing-tool signal) — `run.sh` is a committed named
  script, not a one-shot pipeline. POC scratch Rust if needed becomes
  the named tool.

This plan does not introduce new threat-model surface. All POC traffic
hits `127.0.0.1:7878` (the local simulator). No `<threat_model>` delta
required.
</canonical_refs>

---

## Chapters

- **[T01 — Scaffold POC directory + fixtures + run.sh skeleton](./T01.md)**
- **[T02 — Path (a): reconciliation against mangled checkout](./T02.md)**
- **[T03 — Path (b): bus-remote-shaped push observing mirror lag](./T03.md)**
- **[T04 — Path (c): cheap-precheck refusing fast on SoT version mismatch](./T04.md)**
- **[T05 — Fill out POC-FINDINGS.md + per-phase push (79-01 terminal)](./T05.md)**
