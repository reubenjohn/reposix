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

## Wave plan

Strictly sequential — three waves, one plan each. POC findings inform
attach design; scaffold lands before tests; tests must observe the
implementation from 79-02.

| Wave | Plans | Cargo? | File overlap         | Notes                                                                                              |
|------|-------|--------|----------------------|----------------------------------------------------------------------------------------------------|
| 1    | 79-01 | YES*   | none with 79-02/03   | POC may compile small scratch Rust modules (gix-walking proof, frontmatter-id matching proof)      |
| 2    | 79-02 | YES    | none with 79-01      | scaffold in `crates/reposix-cli/` + reconciliation module in `crates/reposix-cache/`               |
| 3    | 79-03 | YES    | overlap with 79-02   | adds tests + 1 new line in 79-02-touched files (`attach.rs` + `reconciliation.rs` may be edited if a tested behavior reveals a small fix); CLAUDE.md edit |

\* POC may use shell + small-scratch-Rust mix; if Rust scratch is limited to
a single new throwaway crate at `research/v0.13.0-dvcs/poc/scratch/Cargo.toml`
(NOT a workspace member), it does not contend with workspace cargo locks
that 79-02 will later acquire.

Per CLAUDE.md "Build memory budget" the executor for each wave holds the
cargo lock for that wave's compilation. Sequential by-wave honors this
without ambiguity.

`files_modified` overlap audit (per gsd-planner same-wave-zero-overlap rule):

| Plan  | Files                                                                                                                                                                                                                                                                                                                                                              |
|-------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| 79-01 | `research/v0.13.0-dvcs/poc/run.sh`, `research/v0.13.0-dvcs/poc/POC-FINDINGS.md`, `research/v0.13.0-dvcs/poc/{scratch/**,fixtures/**}` (all under throwaway path); CLAUDE.md edit OPTIONAL (only if a permanent learning surfaces; default is NO CLAUDE.md edit since POC is throwaway)                                                                              |
| 79-02 | `crates/reposix-cli/src/main.rs`, `crates/reposix-cli/src/lib.rs`, `crates/reposix-cli/src/attach.rs` (new), `crates/reposix-cache/src/lib.rs`, `crates/reposix-cache/src/reconciliation.rs` (new), `crates/reposix-cache/src/db.rs`, `crates/reposix-cache/src/cache.rs` (3 new public APIs + audit hook), `crates/reposix-cache/src/builder.rs` (audit-call site if needed), `quality/catalogs/agent-ux.json`, `quality/gates/agent-ux/reposix-attach.sh` (new) |
| 79-03 | `crates/reposix-cli/tests/attach.rs` (new), `crates/reposix-cache/src/cache.rs` (one-line `materialized_blob_is_tainted` integration test wiring may belong in `tests/`; see plan), `crates/reposix-cli/src/attach.rs` OR `crates/reposix-cache/src/reconciliation.rs` (small fix-forward if a test surfaces a defect; defaults to NO edit), `quality/catalogs/agent-ux.json` (status FAIL → PASS rewritten by runner), `CLAUDE.md`                                                                  |

Wave 1 ↔ Wave 2 file overlap: NONE (POC under `research/`, scaffold under `crates/`).

Wave 2 ↔ Wave 3 file overlap: ALLOWED (sequential dependency). Wave 3
edits `quality/catalogs/agent-ux.json` (the runner rewrites the row's
`status` field) AND may patch `crates/reposix-cli/src/attach.rs` or
`crates/reposix-cache/src/reconciliation.rs` if integration tests surface
a defect — but only as fix-forward; the SCAFFOLD lands in 79-02 and is
considered complete then.

Optional: if 79-01 surfaces a permanent learning that warrants a CLAUDE.md
edit (e.g., a new convention paragraph), the orchestrator folds that edit
into 79-03's CLAUDE.md update — keeps CLAUDE.md edits to one wave.

## Reframe of DVCS-ATTACH-04 (per checker B2)

**Original sketch acceptance** read "all materialized blobs are wrapped in
`Tainted<Vec<u8>>`" — but `Cache::build_from` per its contract does NOT
materialize blobs (lazy by design; only `Cache::read_blob` materializes,
and only when git invokes the cache via the helper). So the original
"compile-time guarantee" test in T03 was vacuously satisfied (nothing was
ever materialized during attach itself).

**Reframed acceptance for DVCS-ATTACH-04 (resolved via path b):**

> *"The cache materialization API used by `attach` (the `Cache::read_blob`
> path that git invokes lazily) returns `Tainted<Vec<u8>>` per OP-2. Verified
> by BOTH (1) a static type-system assertion in a unit test that imports
> `reposix_core::Tainted` and asserts the function signature; AND (2) an
> integration test that exercises `attach` then forces a single blob
> materialization via the helper-equivalent path and asserts the bytes are
> tainted."*

**Architectural rationale:** blobs are lazy. The `Tainted` contract belongs
to `read_blob`, not to `attach` itself. The integration test forces ONE
materialization (cheapest possible exercise of the lazy path) so the
runtime assertion has a concrete byte stream to grade.

This reframe:
- Closes the vacuity gap.
- Costs ≤ 30 lines of test code (type assertion in 79-02 T03; integration
  test in 79-03 T02).
- Does NOT require changing `Cache::read_blob` (already returns
  `Tainted<Vec<u8>>` at `crates/reposix-cache/src/builder.rs:436`).

The orchestrator updates `.planning/REQUIREMENTS.md` DVCS-ATTACH-04 row to
reflect this reframed acceptance BEFORE the verifier subagent grades P79.
This is an orchestrator-level edit (top-level coordinator action), not a
plan task.

## New GOOD-TO-HAVES entry (per checker B3)

The original 79-02 T01 cited `quality/PROTOCOL.md` § Principle A
("Subagents propose; tools validate and mint") as the catalog-mint path,
but immediately admitted that `reposix-quality bind` only supports the
`docs-alignment` dimension — leaving agent-ux dim mints as hand-edited
JSON, which violates Principle A while citing it.

**Resolution: option (b) — acknowledge the gap.** The 79-02 T01 task
hand-edits `quality/catalogs/agent-ux.json` with an explicit annotation
that this is a documented gap (NOT a Principle A application), and the
orchestrator files a GOOD-TO-HAVES entry tracking the bind-extension work
for a future polish slot (P88 or later).

**GOOD-TO-HAVES entry to be filed at
`.planning/milestones/v0.13.0-phases/GOOD-TO-HAVES.md` (created lazily
when first entry surfaces):**

> ### GOOD-TO-HAVES-01 — `reposix-quality bind` agent-ux dim support
>
> - **Discovered by:** Phase 79 plan-checker (B3).
> - **Size:** S (~30-50 lines of Rust in `crates/reposix-quality/src/`).
> - **Impact:** consistency — closes Principle A bypass for agent-ux dim
>   mints. Today, agent-ux + perf + security + agent-ux dims are
>   hand-edited JSON; only docs-alignment dim has `bind` verb support.
> - **Why deferred:** keeping P79 scope tight; mint extension is its own
>   work that fits cleanly in a polish slot.
> - **Eager-resolution gate:** would have eaten >1h into P79's scope and
>   introduced a new dependency (the quality binary's verb dispatch path).
>   Defers cleanly to P88 (good-to-haves polish slot of v0.13.0) per OP-8.
> - **Acceptance:** `reposix-quality bind --dimension agent-ux <id> --command <cmd> --verifier <path>`
>   exists and mints the row with required schema fields. Unit test in
>   `crates/reposix-quality/`. Once shipped, the existing hand-edited
>   agent-ux row from P79 can be retroactively re-bound via the verb
>   (no row content change; provenance trail tightens).

The 79-02 T01 task does NOT cite Principle A; instead it annotates:
"Hand-edit per documented gap; tracked in GOOD-TO-HAVES-01 until
`reposix-quality bind` supports agent-ux dim."

The orchestrator (top-level) files the GOOD-TO-HAVES.md entry as part of
the phase-close ritual — this is NOT a plan task.

## POC findings → planner re-engagement protocol

The POC's whole purpose is to surface decisions the architecture sketch
didn't anticipate. The orchestrator MUST treat `POC-FINDINGS.md` as a
potential plan-revision input:

1. **At 79-01 close** (after POC commits land + push), orchestrator reads
   `research/v0.13.0-dvcs/poc/POC-FINDINGS.md` from origin/main.
2. **Decision gate** — orchestrator routes to one of:
   - **a) No revision needed.** Findings are informational only (e.g.,
     "frontmatter parse via `serde_yaml::from_str` is straightforward;
     no algorithm change"). Proceed directly to 79-02 → 79-03 execution
     as-drafted.
   - **b) In-place revision.** Findings warrant a tweak to 79-02 or 79-03
     (e.g., "reconciliation needs a `--orphan-policy=defer` option not yet
     enumerated"). Orchestrator dispatches a planner subagent with
     `<revision_context>` containing the POC findings + the existing
     plan(s). Planner produces a revised PLAN.md (NEW commit; no
     `--amend`; commit message cites the finding source).
   - **c) Phase split.** Findings reveal scope > combined 79-02 + 79-03
     budget (e.g., "reconciliation cases need >5 distinct rules" — exact
     early-signal trigger from `vision-and-mental-model.md` § "Risks").
     Orchestrator surfaces split options to owner: continue P79 with
     reduced scope + defer extras to P79.5 or P88, OR split inline. Owner
     approves a path.
3. **Artifact contract** — `POC-FINDINGS.md` MUST contain a top-level
   subsection `## Implications for 79-02` (the section name retained from
   the original draft for continuity; treat as "Implications for 79-02
   AND/OR 79-03") listing 0-N items, each tagged `INFO | REVISE | SPLIT`.
   Orchestrator routes by the highest-severity tag present.

This checkpoint is BLOCKING — 79-02 execution does not begin until the
checkpoint is resolved. The check costs ~5 minutes of orchestrator
read-then-decide; saves the >5h cost of executing a stale plan.

## Phase-close protocol

Per CLAUDE.md OP-7 + REQUIREMENTS.md § "Recurring success criteria across
every v0.13.0 phase":

1. **All commits pushed.** Each plan terminates with `git push origin main`
   (per CLAUDE.md "Push cadence — per-phase, codified 2026-04-30, closes
   backlog 999.4"). 79-01 pushes after the POC + FINDINGS land; 79-02
   pushes after the scaffold lands; 79-03 pushes after tests + docs +
   catalog-flip land. Pre-push gate-passing is part of each plan's close
   criterion.
2. **Pre-push gate GREEN** for each plan's push. If pre-push BLOCKS:
   treat as plan-internal failure (fix, NEW commit, re-push). NO
   `--no-verify` per CLAUDE.md git safety protocol.
3. **POC-FINDINGS checkpoint** between 79-01 and 79-02 (see above).
4. **Verifier subagent dispatched.** AFTER 79-03 pushes (i.e., after Wave
   3 completes — NOT after each individual plan), the orchestrator
   dispatches an unbiased verifier subagent per `quality/PROTOCOL.md` §
   "Verifier subagent prompt template" (verbatim copy). The subagent
   grades ALL P79 catalog rows from artifacts with zero session context.
5. **Verdict at `quality/reports/verdicts/p79/VERDICT.md`.** Format per
   `quality/PROTOCOL.md`. Phase loops back if verdict is RED.
6. **STATE.md cursor advanced.** Update `.planning/STATE.md` Current
   Position from "Phase 79 in flight" → "Phase 79 SHIPPED 2026-MM-DD"
   (commit SHA cited). Update `progress` block: `completed_phases: 2`,
   `total_plans: 7`, `completed_plans: 6`, `percent: ~17`.
7. **CLAUDE.md updated in 79-03.** 79-03's terminal task updates §
   "Commands you'll actually use" to add `reposix attach <backend>::<project>`
   example alongside the existing `reposix init` example, and adds a brief
   mention of the cache reconciliation table convention. 79-01 does NOT
   update CLAUDE.md (POC is throwaway research). 79-02 does NOT update
   CLAUDE.md (scaffold without behavior coverage isn't ready for the
   commands section).
8. **GOOD-TO-HAVES.md filed.** Orchestrator (top-level) files the
   GOOD-TO-HAVES-01 entry per § "New GOOD-TO-HAVES entry" above. NOT a
   plan task.
9. **REQUIREMENTS.md DVCS-ATTACH-04 reframed.** Orchestrator updates
   the requirement row per § "Reframe of DVCS-ATTACH-04" above BEFORE
   the verifier subagent dispatches. NOT a plan task.

## Risks + mitigations

| Risk                                                                                            | Likelihood | Mitigation                                                                                                                                                                                                                                                                                                                                                                          |
|-------------------------------------------------------------------------------------------------|------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| **POC reveals an algorithm-shape decision the sketch didn't anticipate** (LIKELY by design)     | HIGH       | This is the POC's whole purpose. `POC-FINDINGS.md` § "Implications for 79-02" tags each finding INFO/REVISE/SPLIT; orchestrator re-engages planner if any REVISE/SPLIT tag fires (see "POC findings → planner re-engagement"). NOT a surprises-intake item — surfaced findings are the deliverable, not a defect.                                                                  |
| **Reconciliation cases need >5 distinct rules** (early-signal trigger from vision-and-mental-model "Risks") | MEDIUM     | If POC discovers a 6th rule (e.g., "file with `id` matching a record but body diverges → conflict"), orchestrator routes to "Phase split" or scopes 79-02/03 to the 5 sketch-named cases + defers extras. Document the rule in POC-FINDINGS; do NOT silently add to the implementation.                                                                                            |
| **Cache-path-derivation collision** (Q1.1 — derive from SoT URL, not from origin)              | LOW        | Architecture sketch decision Q1.1 is already DECIDED in `decisions.md`: cache path derives from SoT URL passed to `attach`. POC verifies via assertion that two attached checkouts in different working dirs but same SoT spec resolve to the SAME `resolve_cache_path()`. 79-02 task explicitly cites Q1.1.                                                                          |
| **Reconciliation table schema migration** — cache crate needs a new table or column            | MEDIUM     | 79-02 T03 lands the schema migration in `crates/reposix-cache/src/db.rs` BEFORE the reconciliation walk lands; one task per concern. New table `cache_reconciliation` (record_id INTEGER PRIMARY KEY, oid TEXT, local_path TEXT, attached_at TEXT) added via `CREATE TABLE IF NOT EXISTS` (idempotent on re-attach). No backfill needed (zero rows pre-P79).                       |
| **Frontmatter `id` field on files with malformed YAML**                                        | MEDIUM     | Reconciliation walks every `*.md` under HEAD; for each file, `frontmatter::parse()` from `crates/reposix-core/src/record.rs:160` is called. Parse failure for a file MUST log a warning + skip, NOT abort attach (per architecture-sketch reconciliation table row 3 "no `id` field — warn; skip"). 79-03 T01 has explicit malformed-YAML test fixture.                            |
| **Duplicate-id collision** (architecture-sketch reconciliation row 4)                          | LOW        | Hard error per the sketch — attach exits non-zero with explicit message naming both file paths. 79-03 T01 has the duplicate-id fixture; assertion is `cmd.failure().stderr(predicates::str::contains("duplicate id"))` shape.                                                                                                                                                       |
| **POC time-budget overrun** (>2 days)                                                          | MEDIUM     | Per CARRY-FORWARD POC-DVCS-01: surface as SURPRISES-INTAKE candidate before continuing. 79-01 T05 has explicit time-check: orchestrator checks elapsed at 4hr / 8hr / 16hr marks. >16h triggers SURPRISES; >32h triggers checkpoint:human-action.                                                                                                                                  |
| **Cache memory pressure during attach materialization** (large spaces — TokenWorld scale)       | LOW        | Materialization stays lazy per existing `Cache::read_blob` contract — attach only writes tree OIDs, NOT blobs (consistent with `Cache::build_from`). The reconciliation walk reads HEAD blobs via gix; gix is streaming. No buffering of full tree. Documented in 79-02 T03 + cited in CLAUDE.md update (79-03).                                                                    |
| **Re-attach with different SoT silently switches** (Q1.2 — must REJECT)                        | LOW        | 79-03 T02 has explicit test: attach SoT1 → attach SoT2 → expect non-zero exit + clear error. Q1.2 decision is in `decisions.md`. Implementation reads existing `<reposix-remote-name>` config; if present and SoT differs, REJECT.                                                                                                                                                  |
| **DVCS-ATTACH-04 vacuity** (checker B2)                                                        | RESOLVED   | Acceptance reframed (see § "Reframe of DVCS-ATTACH-04"). Type-system assertion in 79-02 T03 + integration test forcing one materialization in 79-03 T02. No longer vacuous.                                                                                                                                                                                                          |
| **Principle A bypass via hand-edit** (checker B3)                                              | RESOLVED   | Hand-edit explicitly annotated as documented gap; GOOD-TO-HAVES-01 filed for the future bind-verb extension. 79-02 T01 no longer cites Principle A on the hand-edit step (see § "New GOOD-TO-HAVES entry").                                                                                                                                                                          |
| **Cargo memory pressure** (load-bearing CLAUDE.md rule)                                        | LOW        | Strict serial cargo across all three plans. Per-crate fallback (`cargo nextest run -p reposix-cli`, `-p reposix-cache`) is documented in each plan. 79-02 + 79-03 each stay ≤ 4 cargo-heavy tasks per checker B1.                                                                                                                                                                    |
| **Pre-push hook BLOCKs on a pre-existing drift unrelated to P79**                              | LOW        | Per CLAUDE.md § "Push cadence — per-phase": treat as phase-internal failure. Diagnose, fix, NEW commit (NEVER amend), re-push. Do NOT bypass with `--no-verify`.                                                                                                                                                                                                                       |
| **Walker schema drift from P78 surfaces in P79 docs-alignment**                                | LOW        | P78 78-03 stabilized `Row::source_hashes` parallel-array. New attach-related catalog rows P79 lands use the new schema natively. No migration delta needed.                                                                                                                                                                                                                          |

## +2 reservation: out-of-scope candidates

Initialize `.planning/milestones/v0.13.0-phases/SURPRISES-INTAKE.md` and
`GOOD-TO-HAVES.md` lazily — only when an entry surfaces during a plan's
execution. P78 closed without surfacing entries; **P79 surfaces ONE
GOOD-TO-HAVES entry pre-execution** (GOOD-TO-HAVES-01, see § above).

Anticipated candidates the plans flag (per OP-8):

- **MEDIUM** — POC time overrun (79-01 T05 fail mode). >2d → SURPRISES candidate.
- **MEDIUM** — Reconciliation rule count exceeds 5 (79-01 fail mode). Routes to 79-02 plan revision; if scope expansion is unmanageable, SPLIT (orchestrator decides).
- **LOW** — `--orphan-policy` option needs additional values not enumerated in sketch. Eager-resolve in 79-02 if < 30 min; else GOOD-TO-HAVES.
- **LOW** — `reposix detach` referenced in Q1.2 error message but not implemented. If error message is acceptable without a working `detach` command, file as GOOD-TO-HAVES; otherwise SURPRISES.
- **LOW** — POC reveals that `extensions.partialClone=<reposix-remote-name>` clobbers a pre-existing `extensions.partialClone=origin` set by a prior `reposix init`. Per Q1.3 decision: idempotent re-attach refreshes cache state — confirm clobber is the right semantic in POC; if not, file as REVISE.
- **PRE-EXECUTION** — GOOD-TO-HAVES-01 (already filed): `reposix-quality bind` agent-ux dim support.

Items NOT in scope for P79 (deferred per the v0.13.0 ROADMAP):

- Mirror-lag refs (P80). Do not pre-stage `refs/mirrors/...` write code.
- Bus remote URL parser / prechecks / writes (P82+). Bus-remote *shape* may be exercised in POC for findings-gathering only; production implementation defers.
- L1 perf migration (P81). Do not touch the existing `list_records` walk.
- Webhook-driven sync (P84). Out of scope.
- DVCS docs (P85). Out of scope; 79-03 only updates CLAUDE.md, not `docs/`.
- L2/L3 cache-desync hardening (deferred to v0.14.0 per architecture-sketch).
- Multi-SoT attach (Q1.2 — explicitly REJECTED in 79-02; v0.14.0 origin-of-truth scope).

## Subagent delegation

Per CLAUDE.md "Subagent delegation rules" + the gsd-planner spec
"aggressive subagent delegation":

| Plan / Task                              | Delegation                                                                                                                                                                                                                                          |
|------------------------------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| 79-01 T01 (POC scaffold + fixtures)      | `gsd-executor` — small shell + Rust scratch; holds cargo lock for any Rust compile.                                                                                                                                                                |
| 79-01 T02-T04 (POC three-paths run)      | Same 79-01 executor (continues sequentially). Time-bounded — orchestrator pings at 4hr / 8hr / 16hr marks.                                                                                                                                       |
| 79-01 T05 (FINDINGS + push)              | Same 79-01 executor (terminal task).                                                                                                                                                                                                              |
| **POC-FINDINGS checkpoint**              | Orchestrator (top-level) reads FINDINGS, decides routing. If REVISE/SPLIT tag fires, dispatches `gsd-planner` with `<revision_context>` for an in-place 79-02 (or 79-03) plan revision (NEW commit; NO amend).                                    |
| 79-02 T01 (catalog row + verifier)       | `gsd-executor` — catalog-first commit; **hand-edits agent-ux JSON per documented gap (NOT Principle A)**; verifier shell ships in same atomic commit.                                                                                            |
| 79-02 T02 (subcommand scaffold)          | Same 79-02 executor. Cargo lock held for the cli crate.                                                                                                                                                                                          |
| 79-02 T03 (cache reconciliation module + new APIs + audit hook + Tainted type assertion + push) | Same 79-02 executor (terminal task). Adds `Cache::list_record_ids` / `find_oid_for_record` / `connection_mut` / `log_attach_walk` (confirmed not-yet-extant via grep). |
| 79-03 T01 (5 reconciliation integration tests) | `gsd-executor` (Wave 3) — holds cargo lock; runs `cargo nextest run -p reposix-cli --tests attach`.                                                                                                                                              |
| 79-03 T02 (re-attach idempotency + reject + Tainted-materialization integration test + audit-row test) | Same 79-03 executor.                                                                                                                                                                                                                              |
| 79-03 T03 (CLAUDE.md update + catalog flip + push)   | Same 79-03 executor (terminal task).                                                                                                                                                                                                              |
| Phase verifier (P79 close)               | Unbiased subagent dispatched by orchestrator AFTER 79-03 pushes per `quality/PROTOCOL.md` § "Verifier subagent prompt template" (verbatim). Zero session context; grades catalog rows from artifacts.                                            |

Phase verifier subagent's verdict criteria (extracted for P79):

- POC-01: `research/v0.13.0-dvcs/poc/run.sh` exits 0 against the simulator; `POC-FINDINGS.md` exists with `## Implications for 79-02` subsection; three integration paths (a/b/c) each have a transcript-style log under `research/v0.13.0-dvcs/poc/logs/` or inline in FINDINGS; time-budget annotation present.
- DVCS-ATTACH-01: `reposix attach` subcommand exists in `--help` output; integration test `attach_against_vanilla_clone_sets_partial_clone` passes; cache appears at `resolve_cache_path(<sot-backend>, <sot-project>)`; remote URL written matches `reposix::<sot-spec>` (or `?mirror=` form when default `--bus`).
- DVCS-ATTACH-02: 5 reconciliation test cases pass — `attach_match_records_by_id`, `attach_warns_on_backend_deleted`, `attach_skips_no_id_files`, `attach_errors_on_duplicate_id`, `attach_marks_mirror_lag_for_next_fetch`.
- DVCS-ATTACH-03: 2 tests pass — `re_attach_same_sot_is_idempotent`, `re_attach_different_sot_is_rejected`.
- DVCS-ATTACH-04 (REFRAMED): (1) static type-system assertion test compiles + passes — proves `Cache::read_blob` returns `Tainted<Vec<u8>>`; (2) integration test `attach_then_read_blob_returns_tainted` passes — exercises one materialization via `Cache::read_blob` after attach and asserts the bytes are `Tainted<Vec<u8>>`.
- New catalog row `agent-ux/reposix-attach-against-vanilla-clone` is bound (hand-edit per documented gap GOOD-TO-HAVES-01) with `kind: mechanical`, `cadence: pre-pr`, verifier exits 0; status PASS after 79-03 T03.
- OP-3: cache `audit_events_cache` table contains a row with `event_type = 'attach_walk'` (or whatever `Cache::log_attach_walk` writes) for the integration-test attach run. **Unconditional — no deferral pre-authorized.**
- Recurring (per phase): catalog-first ordering preserved (79-02 T01 commits catalog rows BEFORE T02-T03 implementation); per-plan push completed; verdict file at `quality/reports/verdicts/p79/VERDICT.md`; CLAUDE.md updated in 79-03.

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
