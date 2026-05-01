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
- 🚧 **v0.12.0 Quality Gates** — Phases 56–65 (P56–P64 shipped 2026-04-27/28; **P64 docs-alignment dimension shipped 2026-04-28 — verdict GREEN at `quality/reports/verdicts/p64/VERDICT.md`**; P65 backfill (top-level mode) is the only remaining phase before tag; tag held until P65 ships)

## Phases

## v0.13.0 DVCS over REST (PLANNING)

> **Status:** scoped 2026-04-30. Phases 78–88 derive from `.planning/research/v0.13.0-dvcs/architecture-sketch.md` § "Phase decomposition (sketch)" + 15 ratified open-question decisions in `.planning/research/v0.13.0-dvcs/decisions.md` + 4 carry-forward items in `.planning/milestones/v0.13.0-phases/CARRY-FORWARD.md`. Source-of-truth handover bundle: `.planning/research/v0.13.0-dvcs/{vision-and-mental-model,architecture-sketch,kickoff-recommendations,decisions}.md` + `.planning/milestones/v0.13.0-phases/CARRY-FORWARD.md` + `.planning/research/v0.14.0-observability-and-multi-repo/vision-and-mental-model.md` (so v0.13.0 knows what NOT to absorb).

**Thesis.** Shift from "VCS over REST" (one developer, one backend) to "DVCS over REST" — confluence (or any one issues backend) remains the source of truth, but a plain-git mirror on GitHub becomes the universal-read surface for everyone else. Devs `git clone git@github.com:org/repo.git` with vanilla git (zero reposix install), get all markdown, edit, commit. Install reposix only when they want to write back; `reposix attach` reconciles their existing checkout against the SoT, then `git push` via a bus remote fans out atomically to confluence (SoT-first) and the GH mirror. The litmus test: vanilla-git clone → attach → edit → bus-push → webhook-driven mirror catch-up after a browser-side confluence edit, with conflict detection in both directions.

**Mental model.** Three roles: SoT-holder (reposix-equipped, writes via bus); mirror-only consumer (vanilla git, read-only); round-tripper (reposix-equipped after `attach`, writes via bus). Bus remote: precheck-then-SoT-first-write — cheap network checks (`ls-remote` mirror, `list_changed_since` on SoT) bail before reading stdin; on success, REST-write to SoT then `git push` to mirror; mirror-write failure leaves "mirror lag" recoverable on next push, not data loss. Mirror-lag observability via plain-git refs (`refs/mirrors/confluence-head`, `refs/mirrors/confluence-synced-at`) — vanilla `git fetch` brings them along; `git log` shows staleness.

**Recurring success criteria for EVERY phase (P78–P88)** — these are non-negotiable per project CLAUDE.md Operating Principles + the v0.12.0 autonomous-execution protocol carried into v0.13.0; they are NOT separate REQ-IDs:

1. **Catalog-first.** The phase's FIRST commit writes the catalog rows (the end-state contract) under `quality/catalogs/<file>.json` BEFORE any implementation commit. The verifier subagent grades against catalog rows that already exist.
2. **CLAUDE.md updated in the same PR** (per QG-07). Every phase that introduces a new file, convention, gate, or operational rule revises the relevant CLAUDE.md section in the same PR — not deferred to the milestone close.
3. **Per-phase push** (codified 2026-04-30, closes backlog 999.4): `git push origin main` BEFORE verifier-subagent dispatch. Pre-push gate-passing is part of phase-close criterion. Trivial in-phase chores ride on the terminal push; deferral is not a closure path.
4. **Phase close = unbiased verifier subagent dispatch (OP-7).** Orchestrator dispatches an isolated subagent with zero session context that grades all catalog rows for this phase against artifacts under `quality/reports/verifications/`; verdict written to `quality/reports/verdicts/p<N>/VERDICT.md`; phase does not close on RED.
5. **Eager-resolution preference (OP-8).** Items < 1hr / no new dependency get fixed in the discovering phase; else appended to `.planning/milestones/v0.13.0-phases/SURPRISES-INTAKE.md` (severity + what + why-out-of-scope + sketched resolution) or `GOOD-TO-HAVES.md` (XS/S/M sized polish). The +2 reservation (P87 + P88) drains them.
6. **Simulator-first (OP-1).** All v0.13.0 phases run end-to-end against the simulator. Two simulator instances in one process serve as "confluence-shaped SoT" + "GitHub-shaped mirror" for tests. Real-backend tests (TokenWorld + reubenjohn/reposix) gate the milestone close, not individual phase closes.
7. **Tainted-by-default (OP-2).** Mirror writes carry tainted bytes from the SoT. The GH mirror's frontmatter must preserve `Tainted<T>` semantics — a downstream agent reading from the mirror gets the same trifecta protection as one reading SoT directly. The `attach` cache marks all materialized blobs as tainted.
8. **Audit log non-optional (OP-3).** Every bus-remote push writes audit rows to BOTH tables — cache audit (helper RPC turn) + backend audit (SoT REST mutation). Mirror push writes a cache-audit row noting "mirror lag now zero" or "mirror lag now N." Webhook-driven syncs write cache-audit rows too.

### Phase 78: Pre-DVCS hygiene — gix bump, WAIVED-row verifiers, multi-source walker (v0.13.0)

**Goal:** Land three independent, mutually-parallelizable hygiene items as a single light kickoff phase BEFORE the POC + attach work begins. (a) Bump `gix` off the yanked `=0.82.0` baseline (closes #29 + #30; `gix-actor 0.40.1` also yanked; `=`-pin is load-bearing per CLAUDE.md § Tech stack). (b) Land verifier scripts for the 3 currently-WAIVED structure rows in `quality/catalogs/freshness-invariants.json` (`no-loose-top-level-planning-audits`, `no-pre-pivot-doc-stubs`, `repo-org-audit-artifact-present`) BEFORE waivers expire 2026-05-15 — auto-renewal would defeat the catalog-first principle. (c) Schema-migrate the docs-alignment walker to watch every source on `Source::Multi` rows (carry-forward `MULTI-SOURCE-WATCH-01` from v0.12.1 P75; path-(b) `source_hashes: Vec<String>` parallel-array with `serde(default)` + one-time backfill of the populated 388-row catalog; walker AND-compares per-source hashes; regression tests in `crates/reposix-quality/tests/walk.rs::walk_multi_source_*` exercise non-first-source drift). Operating-principle hooks: **OP-1 simulator-first** (none of these items touch a real backend); **OP-4 self-improving infrastructure** (waiver auto-renewal is the failure mode this avoids; multi-source watch closes a known false-negative window).

**Requirements:** HYGIENE-01, HYGIENE-02, MULTI-SOURCE-WATCH-01

**Depends on:** — (entry-point phase; v0.12.1 SHIPPED 2026-04-30 is the precondition)

**Success criteria:**
1. **gix bump GREEN:** `crates/*/Cargo.toml` `gix = "=0.82.0"` and `gix-actor`-family `=`-pins replaced with the next non-yanked baseline; `cargo check --workspace` GREEN (single invocation per CLAUDE.md "Build memory budget"); `cargo nextest run --workspace` GREEN (per-crate if memory pressure); CLAUDE.md § Tech stack updated to cite the new version; issues #29 + #30 closed with the bump SHA.
2. **3 WAIVED → PASS:** `quality/gates/structure/{no-loose-top-level-planning-audits,no-pre-pivot-doc-stubs,repo-org-audit-artifact-present}.sh` exist (5–30 line TINY shape per docs-alignment dimension precedent); each catalog row in `quality/catalogs/freshness-invariants.json` flips `WAIVED → PASS` (waiver block deleted); tested via `python3 quality/runners/run.py --cadence pre-push`.
3. **Walker watches every Multi source:** `Row::source_hashes: Vec<String>` parallel-array to `source.as_slice()`; `verbs::walk` AND-compares per-source hashes — row enters `STALE_DOCS_DRIFT` on ANY index drift; `verbs::bind` writes/preserves all entries on the parallel array (Single → 1-element vec; Multi append → push the new hash; Multi same-source rebind → refresh that index only); existing single `source_hash` field migrates via `serde(default)` + one-time backfill (read `source_hash` if present, push it into `source_hashes[0]`); regression tests at `crates/reposix-quality/tests/walk.rs::walk_multi_source_*` cover stable / first-drift / non-first-drift / single-rebind-heal cases.
4. Catalog-first commit lands BEFORE implementation: 3 WAIVED rows flip to PASS in the same commit that adds the 3 verifier scripts; `MULTI-SOURCE-WATCH-01` carry-forward row in `quality/catalogs/doc-alignment.json` (or carry-forward catalog) authored as part of the schema migration commit.
5. CLAUDE.md updated in the same PR: Tech stack cites new gix version; the v0.12.1 P75 "MULTI-SOURCE-WATCH-01 deferred" sentence in the P75 H3 subsection updates to "P78 closed via path-(b) walker schema migration"; pre-push hook one-liner unchanged (the runner discovers the new verifiers automatically).
6. Phase close: `git push origin main`, then unbiased verifier subagent grades all catalog rows GREEN; verdict at `quality/reports/verdicts/p78/VERDICT.md`.

**Context anchor:** `.planning/research/v0.13.0-dvcs/decisions.md` § "gix yanked-pin" + § "WAIVED structure rows", `.planning/milestones/v0.13.0-phases/CARRY-FORWARD.md` (`GIX-YANKED-PIN-01`, `WAIVED-STRUCTURE-ROWS-03`, `MULTI-SOURCE-WATCH-01`), `quality/catalogs/freshness-invariants.json` (the 3 WAIVED rows being closed), `crates/reposix-quality/src/lib.rs` + `crates/reposix-quality/tests/walk.rs` (the schema migration + regression-test home), CLAUDE.md § Tech stack + § "v0.12.1 P75" subsection.

**Plans:** TBD

### Phase 79: POC + `reposix attach` core (v0.13.0)

**Goal:** Ship the POC FIRST as throwaway code in `research/v0.13.0-dvcs/poc/` to surface algorithm-shape decisions, integration friction, and design questions the architecture sketch did not anticipate. Then implement `reposix attach <backend>::<project>` as a real subcommand in `crates/reposix-cli/`. The POC's `POC-FINDINGS.md` feeds directly into the implementation's PLAN.md so the implementation absorbs surprises cheaply (v0.9.0 precedent: ~1 day POC saved 3–4 days mid-phase rework). `attach` builds a fresh cache from REST against an existing checkout (created however — vanilla `git clone`, prior `reposix init`, hand-edited tree); reconciles cache OIDs against current `HEAD` by walking the tree and matching files to backend records by `id` in frontmatter; populates the cache reconciliation table; adds remote `reposix::<sot-spec>?mirror=<existing-origin-url>` (or `reposix::<sot-spec>` if `--no-bus`); sets `extensions.partialClone=<reposix-remote-name>` on the new reposix remote (existing `origin` keeps plain-git semantics). Reconciliation produces clear errors per `architecture-sketch.md` § "Reconciliation cases" — match / backend-record-deleted (`--orphan-policy={delete-local,fork-as-new,abort}`) / no-id-frontmatter / duplicate-id (hard error) / mirror-lag. Re-attach with different SoT REJECTED per Q1.2; same-SoT re-attach IDEMPOTENT per Q1.3. All materialized blobs marked `Tainted<Vec<u8>>` per OP-2. Cache path derived from the SoT URL passed to `attach`, NOT from `remote.origin.url` (per Q1.1).

**Requirements:** POC-01, DVCS-ATTACH-01, DVCS-ATTACH-02, DVCS-ATTACH-03, DVCS-ATTACH-04

**Depends on:** P78 GREEN (gix baseline stable; walker schema migration so new attach-specific catalog rows can land cleanly)

**Success criteria:**
1. **POC ships in `research/v0.13.0-dvcs/poc/`:** runnable `run.sh` (or equivalent) exercises three integration paths against the simulator — (a) `attach` against a deliberately-mangled checkout with mixed `id`-bearing + `id`-less files; (b) bus-remote push observing mirror lag (SoT writes succeed, mirror trailing); (c) cheap-precheck path refusing fast on SoT version mismatch. `POC-FINDINGS.md` lists what the POC surfaced (algorithm-shape decisions, integration friction, design questions). Time budget: ~1 day; if exceeding 2 days, surface as SURPRISES-INTAKE entry before continuing. POC code is NOT v0.13.0 implementation.
2. **`reposix attach` exists as a real subcommand:** in CWD with no special prerequisites on how the checkout was created, `reposix attach confluence::SPACE` builds a fresh cache directory at the standard location derived from `<sot-spec>` (NOT from `remote.origin.url`), REST-lists the backend, populates cache OIDs (filenames + tree structure; blobs lazy on first materialize), reconciles by walking current `HEAD` tree and matching files to backend records by `id` in frontmatter, records matches in cache reconciliation table, adds remote `reposix::<sot-spec>?mirror=<existing-origin-url>` (or `reposix::<sot-spec>` if `--no-bus`), sets `extensions.partialClone=<reposix-remote-name>`. Existing `origin` (the GH mirror) keeps plain-git semantics.
3. **Reconciliation cases all tested:** test fixtures cover every row in `architecture-sketch.md` § "Reconciliation cases" — match / backend-record-deleted (warn + skip + offer `--orphan-policy={delete-local,fork-as-new,abort}`) / no-id-frontmatter (warn + skip) / duplicate-id (hard error) / mirror-lag (cache marks for next fetch). Each case has a corresponding integration test in `crates/reposix-cli/tests/attach_*.rs`.
4. **Re-attach semantics:** re-attach with a different SoT spec is REJECTED with the verbatim error from Q1.2 (*"working tree already attached to <existing-sot>; multi-SoT not supported in v0.13.0. Run `reposix detach` first or pick the existing SoT."*). Re-attach with the SAME SoT spec is IDEMPOTENT and refreshes cache state against current backend. Tested in both directions.
5. **Tainted-by-default:** all blobs materialized by `attach` carry `Tainted<Vec<u8>>` per OP-2. Asserted by a unit test on the cache crate's materialization path.
6. **Catalog rows land first:** `quality/catalogs/agent-ux.json` (or new `dvcs-attach.json`) carries rows for the 5 reconciliation cases + the re-attach reject + the idempotent re-attach + the Tainted assertion BEFORE `crates/reposix-cli/src/cmds/attach.rs` is committed. CLAUDE.md updated in same PR (new `attach` subcommand mentioned in § Commands; new dimension/cadence rows referenced if added).
7. Phase close: `git push origin main`; verifier subagent grades all catalog rows GREEN; verdict at `quality/reports/verdicts/p79/VERDICT.md`.

**Context anchor:** `.planning/research/v0.13.0-dvcs/architecture-sketch.md` § "1. `reposix attach <backend>::<project>`" + § "Reconciliation cases" + Q1.1/Q1.2/Q1.3, `.planning/research/v0.13.0-dvcs/decisions.md` § "Phase-N (`reposix attach`) decisions" + § "POC scope", `.planning/milestones/v0.13.0-phases/CARRY-FORWARD.md` § "POC-DVCS-01", `crates/reposix-cli/src/main.rs` (subcommand dispatch site), `crates/reposix-cache/src/lib.rs` (lazy materialization path; reconciliation-table home), `crates/reposix-core/src/tainted.rs` (Tainted<T> contract).

**Plans:** TBD

### Phase 80: Mirror-lag refs (`refs/mirrors/confluence-head`, `confluence-synced-at`) (v0.13.0)

**Goal:** Wire mirror-lag observability into plain-git refs that vanilla `git fetch` brings along, so `git log refs/mirrors/confluence-synced-at` reveals staleness to readers who never installed reposix. Two refs in the `refs/mirrors/...` namespace per Q2.1 (better discoverability than `refs/notes/...`): `confluence-head` records the SHA of the SoT's `main` at last sync; `confluence-synced-at` is an annotated tag with a timestamp message. Helper APIs land in `crates/reposix-cache/`. The existing single-backend push (today's `handle_export`) is wired to update both refs on success — this is the pre-bus integration point so mirror-lag observability is in place BEFORE the bus remote phases ship. Webhook sync also writes both refs (later in P84). Bus push will update both refs (later in P83). Q2.3 ratified: bus updates both refs (consistency over optimization); webhook becomes a no-op refresh when bus already touched them. Reject messages cite the refs in hints — *"your origin (GH mirror) was last synced from confluence at <timestamp> (N minutes ago); run `reposix sync` to update local cache, then `git rebase`."*

**Requirements:** DVCS-MIRROR-REFS-01, DVCS-MIRROR-REFS-02, DVCS-MIRROR-REFS-03

**Depends on:** P79 GREEN (`attach` cache exists; reconciliation table is the home for ref-state lookup; mirror-lag rows in attach reconciliation feed reject-message hints)

**Success criteria:**
1. **Helper APIs in `crates/reposix-cache/`:** `write_mirror_head(sot_sha: gix::ObjectId)` + `write_mirror_synced_at(timestamp: chrono::DateTime<Utc>)` + readers exist; refs are stored under `refs/mirrors/<sot-host>-head` and `refs/mirrors/<sot-host>-synced-at` (sot-host derived from SoT spec; for confluence == `confluence`); annotated tag for `synced-at` carries the timestamp in the message body for plain `git log` readability.
2. **Existing single-backend push updates both refs on success:** `crates/reposix-remote/src/main.rs::handle_export` success path writes a cache-audit row noting "mirror sync written" AND calls the cache helpers to update both refs. On reject path, refs untouched (consistent with cache-untouched-on-reject contract from v0.9.0 ARCH-08).
3. **Vanilla `git fetch` brings refs along:** integration test asserts that after a single-backend push, a fresh `git clone` of the mirror (or `git fetch` from an existing clone) brings both refs into the local repo without any reposix awareness on the reader side. `git for-each-ref refs/mirrors/` returns both refs; `git log refs/mirrors/confluence-synced-at -1` shows the timestamp message.
4. **Reject messages cite the refs in hints:** when conflict detection trips, the helper's stderr error names the two refs with their timestamps — verbatim form per Q2.2 doc clarity contract: *"refs/mirrors/confluence-synced-at is the timestamp the mirror last caught up to confluence — it is NOT a 'current SoT state' marker"*. Reject message names the SoT-edit-vs-mirror-sync gap as the cause when applicable.
5. **Catalog rows land first:** `quality/catalogs/agent-ux.json` (or new `mirror-refs.json`) carries rows for ref-write-on-success + ref-read-by-vanilla-clone + reject-message-cites-refs BEFORE the cache helper commits. CLAUDE.md updated to document the `refs/mirrors/...` namespace convention in the same PR.
6. Phase close: `git push origin main`; verifier subagent grades all catalog rows GREEN; verdict at `quality/reports/verdicts/p80/VERDICT.md`.

**Context anchor:** `.planning/research/v0.13.0-dvcs/architecture-sketch.md` § "2. Mirror-lag observability via plain-git refs" + Q2.1/Q2.2/Q2.3, `.planning/research/v0.13.0-dvcs/decisions.md` § "Phase-N+1 (mirror-lag refs) decisions", `crates/reposix-cache/src/lib.rs` (helper home), `crates/reposix-remote/src/main.rs::handle_export` lines 300–407 (the existing push path being wired).

**Plans:** TBD

### Phase 81: L1 perf migration — `list_changed_since`-based conflict detection (v0.13.0)

**Goal:** Replace today's unconditional `list_records` walk in `crates/reposix-remote/src/main.rs::handle_export` (lines 334–348) with `list_changed_since`-based conflict detection so the bus remote (P82–P83) inherits the cheap path, NOT the expensive one. Per `decisions.md` Q3.1 strong recommendation: ship L1 inline in v0.13.0 BEFORE bus remote ships, otherwise the bus would inherit O(N)-REST-call-per-push and violate the DVCS thesis ("DVCS at the same UX as plain git" — plain git's `git push` does ~3 REST round-trips, bus-remote `git push` doing 100+ would dismiss reposix as a toy). Net REST cost on success path collapses to one call (`list_changed_since`) plus actual REST writes. L1 trades one safety property: today's `list_records` would catch a record that exists on backend but is missing from cache (cache desync from a previous failed sync). With L1, cache is trusted as the prior. Add `reposix sync --reconcile` as the on-demand escape hatch for users who suspect desync — does a full `list_records` walk + cache reconciliation. L2/L3 hardening explicitly defers to v0.14.0 per architecture-sketch § "Performance subtlety". Both single-backend and bus push paths benefit from L1 (single-backend was the pre-existing inefficiency; bus would have inherited it).

**Requirements:** DVCS-PERF-L1-01, DVCS-PERF-L1-02, DVCS-PERF-L1-03

**Depends on:** P80 GREEN (mirror-lag refs in place — the `last_fetched_at` column they expose to the cache is what `list_changed_since(last_fetched_at)` reads)

**Success criteria:**
1. **`list_records` walk replaced:** `crates/reposix-remote/src/main.rs::handle_export` lines 334–348 no longer call `state.backend.list_records(&state.project)` unconditionally. Instead, the precheck uses `backend.list_changed_since(last_fetched_at)` — for each changed record, check against the version in cache's prior tree; mismatch (record changed AND we're trying to push it) → reject with detailed error citing record id + cache version + backend version; no overlap → continue, update cache after step so subsequent pushes have fresh prior. Net REST cost on success path: one call (`list_changed_since`) plus actual REST writes.
2. **`reposix sync --reconcile` exists:** new subcommand in `crates/reposix-cli/` does on-demand full `list_records` walk + cache reconciliation. Documented in `docs/concepts/dvcs-topology.md` (P85) and helper stderr hints (when reject messages suggest cache-desync as a possible cause).
3. **Both push paths benefit:** single-backend push (existing) AND bus push (P82–P83 will inherit) both use the L1 conflict-detection mechanism — no path-specific copies of the algorithm.
4. **Performance regression test:** synthetic test against the simulator with N=200 records seeds the SoT, performs a one-record edit, asserts the push uses `list_changed_since` (1 REST call) NOT `list_records` (would be 4+ paginated calls at page-size 50). Catalog row in `perf-targets` (or new dimension home) records the floor (1 + W where W = REST writes for changed records).
5. **L2/L3 deferral documented inline:** `crates/reposix-remote/src/main.rs` near the new precheck has a comment referencing `architecture-sketch.md` § "Performance subtlety" and the `.planning/research/v0.14.0-observability-and-multi-repo/vision-and-mental-model.md` § "L2/L3 cache-desync hardening" — future agents reading the helper code shouldn't have to rediscover the cost-vs-correctness tradeoff from scratch.
6. **Catalog rows land first:** `quality/catalogs/perf-targets.json` row for the L1 cost floor + `agent-ux.json` row for `sync --reconcile` UX + a docs-alignment row binding the architecture-sketch performance-subtlety prose to the regression test BEFORE the helper edit lands. CLAUDE.md updated in the same PR (Performance section if any; § Commands for `reposix sync --reconcile`).
7. Phase close: `git push origin main`; verifier subagent grades all catalog rows GREEN; verdict at `quality/reports/verdicts/p81/VERDICT.md`.

**Context anchor:** `.planning/research/v0.13.0-dvcs/architecture-sketch.md` § "Performance subtlety: today's `list_records` walk on every push", `.planning/research/v0.13.0-dvcs/decisions.md` § "Q3.1 — Performance" (RATIFIED inline L1), `crates/reposix-remote/src/main.rs::handle_export` lines 334–348 (the call site being replaced), `crates/reposix-core/src/lib.rs` (`BackendConnector::list_changed_since` already shipped in v0.9.0 ARCH-06), `.planning/research/v0.14.0-observability-and-multi-repo/vision-and-mental-model.md` § "L2/L3 cache-desync hardening" (deferral target).

**Plans:** TBD

### Phase 82: Bus remote — URL parser, prechecks, fetch dispatch (v0.13.0)

**Goal:** Stand up the bus remote's read/dispatch surface. URL parser recognizes `reposix::<sot-spec>?mirror=<mirror-url>` per Q3.3 (query-param form; `+`-delimited form explicitly rejected; `bus://` scheme keyword dropped). Single-backend `reposix::<sot-spec>` URLs continue to work via the existing `handle_export` code path. CHEAP PRECHECK A (mirror drift via `git ls-remote`) and CHEAP PRECHECK B (SoT drift via `backend.list_changed_since(last_fetched_at)`) ship with the fail-fast behavior — both prechecks run BEFORE reading stdin, so failure costs nothing. Bus handler does NOT advertise `stateless-connect` for fetch (Q3.4) — read goes to the SoT directly via the existing single-backend code path. This phase is dispatch-only; the WRITE fan-out (steps 4–9 of the bus algorithm) lands in P83. Splitting the bus remote across two phases isolates the riskier write fan-out from the URL/precheck/fetch surface; if bus-write fan-out (P83) reveals scope creep, this phase still ships independently.

**Requirements:** DVCS-BUS-URL-01, DVCS-BUS-PRECHECK-01, DVCS-BUS-PRECHECK-02, DVCS-BUS-FETCH-01

**Depends on:** P81 GREEN (L1 conflict-detection is the substrate of PRECHECK B; bus inherits cheap path, not expensive one)

**Success criteria:**
1. **URL parser:** `reposix::<sot-spec>?mirror=<mirror-url>` parses to a `BusRemote { sot, mirror }` struct in `crates/reposix-remote/`; single-backend `reposix::<sot-spec>` URLs continue to dispatch to the existing `handle_export` code path; `+`-delimited form is REJECTED with a clear error naming the query-param syntax. Tested via golden URL fixtures.
2. **CHEAP PRECHECK A — mirror drift:** bus handler runs `git ls-remote <mirror> main`, compares returned SHA to local `refs/remotes/<mirror>/main`. On drift: emit `error refs/heads/main fetch first` to git on stdout AND hint to stderr (*"your GH mirror has new commits; git fetch <mirror> first"*); bail. NO confluence work done. NO stdin read. Tested with a simulator-backed mirror that drifts mid-test.
3. **CHEAP PRECHECK B — SoT drift:** bus handler runs `backend.list_changed_since(last_fetched_at)` on the SoT. On drift overlapping with the push set: emit `error refs/heads/main fetch first` + hint citing mirror-lag refs (*"confluence has changes since your last fetch; the GH mirror was last synced at <timestamp>; run `git pull --rebase`"*); bail. NO writes done. NO stdin read.
4. **Bus does NOT advertise `stateless-connect`:** fetch goes to the SoT directly via the existing single-backend code path. The helper's `capabilities` advertisement for a `reposix::...?mirror=...` URL omits `stateless-connect` and includes only `export`. Documented in helper stderr help text. Tested by asserting the capability list in a unit test.
5. **No `git remote` for the mirror = clear error:** bus URL referencing a `<mirror-url>` not configured as a local `git remote` fails with the verbatim hint from Q3.5 — *"configure the mirror remote first: `git remote add <name> <mirror-url>`"*. NO auto-mutation of user's git config. Tested.
6. **Catalog rows land first:** `quality/catalogs/agent-ux.json` (or new `bus-remote.json`) carries rows for URL-parse / PRECHECK-A / PRECHECK-B / fetch-not-advertised / no-remote-error BEFORE the helper code commits. CLAUDE.md updated to document the bus URL scheme (Tech stack § or Commands §) in the same PR.
7. Phase close: `git push origin main`; verifier subagent grades all catalog rows GREEN; verdict at `quality/reports/verdicts/p82/VERDICT.md`.

**Context anchor:** `.planning/research/v0.13.0-dvcs/architecture-sketch.md` § "3. Bus remote with cheap-precheck + SoT-first-write" steps 1–3 + Q3.3/Q3.4/Q3.5, `.planning/research/v0.13.0-dvcs/decisions.md` § "Phase-N+2 / N+3 (bus remote) decisions", `crates/reposix-remote/src/main.rs` (URL dispatch + capabilities advertisement site), `crates/reposix-core/src/lib.rs` (`list_changed_since` for PRECHECK B), `crates/reposix-cache/` (mirror-lag-ref readers from P80 for hint composition).

**Plans:** TBD

### Phase 83: Bus remote — write fan-out (SoT-first, mirror-best-effort, fault injection) (v0.13.0)

**Goal:** Implement the riskiest part of the bus remote — the SoT-first-write algorithm with mirror-best-effort fallback and full fault-injection coverage. Per architecture-sketch § "Phase decomposition" *"phase N+3 is the riskiest and may want to split"* — and per `decisions.md` Q3.6 *"surface, no helper-side retry"*. Algorithm: read fast-import stream from stdin and buffer; apply REST writes to SoT (confluence); on success, write audit rows to BOTH tables (cache + backend) and update `last_fetched_at`; then `git push` to GH mirror; on mirror failure, write mirror-lag audit row, update `refs/mirrors/confluence-head` to new SoT SHA but NOT `refs/mirrors/confluence-synced-at` (stays at last successful mirror sync), print warning to stderr, return ok to git (SoT contract satisfied — recoverable on next push). On mirror success, update `refs/mirrors/confluence-synced-at` to now and send `ok refs/heads/main` back to git. NO helper-side retry on transient mirror-write failures (Q3.6) — surface, audit, let user retry. Bus URL with no local `git remote` for the mirror fails per P82 (already shipped). Fault-injection tests cover every documented failure case: kill GH push between confluence-write and ack; kill confluence-write mid-stream; simulate confluence 409 after precheck passed. Each produces correct audit + recoverable state. **If during planning this phase looks > 1 PR's worth of work, split into P83a (write fan-out core) + P83b (fault injection + audit completeness).**

**Requirements:** DVCS-BUS-WRITE-01, DVCS-BUS-WRITE-02, DVCS-BUS-WRITE-03, DVCS-BUS-WRITE-04, DVCS-BUS-WRITE-05, DVCS-BUS-WRITE-06

**Depends on:** P82 GREEN (URL dispatch + prechecks; bus write fan-out runs after PRECHECK A + B succeed and stdin is read)

**Success criteria:**
1. **SoT-first write:** bus handler reads fast-import stream from stdin into a buffer; applies REST writes to confluence; on success writes audit rows to BOTH tables (`audit_events_cache` for the helper RPC turn + `audit_events` for the backend mutation) and updates `last_fetched_at`; on any SoT-write failure, bails — mirror unchanged, no recovery needed.
2. **Mirror-best-effort write:** after SoT-write success, bus handler runs `git push <mirror> main`. On mirror-write success: updates `refs/mirrors/confluence-synced-at` to now, sends `ok refs/heads/main` back to git on stdout. On mirror-write failure: writes a mirror-lag cache-audit row, updates `refs/mirrors/confluence-head` to the new SoT SHA but NOT `confluence-synced-at` (stays at last successful mirror sync), prints warning to stderr (*"SoT push succeeded; mirror push failed (will retry on next push or via webhook sync). Reason: <error>."*), returns `ok refs/heads/main` to git anyway (SoT contract satisfied).
3. **No helper-side retry:** on transient mirror-write failure, helper does NOT retry — surfaces the partial-failure state via the audit row + stderr warning. User retries the whole push. Tested with a simulator-backed mirror that fails the first push, succeeds on retry.
4. **Bus URL with no local `git remote` for the mirror** still fails with P82's verbatim hint (`Q3.5`); regression test ensures P82 + P83 don't silently auto-mutate user's git config.
5. **Fault-injection coverage:** integration tests in `crates/reposix-remote/tests/bus_fault_injection_*.rs` cover (a) kill GH push between confluence-write and ack — assert SoT-state-correct, mirror-lag audit row written, `refs/mirrors/confluence-head` updated, `confluence-synced-at` NOT updated, `ok` returned to git; (b) kill confluence-write mid-stream — assert no SoT writes succeeded, no audit rows, mirror unchanged, helper exits non-zero with clear error; (c) simulate confluence 409 after precheck passed — assert no SoT writes succeeded, mirror unchanged, helper exits non-zero with version-mismatch error citing record id. Each test asserts (audit-row count + state + ref state + helper exit code).
6. **Audit completeness:** every bus-remote push end-state writes audit rows to BOTH tables per OP-3 (cache for the helper RPC turn — accept/reject/partial-fail; backend for the actual SoT REST mutations). The mirror push's outcome is audited as a cache-audit row noting `mirror_lag_delta = 0` (success) or `mirror_lag_delta = N` (lag).
7. **Catalog rows land first:** rows for SoT-first / mirror-best-effort / no-retry / each fault-injection scenario / audit-completeness BEFORE the bus-write commits. CLAUDE.md updated in same PR (§ Threat model audit-log section if mirror-lag audit shape is new).
8. Phase close: `git push origin main`; verifier subagent grades all catalog rows GREEN; verdict at `quality/reports/verdicts/p83/VERDICT.md`.

**Context anchor:** `.planning/research/v0.13.0-dvcs/architecture-sketch.md` § "3. Bus remote with cheap-precheck + SoT-first-write" steps 4–9 + Q3.6, `.planning/research/v0.13.0-dvcs/decisions.md` § "Q3.6 — Retry on transient mirror-write failures", `crates/reposix-remote/src/main.rs::handle_export` (the existing single-backend `handle_export` whose write logic the bus wraps verbatim per architecture-sketch § "Tie-back to existing helper code"), `crates/reposix-cache/` audit-row helpers + mirror-ref helpers from P80, `crates/reposix-core/src/audit.rs` + `crates/reposix-cache/src/audit.rs` (the dual audit-table contract).

**Plans:** TBD

### Phase 84: Webhook-driven mirror sync — GH Action workflow + setup guide (v0.13.0)

**Goal:** Ship the reference GitHub Action workflow that keeps the GH mirror current with confluence-side edits — the pull side of the DVCS topology. Workflow at `.github/workflows/reposix-mirror-sync.yml` triggers on `repository_dispatch` (event type `reposix-mirror-sync`) plus a cron safety net (default `*/30`, configurable via workflow `vars` per Q4.1). Workflow runs `reposix init confluence + git push <mirror>` and updates `refs/mirrors/...` refs. Uses `--force-with-lease` against last known mirror ref so a concurrent bus-push's race doesn't corrupt mirror state. First-run handling (no existing mirror refs, empty mirror) is graceful per Q4.3 — populates refs on first run; verified by sandbox test against TokenWorld. Latency target: < 60s p95 from confluence edit to GH ref update; measured in sandbox during this phase; if p95 > 120s, document the constraint and tune ref semantics. Backends without webhooks (Q4.2): cron path becomes the only sync mechanism — workflow already supports cron-only mode by omitting the `repository_dispatch` trigger; documented in P85's `dvcs-mirror-setup.md`.

**Requirements:** DVCS-WEBHOOK-01, DVCS-WEBHOOK-02, DVCS-WEBHOOK-03, DVCS-WEBHOOK-04

**Depends on:** P80 GREEN (mirror-lag refs exist) + P83 GREEN (bus write semantics defined; webhook is a no-op refresh when bus already touched the refs)

**Success criteria:**
1. **Reference workflow ships:** `.github/workflows/reposix-mirror-sync.yml` (or `docs/guides/dvcs-mirror-setup-template.yml` referenced from the workflow) per architecture-sketch § "Webhook-driven mirror sync". Triggers: `repository_dispatch` (event type `reposix-mirror-sync`) + cron safety net (default `*/30 * * * *`, configurable via workflow `vars`). Steps: checkout, `cargo binstall reposix`, `reposix init confluence::<space> /tmp/sot`, fetch mirror, `git push mirror main --force-with-lease=refs/heads/main:$(git rev-parse mirror/main)`, push the two `refs/mirrors/...` refs.
2. **`--force-with-lease` race protection:** workflow uses `--force-with-lease` against the last known mirror ref so concurrent bus-push doesn't corrupt mirror state. Sandbox test: simulate a bus push landing between the workflow's fetch and its push; assert workflow exits cleanly (lease check fails) without clobbering bus-push commits.
3. **First-run handling:** workflow runs cleanly against an empty GH mirror (no `refs/heads/main`, no `refs/mirrors/...`) and populates them on first run per Q4.3. Verified by sandbox test; if simulator + wiremock combo doesn't cover first-run gracefully, the gap surfaces in `SURPRISES-INTAKE.md` for P87 absorption.
4. **Latency target measured:** sandbox test (TokenWorld + simulator harness) measures end-to-end latency from confluence edit to GH ref update across ≥10 events; p95 captured and reported in `quality/reports/verifications/perf/webhook-latency.json`. Target < 60s p95; if p95 > 120s, P85 docs document the constraint and ref-semantics tuning.
5. **Backends-without-webhooks fallback:** workflow's `repository_dispatch` trigger is removable without breaking; cron-only mode tested in CI. P85's `dvcs-mirror-setup.md` documents this fallback per Q4.2.
6. **Webhook updates `refs/mirrors/...`:** integration test asserts that after a webhook-triggered run, both `refs/mirrors/confluence-head` and `refs/mirrors/confluence-synced-at` are updated; vanilla `git fetch` from a Dev B clone brings the refs along.
7. **Catalog rows land first:** `quality/catalogs/agent-ux.json` (or `webhook-sync.json`) rows for trigger-dispatch / cron-fallback / force-with-lease-race / first-run-empty-mirror / latency-floor / backends-without-webhooks BEFORE the workflow YAML lands. CLAUDE.md updated to document the workflow path + secrets convention in the same PR.
8. Phase close: `git push origin main`; verifier subagent grades all catalog rows GREEN; verdict at `quality/reports/verdicts/p84/VERDICT.md`.
**UI hint**: yes

**Context anchor:** `.planning/research/v0.13.0-dvcs/architecture-sketch.md` § "Webhook-driven mirror sync" + Q4.1/Q4.2/Q4.3, `.planning/research/v0.13.0-dvcs/decisions.md` § "Phase-N+4 (webhook sync) decisions", existing CI workflows in `.github/workflows/` (precedent for secrets + Atlassian creds wiring), `docs/reference/testing-targets.md` (TokenWorld is the sanctioned real-backend webhook sandbox).

**Plans:** TBD

### Phase 85: DVCS docs — topology, mirror setup, troubleshooting, cold-reader pass (v0.13.0)

**Goal:** Make v0.13.0 legible to a cold reader. Three new doc surfaces ship: `docs/concepts/dvcs-topology.md` (three roles + diagram from `vision-and-mental-model.md` + when-to-choose-which-pattern guidance + the verbatim Q2.2 clarification *"`refs/mirrors/confluence-synced-at` is the timestamp the mirror last caught up to confluence — it is NOT a 'current SoT state' marker"*); `docs/guides/dvcs-mirror-setup.md` (walk-through of webhook + Action setup for an owner installing v0.13.0 against a confluence space; backends-without-webhooks fallback per Q4.2; cleanup procedure); troubleshooting matrix entries in `docs/guides/troubleshooting.md` covering bus-remote `fetch first` rejection messages (cite mirror-lag refs as the diagnostic), attach reconciliation warnings, webhook race conditions, cache-desync recovery via `reposix sync --reconcile`. Cold-reader pass via `doc-clarity-review` against a reader who has read only `docs/index.md` + `docs/concepts/mental-model-in-60-seconds.md`. Zero critical-friction findings before milestone close. Banned-words enforced (no FUSE residue; no `partial-clone` / `promisor` / `stateless-connect` / `fast-import` / `protocol-v2` above Layer 3 per v0.10.0 P2 framing principles). Operating-principle hooks: **OP-1 close the feedback loop** — the doc IS the test (cold-reader pass is the verifier); **OP-6 ground truth obsession** — `dvcs-topology.md` is no longer a forgotten todo when shipped, it's the canonical reference for the DVCS thesis.

**Requirements:** DVCS-DOCS-01, DVCS-DOCS-02, DVCS-DOCS-03, DVCS-DOCS-04

**Depends on:** P79 + P80 + P81 + P82 + P83 + P84 ALL GREEN (docs describe their behavior; cannot ship before the surfaces they document do)

**Success criteria:**
1. **`docs/concepts/dvcs-topology.md` ships:** three roles (SoT-holder, mirror-only consumer, round-tripper) explained with the diagram from `vision-and-mental-model.md`; mirror-lag refs explained — explicitly: *"`refs/mirrors/confluence-synced-at` is the timestamp the mirror last caught up to confluence, NOT a 'current SoT state' marker"* per Q2.2; when-to-choose-which-pattern guidance for new readers. Banned-words clean (no `FUSE` / `kernel` residue; no jargon-above-Layer-3 leaks).
2. **`docs/guides/dvcs-mirror-setup.md` ships:** end-to-end walk-through of webhook + GitHub Action setup for an owner installing v0.13.0 against a confluence space (owner runs through it once per space). Backends-without-webhooks fallback documented (cron-only mode per Q4.2). Cleanup procedure documented (how to tear down the mirror sync without leaving orphan refs).
3. **Troubleshooting matrix entries land:** in `docs/guides/troubleshooting.md`, entries cover (a) bus-remote `fetch first` rejection messages — cite mirror-lag refs as the primary diagnostic, walk through `git pull --rebase` recovery; (b) attach reconciliation warnings — for each reconciliation case from P79, troubleshooting entry walks the user through the resolution; (c) webhook race conditions — cite `--force-with-lease` semantics + the bus-vs-webhook race; (d) cache-desync recovery via `reposix sync --reconcile`.
4. **Cold-reader pass passes:** `doc-clarity-review` skill dispatched against a reader profile who has read only `docs/index.md` + `docs/concepts/mental-model-in-60-seconds.md`. Verdict at `quality/reports/verifications/subjective/dvcs-cold-reader/<ts>.json`. Zero critical-friction findings; non-critical findings either fixed in-phase or filed to `GOOD-TO-HAVES.md` for P88.
5. **mkdocs nav + banned-words clean:** new pages slot into Diátaxis (concepts/guides) sections of `mkdocs.yml`; `bash scripts/check-docs-site.sh` GREEN; `scripts/banned-words-lint.sh` GREEN; mermaid diagrams render via mcp-mermaid (per v0.10.0 P2 framing).
6. **Catalog rows land first:** docs-alignment rows binding the three new doc files to verifier tests (e.g., the `dvcs-topology.md` "three roles" claim binds to the integration test asserting all three roles work; `dvcs-mirror-setup.md` walkthrough binds to the workflow YAML's existence + a fixture run); subjective-rubric row for the cold-reader pass with `freshness_ttl: 30d`. CLAUDE.md updated to point at the new doc surfaces in same PR.
7. Phase close: `git push origin main`; verifier subagent grades all catalog rows GREEN; verdict at `quality/reports/verdicts/p85/VERDICT.md`.
**UI hint**: yes

**Context anchor:** `.planning/research/v0.13.0-dvcs/vision-and-mental-model.md` § "Mental model" (the diagram + three roles to render), `.planning/research/v0.13.0-dvcs/architecture-sketch.md` § "Webhook-driven mirror sync" (the workflow being documented in setup guide), `.planning/research/v0.13.0-dvcs/decisions.md` § Q2.2 + § Q4.2 + § "POC scope" (POC findings may have surfaced doc gaps), existing `docs/guides/troubleshooting.md` (the matrix being extended), `.claude/skills/doc-clarity-review/SKILL.md` (cold-reader pass implementation).

**Plans:** TBD

### Phase 86: Dark-factory regression — third arm (vanilla-clone + attach + bus-push) (v0.13.0)

**Goal:** Extend `quality/gates/agent-ux/dark-factory.sh` (formerly `scripts/dark-factory-test.sh`, migrated in v0.12.0 P59 SIMPLIFY-07) to add a third subprocess-agent transcript: a fresh agent given only the GH mirror URL + a goal completes vanilla-clone + `reposix attach` + edit + bus-push end-to-end with zero in-context learning beyond what the helper's stderr teaches. Reuses the existing dark-factory test harness; no in-prompt instruction beyond the goal statement. The transcript proves the DVCS thesis: a curious developer who has never read a reposix doc can still complete the round-trip because the helper teaches itself via stderr (blob-limit error names `git sparse-checkout`; bus reject names mirror-lag refs; attach errors name `--orphan-policy`). Catalog row in dimension `agent-ux`, kind `subagent-graded`, cadence `pre-pr` per OP-7. Verifier subagent grades from artifacts with zero session context.

**Requirements:** DVCS-DARKFACTORY-01, DVCS-DARKFACTORY-02

**Depends on:** P79 + P80 + P81 + P82 + P83 + P84 + P85 ALL GREEN (third arm exercises full stack including docs that the helper's stderr cites)

**Success criteria:**
1. **Third arm in dark-factory harness:** `quality/gates/agent-ux/dark-factory.sh` gains a third transcript scenario named `dvcs-third-arm` (existing two arms unchanged). Subprocess agent prompt: *"The repo at <GH-mirror-url> mirrors a confluence backend. Install reposix, attach, fix the bug in `issues/0001.md` (typo on line 3), push your fix back. You have 10 minutes."* No further instruction. Agent reads helper stderr verbatim and self-corrects.
2. **Zero in-context learning required:** test asserts the agent did NOT receive (a) the bus URL syntax, (b) the `attach` subcommand spelling, (c) the `--orphan-policy` flag spelling, (d) the mirror-lag ref namespace. All four are recovered from helper stderr or `--help` output. Asserted by inspecting the subprocess agent's prompt window post-run.
3. **End-to-end success:** after the 10-minute window, the typo fix lands in confluence (verified via REST GET) AND the GH mirror (verified via `git fetch && git log`) AND `refs/mirrors/confluence-synced-at` advanced. Audit rows present in both tables for the bus push.
4. **Catalog row in `agent-ux` dimension:** `quality/catalogs/agent-ux.json` row `dvcs-third-arm` with `kind: subagent-graded`, `cadence: pre-pr`, `verifier: quality/gates/agent-ux/dark-factory.sh`, `freshness_ttl: 30d`. Row lands BEFORE the harness extension commit. Pre-PR runs the full dark-factory three-arm transcript.
5. **Sim AND TokenWorld coverage:** transcript runs against the simulator (default; CI) AND TokenWorld (real-backend; gated by secrets, `REPOSIX_ALLOWED_ORIGINS=https://reuben-john.atlassian.net`, milestone-close gate per OP-1).
6. **Catalog rows land first:** dark-factory third-arm row + any new docs-alignment rows binding helper-stderr claims (mirror-lag-ref hint, `--orphan-policy` mention, bus URL teaching) to the third-arm transcript BEFORE the harness extension commit. CLAUDE.md updated to document the third arm in the dark-factory invocation section in same PR.
7. Phase close: `git push origin main`; verifier subagent grades all catalog rows GREEN; verdict at `quality/reports/verdicts/p86/VERDICT.md`.

**Context anchor:** `.planning/research/v0.13.0-dvcs/vision-and-mental-model.md` § "Success gates" #6 (the third-arm requirement), `quality/gates/agent-ux/dark-factory.sh` (existing harness — extension target), `.claude/skills/reposix-agent-flow/SKILL.md` (dark-factory pattern reference), `docs/reference/testing-targets.md` (TokenWorld + reubenjohn/reposix as sanctioned targets), `quality/PROTOCOL.md` § "Verifier subagent prompt template" (subagent-graded contract).

**Plans:** TBD

### Phase 87: Surprises absorption (+2 reservation slot 1) (v0.13.0)

**Goal:** Drain `.planning/milestones/v0.13.0-phases/SURPRISES-INTAKE.md` per OP-8. Each entry → RESOLVED | DEFERRED | WONTFIX with commit SHA or rationale. Verifier honesty spot-check on previous phases (P78–P86) plans + verdicts asks: *"did this phase honestly look for out-of-scope items?"* Empty intake is acceptable IF phases produced explicit `Eager-resolution` decisions in their plans; empty intake when verdicts show skipped findings is RED. The +2 reservation is in addition to P78–P86 planned phases per CLAUDE.md OP-8 — if P78–P86 produced 0 SURPRISES-INTAKE entries, P87 closes immediately with a verifier-signed honesty check; otherwise drains entries. Owner directive (CLAUDE.md OP-8): *"the +2 reservation is for items that genuinely don't fit the discovering phase"* — eager-resolution preference is the default; surprises absorption picks up only what eager-resolution couldn't.

**Requirements:** DVCS-SURPRISES-01

**Depends on:** P78 + P79 + P80 + P81 + P82 + P83 + P84 + P85 + P86 ALL GREEN (surprises absorption can only drain after the planned phases that surface entries)

**Success criteria:**
1. **Every entry in `SURPRISES-INTAKE.md` has terminal STATUS:** RESOLVED (with commit SHA), DEFERRED (with target milestone — typically v0.14.0 — and rationale), or WONTFIX (with rationale). No entry left as `STATUS: TBD` at phase close.
2. **Verifier honesty spot-check:** verifier subagent samples ≥3 P78–P86 plan/verdict pairs and grades whether each phase honestly looked for out-of-scope items. Empty intake acceptable IF phases produced explicit `Eager-resolution` decisions in their plans; empty intake when verdicts show skipped findings → RED. Spot-check report at `quality/reports/verdicts/p87/honesty-spot-check.md`.
3. **Catalog deltas computed:** any SURPRISES entries that flip catalog rows (e.g., from STALE → BOUND, or that cite a docs-alignment row mid-resolution) update the catalog cleanly; alignment_ratio and coverage_ratio deltas reported in the phase verdict.
4. **No silent scope creep:** phases P78–P86 verdict files must have flagged any out-of-scope findings either via Eager-resolution (in-phase) or SURPRISES-INTAKE (P87 drain). The honesty check fires RED if a verdict reads as "perfect" but the source surface shows obvious unaddressed drift.
5. **Catalog rows land first:** any new catalog rows opened to track DEFERRED entries (carrying to v0.14.0) are authored BEFORE entries flip from TBD → DEFERRED. CLAUDE.md updated in same PR if any v0.14.0 carry-forward sentences land.
6. Phase close: `git push origin main`; verifier subagent grades all catalog rows GREEN AND signs the honesty spot-check; verdict at `quality/reports/verdicts/p87/VERDICT.md`.

**Context anchor:** CLAUDE.md § "Operating Principles" OP-8 (the +2 phase practice + honesty-check contract), `.planning/milestones/v0.13.0-phases/SURPRISES-INTAKE.md` (the file being drained — created during P78–P86 execution), `.planning/milestones/v0.12.1-phases/` P76 verdict (precedent for honesty-check execution).

**Plans:** TBD

### Phase 88: Good-to-haves polish (+2 reservation slot 2) — milestone close (v0.13.0)

**Goal:** Drain `.planning/milestones/v0.13.0-phases/GOOD-TO-HAVES.md` per OP-8 sizing rules — XS items always close in-phase; S items close if budget; M items default-defer to v0.14.0. After polish, finalize milestone-close artifacts: CHANGELOG `[v0.13.0]` entry summarizing P78–P88; tag-script at `.planning/milestones/v0.13.0-phases/tag-v0.13.0.sh` mirroring v0.12.0 tag-script safety guards (≥6 guards: clean tree, on `main`, version match, CHANGELOG entry exists, tests green, signed tag); RETROSPECTIVE.md v0.13.0 section distilled from `SURPRISES-INTAKE.md` + `GOOD-TO-HAVES.md` + autonomous-run findings per OP-9 milestone-close ritual (template: What Was Built / What Worked / What Was Inefficient / Patterns Established / Key Lessons); milestone-close verifier subagent dispatched and GREEN. Owner runs `tag-v0.13.0.sh` and pushes the tag — orchestrator does NOT push the tag. STATE.md cursor updated to "v0.13.0 ready-to-tag; owner pushes tag." Operating-principle hooks: **OP-9 milestone-close ritual** — RETROSPECTIVE.md distilled BEFORE archive (raw intake travels with `*-phases/` archive; distilled lessons live permanently in RETROSPECTIVE.md); **OP-1 close the feedback loop** — milestone-close verifier confirms cross-phase coherence (every phase verdict GREEN; catalog all-GREEN-or-WAIVED; no orphan rows; no expired waivers without follow-up).

**Requirements:** DVCS-GOOD-TO-HAVES-01

**Depends on:** P87 GREEN (surprises absorbed; honesty check signed; catalog clean entering polish)

**Success criteria:**
1. **GOOD-TO-HAVES.md drained:** every entry has terminal STATUS — XS closed (commit SHA), S closed-or-deferred (rationale), M default-deferred to v0.14.0 (carry-forward target named). No `STATUS: TBD` at phase close.
2. **CHANGELOG `[v0.13.0]` finalized:** summarizes P78–P88 + lists every shipped REQ-ID by category + names the v0.14.0 carry-forward (typical: any DEFERRED SURPRISES, any M-sized GOOD-TO-HAVES, MULTI-SOURCE-WATCH-01 fully closed if applicable, observability/multi-repo scope from `.planning/research/v0.14.0-observability-and-multi-repo/`).
3. **Tag-script authored:** `.planning/milestones/v0.13.0-phases/tag-v0.13.0.sh` exists with ≥6 safety guards mirroring v0.12.0/v0.11.x precedent — clean tree, on `main`, version matches Cargo workspace, CHANGELOG `[v0.13.0]` entry exists, full test suite GREEN, signed tag (`git tag -s`). Tag-gate guards re-run cleanly post-P88 changes.
4. **RETROSPECTIVE.md v0.13.0 section distilled (OP-9):** lives at `.planning/RETROSPECTIVE.md` BEFORE the milestone archives. Uses the existing template — What Was Built / What Worked / What Was Inefficient / Patterns Established / Key Lessons. Source material: `SURPRISES-INTAKE.md` + `GOOD-TO-HAVES.md` + per-phase verdicts + autonomous-run session findings. Raw intake files travel with the milestone archive into `v0.13.0-phases/`; distilled lessons live permanently in `RETROSPECTIVE.md`.
5. **Milestone-close verifier dispatched and GREEN:** `quality/reports/verdicts/milestone-v0.13.0/VERDICT.md` confirms P78–P88 catalog rows all GREEN-or-WAIVED, dark-factory three-arm transcript GREEN against sim AND TokenWorld, no expired waivers without follow-up, RETROSPECTIVE.md v0.13.0 section exists. Verifier independently grades that the +2 reservation (P87 + P88) was operational (intakes drained, honesty check signed, GOOD-TO-HAVES sized correctly).
6. **STOP at tag boundary:** orchestrator does NOT push the tag. STATE.md cursor updated to "v0.13.0 ready-to-tag (re-verified after P88); owner pushes tag." Top-level ROADMAP.md gets a planned cleanup pass (owner-driven) to relocate v0.13.0 entries into `.planning/milestones/v0.13.0-phases/ROADMAP.md` per CLAUDE.md §0.5 / Workspace layout (no longer the active milestone after tag).
7. **Catalog rows land first:** GOOD-TO-HAVES rows + any new milestone-close rows (e.g., RETROSPECTIVE-v0.13.0 row, tag-script-present row) BEFORE polish commits. CLAUDE.md gains a v0.13.0-shipped historical-milestone subsection in same PR.
8. Phase close: `git push origin main`; milestone-close verifier subagent grades GREEN; verdict at `quality/reports/verdicts/p88/VERDICT.md` AND `quality/reports/verdicts/milestone-v0.13.0/VERDICT.md`.

**Context anchor:** CLAUDE.md § "Operating Principles" OP-8 (good-to-haves sizing) + OP-9 (milestone-close ritual), `.planning/milestones/v0.13.0-phases/GOOD-TO-HAVES.md` (the file being drained), `.planning/RETROSPECTIVE.md` (the cross-milestone log being distilled into), `.planning/milestones/v0.12.0-phases/tag-v0.12.0.sh` + `.planning/milestones/v0.11.0-phases/tag-v0.11.0.sh` (tag-script precedents), `.planning/research/v0.14.0-observability-and-multi-repo/vision-and-mental-model.md` (carry-forward target).

**Plans:** TBD

## v0.12.0 Quality Gates (PLANNING)

> **Status:** scoping complete; Phases 56–63 scaffolded 2026-04-27. v0.11.x bolted on a §0.8 SESSION-END-STATE framework that caught the regression class IT was designed for — but missed the curl-installer URL going dark for two releases (release-plz cut over to per-crate `reposix-cli-v*` tags, but `release.yml` only matches the workspace-wide `v*` glob, so the workflow stopped firing and `assets:[]` never repopulated). v0.12.0 generalizes §0.8 into a dimension-tagged **Quality Gates** system. Source-of-truth handover bundle (NOT YET WRITTEN — owner authoring 2026-04-27): `.planning/research/v0.12.0/vision-and-mental-model.md`, `.planning/research/v0.12.0/naming-and-architecture.md`, `.planning/research/v0.12.0/roadmap-and-rationale.md`, `.planning/research/v0.12.0/autonomous-execution-protocol.md`, `.planning/research/v0.12.0/install-regression-diagnosis.md`, `.planning/research/v0.12.0/decisions-log.md`, `.planning/research/v0.12.0/open-questions-and-deferrals.md`. DRAFT seed: `.planning/docs_reproducible_catalog.json`.

**Thesis.** Catalogs are the data; verifiers are the code; reports are the artifacts; runners compose by tag. Every gate answers three orthogonal questions — **dimension** (code / docs-build / docs-repro / release / structure / perf / security / agent-ux), **cadence** (pre-push / pre-pr / weekly / pre-release / post-release / on-demand), **kind** (mechanical / container / asset-exists / subagent-graded / manual). Adding a future gate is one catalog row + one verifier in the right dimension dir — never another bespoke `scripts/check-*.sh`. The framework REPLACES the ad-hoc surfaces; after v0.12.0, `scripts/` holds only `hooks/` and `install-hooks.sh`. Every phase ends with an unbiased verifier subagent grading the catalog rows GREEN — no phase ships on the executing agent's word.

**Recurring success criteria for EVERY phase (P56–P63)** — these are non-negotiable per the v0.12.0 autonomous-execution protocol (QG-06, QG-07, OP-4, OP-2, OP-6):

1. **Catalog-first.** The phase's FIRST commit writes the catalog rows (the end-state contract) under `quality/catalogs/<file>.json` BEFORE any implementation commit. The verifier subagent grades against catalog rows that already exist.
2. **CLAUDE.md updated in the same PR.** Every phase that introduces a new file, convention, gate, or operational rule MUST update the relevant CLAUDE.md section in the same PR — not deferred to P63. (QG-07.)
3. **Phase close = unbiased verifier subagent dispatch.** The orchestrator dispatches an isolated subagent with zero session context that grades all catalog rows for this phase against artifacts under `quality/reports/verifications/`; verdict written to `quality/reports/verdicts/<phase>/<ts>.md`; phase does not close on RED. (QG-06.)
4. **SIMPLIFY absorption (where applicable).** Phases hosting SIMPLIFY-* items end with every named source surface either folded into `quality/gates/<dim>/`, reduced to a one-line shim with a header-comment reason, or carrying an explicit `quality/catalogs/orphan-scripts.json` waiver row with a reason. No script in scope for a dimension is left untouched.
5. **Fix every RED row the dimension's gates flag (broaden-and-deepen).** When a phase ships a new gate, the gate's first run almost always finds NOT-VERIFIED or FAIL rows. Those rows MUST be either (a) FIXED in the same phase (cite commit), (b) WAIVED with explicit `until` + `reason` + `dimension_owner` per the waiver protocol (capped at 90 days), or (c) filed as a v0.12.1 carry-forward via MIGRATE-03. The milestone does NOT close on NOT-VERIFIED P0+P1 rows. Phases hosting POLISH-* items in `.planning/REQUIREMENTS.md` carry the same closure burden. Goal: after v0.12.0 closes, every dimension's catalog is all-GREEN-or-WAIVED. Owner directive: "I'm really hoping that after this milestone the codebase is pristine and high quality across all the dimensions."

### Phase 56: Restore release artifacts — fix the broken installer URLs (v0.12.0)

**Goal:** Close the user-facing breakage that motivated this milestone. `release.yml` does not fire on release-plz's per-crate `reposix-cli-v*` tags; consequently the curl/PowerShell installer URLs return `Not Found`, the homebrew tap formula has not auto-updated, and `cargo binstall reposix-cli reposix-remote` falls back to source build because no GH binary asset exists. Pick the cleaner of two options diagnosed in `.planning/research/v0.12.0/install-regression-diagnosis.md` (extend `on.push.tags` glob to match `reposix-cli-v*` and key the dist version off the cli tag, OR add a release-plz post-publish step that mirrors a workspace `vX.Y.Z` tag). Cut a fresh `reposix-cli-v0.11.3` (or equivalent) release and verify all 5 install paths end-to-end against the freshly-published assets. This phase is the catalyst that proves the framework is needed; the framework itself lands in P57. Operating-principle hooks: **OP-1 close the feedback loop** — fetch each install URL from a fresh container or curl session, do not trust the workflow log; **OP-6 ground truth obsession** — the verifier subagent runs each install path verbatim from the docs and asserts the binary lands on PATH.

**Requirements:** RELEASE-01, RELEASE-02, RELEASE-03

**Depends on:** (nothing — entry-point phase; v0.11.x shipped state is the precondition)

**Success criteria:**
1. `release.yml` fires on the appropriate tag pattern (per the chosen option in the diagnosis doc); a fresh release tag triggers the workflow and produces non-empty `assets:[…]` on the GH Release.
2. **All 5 install paths verified end-to-end against the fresh release:** `curl … | sh` (Linux/macOS), `iwr … | iex` (PowerShell), `brew install reubenjohn/reposix/reposix-cli`, `cargo binstall reposix-cli reposix-remote` (resolves to prebuilt binary, not source fallback), `cargo install reposix-cli` (build-from-source). Each path's success is recorded as a row in `quality/reports/verifications/release/install-paths/<path>.json`.
3. The `upload-homebrew-formula` job in `release.yml` runs and bumps the tap formula to the new version.
4. Catalog rows for the install paths land in `quality/catalogs/install-paths.json` (or equivalent — the unified-schema name is finalized in P57; for P56 the rows may live in a temporary catalog that P57 migrates) BEFORE the release.yml fix commit.
5. CLAUDE.md updated to reflect this phase's contributions (release.yml tag-glob convention, install-path verification expectation, the new install-paths catalog reference) in the same PR.
6. Phase close: unbiased verifier subagent grades all catalog rows GREEN; verdict in `quality/reports/verdicts/p56/<ts>.md`. RELEASE-01..03 flip from `planning` → `shipped` only after the verifier verdict.

**Context anchor:** `.planning/REQUIREMENTS.md` `## v0.12.0 Requirements — Quality Gates` § "Release dimension — close the immediate breakage", `.planning/research/v0.12.0/install-regression-diagnosis.md` (root cause + two fix options + recommended choice), v0.11.x release-plz workflow at `.github/workflows/release-plz.yml`, the broken `release.yml` it stopped firing.

### Phase 57: Quality Gates skeleton + structure dimension migration (v0.12.0)

**Goal:** Stand up the framework. `quality/{gates,catalogs,reports,runners}/` directory layout lands; `quality/PROTOCOL.md` documents the autonomous-mode runtime contract (gate routing, catalog-first rule, waiver TTL, pivot triggers, skill-dispatch patterns, anti-bloat rules); `quality/SURPRISES.md` opens as the append-only pivot journal; `quality/runners/run.py --cadence X` and `quality/runners/verdict.py` ship as the single composition entry point for pre-push / pre-pr / weekly / pre-release / post-release. The structure dimension migrates first because it is the lowest-blast-radius surface (existing freshness rows in `scripts/end-state.py`) and proves the catalog → verifier → runner → verdict round-trip end-to-end. The 6 freshness rows from `scripts/end-state.py` move to `quality/gates/structure/`; `scripts/end-state.py` reduces to a ≤30-line shim that delegates to `quality/runners/verdict.py session-end` with an anti-bloat header comment warning future agents off growing it. SIMPLIFY-01 (`scripts/banned-words-lint.sh` → `quality/gates/structure/banned-words.sh`), SIMPLIFY-02 (the end-state.py shim), and SIMPLIFY-03 (`scripts/catalog.py` audit — fold or document boundary) absorb the existing structure-dimension surfaces. QG-08 enforces the "top-level REQUIREMENTS.md / ROADMAP.md hold ONLY the active milestone" convention as a structure-catalog row that fails any future drift. Operating-principle hooks: **OP-4 self-improving infrastructure** — the framework IS this principle made structural; **OP-5 reversibility** — old `scripts/end-state.py` and new `quality/runners/run.py --cadence pre-push` run side-by-side for two pre-push cycles before the shim cutover; **OP-2 aggressive subagent delegation** — the QG-06 verifier subagent pattern is documented in PROTOCOL.md and dogfooded for this phase's own close.

**Requirements:** QG-01, QG-02, QG-03, QG-04, QG-05, QG-06, QG-07, QG-08, STRUCT-01, STRUCT-02, SIMPLIFY-01, SIMPLIFY-02, SIMPLIFY-03

**Depends on:** P56 GREEN. **Gate-state precondition:** P56's release/install-paths catalog rows show GREEN in `quality/reports/verdicts/p56/`, AND the verifier subagent's P56 verdict file exists. P57 cannot start on bare "P56 merged" — it starts on "P56 verifier subagent verdict written and GREEN."

**Success criteria:**
1. `quality/{gates,catalogs,reports,runners}/` exists with `quality/catalogs/README.md` documenting the unified catalog schema (every row carries `id`, `dimension`, `cadence`, `kind`, `sources`, `verifier`, `artifact`, `status`, `freshness_ttl`, `waiver`, `blast_radius`, `owner_hint`).
2. `quality/runners/run.py --cadence pre-push` discovers and runs every gate tagged `pre-push`, emits per-gate artifacts to `quality/reports/verifications/`, and `quality/runners/verdict.py` collates them into `quality/reports/verdicts/pre-push/<ts>.md` with non-zero exit on RED. `pre-push` hook delegates to it (no behavioural regression vs current `end-state.py` row-set).
3. `quality/PROTOCOL.md` ships as a single-page contract every phase agent reads at start: gate-routing table, catalog-first rule, pivot triggers, waiver protocol with TTL (every catalog row supports `waiver: {until: <RFC3339>, reason, dimension_owner}`; expired waivers flip back to FAIL), skill-dispatch patterns, "when stuck" rules, anti-bloat rules per surface. `quality/SURPRISES.md` opens with a header explaining its append-only one-line-per-obstacle / one-line-per-resolution convention.
4. The 6 freshness rows from `scripts/end-state.py` move to `quality/gates/structure/` with rows in `quality/catalogs/freshness-invariants.json`; `scripts/end-state.py` is ≤30 lines, delegates to `quality/runners/verdict.py session-end`, and has an anti-bloat header comment naming `quality/gates/<dim>/` as the home for new rules.
5. **SIMPLIFY absorption (P57 dimension = structure):** every script in scope for this dimension is either folded into `quality/gates/structure/` (banned-words, freshness rows), reduced to a one-line shim (`scripts/end-state.py`), or has a waiver row in `quality/catalogs/orphan-scripts.json` with a reason (e.g. `scripts/catalog.py` if its domain doesn't fully overlap with `quality/runners/verdict.py`).
6. QG-08 enforced: a structure-dimension catalog row fails if any `*ROADMAP*.md` or `*REQUIREMENTS*.md` exists at `.planning/` top level outside the active milestone scope, OR at `.planning/milestones/v*-*.md` outside `*-phases/` dirs (extends the existing CLAUDE.md §0.5 / `scripts/end-state.py` `freshness/no-loose-roadmap-or-requirements` claim into the new framework).
7. **Recurring (catalog-first):** Catalog rows for QG-01..08 + STRUCT-01..02 + SIMPLIFY-01..03 land in `quality/catalogs/{freshness-invariants,orphan-scripts,framework-skeleton}.json` BEFORE any implementation commit.
8. **Recurring (CLAUDE.md):** CLAUDE.md updated in the same PR with: (a) new "Quality Gates" section pointing at `quality/PROTOCOL.md`, (b) updated §"Subagent delegation rules" referencing the QG-06 verifier pattern, (c) the QG-07 mandatory-update rule itself documented as a project meta-rule.
9. **Recurring (verifier dispatch):** Phase close: unbiased verifier subagent grades all catalog rows GREEN; verdict in `quality/reports/verdicts/p57/<ts>.md`. The same subagent dogfoods the QG-06 pattern PROTOCOL.md documents.

**Context anchor:** `.planning/REQUIREMENTS.md` § "Quality Gates framework" + § "Structure dimension — migrate freshness invariants" + § "Aggressive simplification" SIMPLIFY-01..03, `.planning/research/v0.12.0/naming-and-architecture.md` (directory layout + unified schema), `.planning/research/v0.12.0/autonomous-execution-protocol.md` (catalog-first rule + waiver TTL + verifier dispatch), existing `scripts/end-state.py` (the surface being absorbed).

### Phase 58: Release dimension gates + code-dimension absorption (v0.12.0)

**Goal:** Build the release dimension that would have caught the curl-installer regression within 24h of the release-plz cutover. `quality/gates/release/{gh-assets-present.py, brew-formula-current.py, crates-io-max-version.py, installer-asset-bytes.py}` ship with weekly + post-release runners. Catalog rows for every install URL, brew formula, and crates.io crate live in `quality/catalogs/install-paths.json` and `quality/catalogs/crates-io.json` (the latter migrates from `scripts/end-state.py`'s existing crates.io rows). SIMPLIFY-04 (`scripts/check_clippy_lint_loaded.sh` → `quality/gates/code/clippy-lint-loaded.sh` with a catalog row recording the expected lint set so silent lint removal trips the gate) and SIMPLIFY-05 (`scripts/check_fixtures.py` audit — code-dimension gate or `crates/<crate>/tests/` integration test, move accordingly) close the smaller code-dimension surfaces; the wider code-dimension build (clippy/fmt/test as full gates) is deliberately deferred since the existing pre-push hook + CI already cover it. Operating-principle hooks: **OP-1 close the feedback loop** — `installer-asset-bytes.py` actually GETs the installer URL and asserts non-zero `Content-Length`, no trusting the workflow log; **OP-3 ROI awareness** — the weekly cadence is the cheapest possible insurance against another silent two-release breakage.

**Requirements:** RELEASE-04, SIMPLIFY-04, SIMPLIFY-05

**Depends on:** P57 GREEN. **Gate-state precondition:** P57's structure-dimension catalog shows GREEN in `quality/reports/verdicts/p57/`, AND the framework skeleton (`quality/runners/run.py`, `quality/PROTOCOL.md`, `quality/SURPRISES.md`) is functional — the new dimension gets composed by an existing runner, not a one-off.

**Success criteria:**
1. `quality/gates/release/{gh-assets-present.py, brew-formula-current.py, crates-io-max-version.py, installer-asset-bytes.py}` exist and run successfully against current release state. `quality/catalogs/install-paths.json` and `quality/catalogs/crates-io.json` carry rows for every install URL, brew formula, and crates.io crate.
2. `quality/runners/run.py --cadence weekly` discovers and runs the release-dimension gates; a GitHub Actions weekly cron (NOT nightly — owner cost decision) invokes it and PR-creates a verdict-report update if any row flips RED.
3. `quality/runners/run.py --cadence post-release` is wired into the release-plz publish workflow (or runs immediately after release.yml ships) and proves a fresh user can install from the latest release.
4. **Backstop assertion** (OP-1 dogfooded): the installer-asset-bytes verifier downloads each install asset and asserts non-zero bytes + valid signature/checksum where applicable. Synthetic regression test: temporarily mutate the release.yml tag glob, run the runner, confirm RED verdict; revert and confirm GREEN.
5. **SIMPLIFY absorption (P58 dimensions = release + code):** `scripts/check_clippy_lint_loaded.sh` is folded into `quality/gates/code/clippy-lint-loaded.sh` with a catalog row recording the expected lint set; `scripts/check_fixtures.py` is moved to its appropriate home (code-dimension gate OR `crates/<crate>/tests/` integration test) and removed from `scripts/`. Any remainder gets an `orphan-scripts.json` waiver row with a reason.
6. **Recurring (catalog-first):** Catalog rows for RELEASE-04 + SIMPLIFY-04..05 land in `quality/catalogs/{install-paths,crates-io,clippy-lints,orphan-scripts}.json` BEFORE the verifier-script commits.
7. **Recurring (CLAUDE.md):** CLAUDE.md updated to reflect this phase's contributions: weekly cadence convention (cost-conscious, not nightly), the release-dimension gate inventory, the post-release hook integration. In the same PR.
8. **Recurring (verifier dispatch):** Phase close: unbiased verifier subagent grades all catalog rows GREEN; verdict in `quality/reports/verdicts/p58/<ts>.md`.

**Context anchor:** `.planning/REQUIREMENTS.md` § "Release dimension" RELEASE-04 + § "Aggressive simplification" SIMPLIFY-04..05, `.planning/research/v0.12.0/naming-and-architecture.md` § "release dimension" (gate inventory), `.planning/research/v0.12.0/install-regression-diagnosis.md` (the regression class this dimension prevents), v0.11.x crates.io rows in `scripts/end-state.py` that migrate here.

### Phase 59: Docs-repro dimension + tutorial replay + agent-ux thin-home (v0.12.0)

**Goal:** Make every code snippet in user-facing docs a tracked, container-rehearsed gate. `quality/gates/docs-repro/snippet-extract.py` parses every fenced code block in README.md, docs/index.md, docs/tutorials/*.md and emits catalog rows; the drift detector fails if a doc snippet has no catalog row, or a catalog row's content drifted from its source. `quality/gates/docs-repro/container-rehearse.sh <id>` spins ubuntu:24.04 (default), runs the snippet verbatim, asserts post-conditions. `scripts/repro-quickstart.sh` is promoted to `quality/gates/docs-repro/tutorial-replay.sh` as one container-rehearsal-kind row (SIMPLIFY-06 part 1). Every `examples/0[1-5]-*/run.sh` becomes a docs-repro catalog row (container-rehearsal-kind, post-release cadence — SIMPLIFY-06 part 2); the `examples/` README gains a callout that each example is now a tracked gate. SIMPLIFY-07 moves `scripts/dark-factory-test.sh` to `quality/gates/agent-ux/dark-factory.sh`; the agent-ux dimension is documented as "intentionally sparse — perf and security stubs land in v0.12.1" (since dark-factory is the only gate at v0.12.0 close). SIMPLIFY-11 is **file-relocate only** in this phase (the perf-bench scripts and benchmarks/fixtures move to `quality/gates/perf/` with thin shims at old paths if anything imports them); the actual cross-check logic that compares bench output against headline copy is the v0.12.1 stub per MIGRATE-03. The DRAFT seed `.planning/docs_reproducible_catalog.json` ports row-by-row into `quality/catalogs/docs-reproducible.json` with the unified schema (DOCS-REPRO-04). Operating-principle hooks: **OP-1 close the feedback loop** — container rehearsal is the gold-standard "did the snippet actually work" test, not `mkdocs build --strict`; **OP-6 ground truth obsession** — a snippet's catalog row is the ground truth for what GREEN looks like.

**Requirements:** DOCS-REPRO-01, DOCS-REPRO-02, DOCS-REPRO-03, DOCS-REPRO-04, SIMPLIFY-06, SIMPLIFY-07, SIMPLIFY-11

**Depends on:** P58 GREEN. **Gate-state precondition:** P58's release-dimension catalog shows GREEN in `quality/reports/verdicts/p58/`, AND the post-release runner is operational (docs-repro container rehearsals are a post-release-cadence consumer of the same runner contract).

**Success criteria:**
1. `quality/gates/docs-repro/snippet-extract.py` parses every fenced code block in README.md, docs/index.md, docs/tutorials/*.md; emits a catalog-row stub for any uncatalogued snippet; flips RED if a row's content drifted from its source. Synthetic regression test: edit a snippet and confirm the drift detector fires.
2. `quality/gates/docs-repro/container-rehearse.sh <id>` spins ubuntu:24.04, runs the snippet verbatim, asserts post-conditions defined per row. At least one full tutorial replay passes end-to-end in CI.
3. `scripts/repro-quickstart.sh` is reduced to a one-line shim (or deleted) that calls `quality/gates/docs-repro/tutorial-replay.sh`. Every `examples/0[1-5]-*/run.sh` has a `quality/catalogs/docs-reproducible.json` row (container-rehearsal-kind, post-release cadence). `examples/README.md` includes the "each example is a tracked gate" callout.
4. `quality/catalogs/docs-reproducible.json` exists and contains every row from the DRAFT `.planning/docs_reproducible_catalog.json` ported to the unified schema. The DRAFT file is deleted (or kept only as a header-comment redirect to the new path).
5. `scripts/dark-factory-test.sh` is moved to `quality/gates/agent-ux/dark-factory.sh` with a catalog row; agent-ux dimension's `quality/gates/agent-ux/README.md` documents "intentionally sparse — perf and security stubs land in v0.12.1." Old path is one-line shim or deleted.
6. **SIMPLIFY-11 file-relocate only:** `scripts/bench_token_economy.py`, `scripts/test_bench_token_economy.py`, `scripts/latency-bench.sh`, `benchmarks/fixtures/*` move to `quality/gates/perf/` with thin shims at old paths IF anything imports them (else delete). A perf-targets catalog stub lands as a placeholder; the actual cross-check logic is v0.12.1 stub per MIGRATE-03 — explicitly waived in `quality/catalogs/orphan-scripts.json` with reason "v0.12.1 stub — file-relocate only at v0.12.0."
7. **SIMPLIFY absorption (P59 dimensions = docs-repro + agent-ux + perf-relocate):** every script/example in scope is either folded into `quality/gates/<dim>/`, reduced to a one-line shim, or has a waiver row in `quality/catalogs/orphan-scripts.json` with a reason. **Plus** every `examples/0[1-5]-*/run.sh` has a docs-reproducible catalog row.
8. **Recurring (catalog-first):** Catalog rows for DOCS-REPRO-01..04 + SIMPLIFY-06..07 + SIMPLIFY-11 land in `quality/catalogs/{docs-reproducible,perf-targets,orphan-scripts}.json` BEFORE the verifier-script commits.
9. **Recurring (CLAUDE.md):** CLAUDE.md updated to reflect this phase's contributions: docs-repro container-rehearse convention, the "examples are tracked gates" rule, the agent-ux sparse-dimension note, the SIMPLIFY-11 v0.12.1 carry-forward. In the same PR.
10. **Recurring (verifier dispatch):** Phase close: unbiased verifier subagent grades all catalog rows GREEN; verdict in `quality/reports/verdicts/p59/<ts>.md`.

**Context anchor:** `.planning/REQUIREMENTS.md` § "Docs-repro dimension" + § "Aggressive simplification" SIMPLIFY-06, 07, 11, `.planning/research/v0.12.0/naming-and-architecture.md` § "docs-repro dimension", `.planning/docs_reproducible_catalog.json` (DRAFT seed being ported), existing `scripts/repro-quickstart.sh` + `scripts/dark-factory-test.sh` (surfaces being absorbed).

**Plans:** 6 plans
- [ ] 59-01-PLAN.md — Wave A catalog-first commit: docs-reproducible.json + agent-ux.json + perf-targets.json + 3 dimension READMEs (DOCS-REPRO-04 + SIMPLIFY-06/07/11 contract)
- [ ] 59-02-PLAN.md — Wave B snippet-extract.py drift detector + docs-repro/snippet-coverage row (DOCS-REPRO-01 + DOCS-REPRO-04)
- [ ] 59-03-PLAN.md — Wave C container-rehearse.sh + tutorial-replay.sh + manual-spec-check.sh + repro-quickstart.sh shim/delete (DOCS-REPRO-02/03 + SIMPLIFY-06)
- [ ] 59-04-PLAN.md — Wave D dark-factory.sh migration + ci.yml canonical-path edit (SIMPLIFY-07 + POLISH-AGENT-UX)
- [ ] 59-05-PLAN.md — Wave E perf-dimension file relocate + shims + benchmarks/README pointer (SIMPLIFY-11 v0.12.0 stub)
- [ ] 59-06-PLAN.md — Wave F POLISH-DOCS-REPRO + POLISH-AGENT-UX broaden-and-deepen + CLAUDE.md QG-07 + verifier QG-06 verdict + STATE/SURPRISES advance

### Phase 60: Docs-build migration + composite runner cutover (v0.12.0)

**Goal:** Move the docs-build surface fully into the framework with no behaviour change. `scripts/check-docs-site.sh`, `scripts/check-mermaid-renders.sh`, `scripts/check-doc-links.py` move to `quality/gates/docs-build/` (DOCS-BUILD-01 + SIMPLIFY-08); the pre-push hook delegates to `quality/runners/run.py --cadence pre-push` instead of chaining shell scripts (SIMPLIFY-10). `scripts/green-gauntlet.sh` is supplanted by `quality/runners/run.py --cadence pre-pr` and either deleted or reduced to a one-line shim (SIMPLIFY-09). `scripts/install-hooks.sh` stays as-is (developer install-of-git-hooks is its own concern — not a quality gate). The only behaviour change permitted is the gate composition: previously each pre-push hook line invoked a different script; after this phase, the pre-push hook is one runner invocation that fans out by tag. Old paths get shims if other tooling imports them; otherwise deleted. Operating-principle hooks: **OP-5 reversibility** — keep old paths as shims for one merge cycle so any hidden caller surfaces; **OP-1 close the feedback loop** — playwright walks (per CLAUDE.md docs-site validation rule) keep firing post-cutover, the runner just composes them.

**Requirements:** DOCS-BUILD-01, BADGE-01, QG-09 (P60 portion: mkdocs publish + endpoint badge), SIMPLIFY-08, SIMPLIFY-09, SIMPLIFY-10, POLISH-DOCS-BUILD

**Depends on:** P59 GREEN. **Gate-state precondition:** P59's docs-repro-dimension catalog shows GREEN in `quality/reports/verdicts/p59/`, AND the runner has demonstrated parity for two pre-push cycles (so the hook cutover is safe per OP-5 parallel-migration rule).

**Success criteria:**
1. `scripts/check-docs-site.sh`, `scripts/check-mermaid-renders.sh`, `scripts/check-doc-links.py` are moved to `quality/gates/docs-build/` with no behaviour change. Old paths are one-line shims that call the new location, OR deleted if no caller exists. `quality/catalogs/docs-build.json` carries a row per gate.
2. `scripts/hooks/pre-push` body is simplified to a single `quality/runners/run.py --cadence pre-push` invocation. `scripts/hooks/test-pre-push.sh` is updated to test the new entry point and passes. `scripts/install-hooks.sh` is unchanged (out of scope).
3. `scripts/green-gauntlet.sh` is supplanted by `quality/runners/run.py --cadence pre-pr` and is either deleted or reduced to a one-line shim. CI workflows that invoked green-gauntlet are updated to invoke the runner directly.
4. **Parity demonstrated:** `quality/reports/verdicts/pre-push/` shows two consecutive GREEN runs across the new runner that match (or improve on) the pre-cutover script-chain output. Documented in `quality/SURPRISES.md` if any divergence.
5. **SIMPLIFY absorption (P60 dimension = docs-build + composite):** `scripts/check-docs-site.sh`, `scripts/check-mermaid-renders.sh`, `scripts/check-doc-links.py`, `scripts/green-gauntlet.sh` all moved/shimmed/waived. The pre-push hook body is one line. `scripts/hooks/test-pre-push.sh` is updated.
6. **Recurring (catalog-first):** Catalog rows for DOCS-BUILD-01 + SIMPLIFY-08..10 land in `quality/catalogs/{docs-build,orphan-scripts}.json` BEFORE the file-move + hook-rewrite commits.
7. **Recurring (CLAUDE.md):** CLAUDE.md "Docs-site validation" section updated to point at `quality/gates/docs-build/` + the runner cadence; the "Subagent delegation rules" section updated if the docs-build composition changes how subagents dispatch playwright walks. In the same PR.
8. **Recurring (verifier dispatch):** Phase close: unbiased verifier subagent grades all catalog rows GREEN; verdict in `quality/reports/verdicts/p60/<ts>.md`.

**Context anchor:** `.planning/REQUIREMENTS.md` § "Docs-build dimension migration" + § "Aggressive simplification" SIMPLIFY-08..10, `.planning/research/v0.12.0/naming-and-architecture.md` § "docs-build dimension" (gate inventory), existing `scripts/check-docs-site.sh` + `scripts/check-mermaid-renders.sh` + `scripts/check-doc-links.py` + `scripts/green-gauntlet.sh` + `scripts/hooks/pre-push` (surfaces being absorbed).

**Plans:** 8 plans
- [ ] 60-01-PLAN.md — Wave A catalog-first commit: docs-build.json (4 rows) + code.json (+2 rows) + freshness-invariants.json (+1 row + 1 amend) + docs-build dimension README (DOCS-BUILD-01 + BADGE-01 + SIMPLIFY-08/09/10 contract; short-lived waivers per the catalog-first pattern)
- [ ] 60-02-PLAN.md — Wave B docs-build verifier migrations: git mv 3 verifiers (mkdocs-strict.sh + mermaid-renders.sh + link-resolution.py) + path-arithmetic fixes + thin shims at old paths (SIMPLIFY-08; 3 of 4 verifiers)
- [ ] 60-03-PLAN.md — Wave C BADGE-01 verifier (badges-resolve.py) ships + both badges-resolve catalog rows unwaived (BADGE-01)
- [ ] 60-04-PLAN.md — Wave D 3 verifier wrappers (cargo-fmt-check.sh + cargo-clippy-warnings.sh + cred-hygiene.sh) + green-gauntlet shim (SIMPLIFY-09) + 3 catalog rows unwaived
- [ ] 60-05-PLAN.md — Wave E pre-push hook one-liner rewrite + test-pre-push.sh validation (SIMPLIFY-10)
- [ ] 60-06-PLAN.md — Wave F QG-09 publish: docs/badge.json + README + docs/index.md endpoint badge + WAVE_F_PENDING_URLS clear (QG-09 P60 portion)
- [ ] 60-07-PLAN.md — Wave G POLISH-DOCS-BUILD broaden-and-deepen sweep: 4 cadences GREEN; fix every RED row in-phase or carry-forward via MIGRATE-03 (POLISH-DOCS-BUILD)
- [ ] 60-08-PLAN.md — Wave H phase close: CLAUDE.md QG-07 + STATE.md cursor + REQUIREMENTS.md traceability flips + SURPRISES.md update + verifier subagent verdict GREEN (QG-06 + QG-07)

### Phase 61: Subjective gates skill + freshness TTL enforcement (v0.12.0)

**Goal:** Subjective gates (cold-reader hero clarity, install positioning, headline-numbers sanity) become first-class catalog citizens with TTL freshness enforcement. `quality/catalogs/subjective-rubrics.json` ships with seed rubrics — `cold-reader-hero-clarity`, `install-positioning`, `headline-numbers-sanity` — each with a numeric scoring rubric and `freshness_ttl: 30d` default. The `reposix-quality-review` skill ships at `.claude/skills/reposix-quality-review/SKILL.md`: it reads the catalog, dispatches one unbiased subagent per stale/unverified row in parallel (per OP-2), persists JSON artifacts to `quality/reports/verifications/subjective/`, updates the catalog row's `last_verified` timestamp + `status`. It integrates the existing `doc-clarity-review` skill as one rubric implementation (the cold-reader rubric). SUBJ-03 wires the skill into pre-release cadence so subjective gates with TTL ≥ 14d expired auto-dispatch before any milestone tag push — a release cannot ship with stale subjective rows. Operating-principle hooks: **OP-2 aggressive subagent delegation** — one rubric = one isolated subagent, parallel dispatch; **OP-6 ground truth obsession** — TTL is the explicit "this row's freshness is proof, not history" contract.

**Requirements:** SUBJ-01, SUBJ-02, SUBJ-03

**Depends on:** P60 GREEN. **Gate-state precondition:** P60's docs-build-dimension catalog shows GREEN in `quality/reports/verdicts/p60/`, AND the runner cadence cutover is complete (subjective gates compose into the same runner contract as a `pre-release`-cadence consumer).

**Success criteria:**
1. `quality/catalogs/subjective-rubrics.json` exists with at minimum 3 seed rows: `cold-reader-hero-clarity`, `install-positioning`, `headline-numbers-sanity`. Each row carries a numeric scoring rubric, `freshness_ttl` (default 30d), `last_verified` timestamp, and `status`.
2. `.claude/skills/reposix-quality-review/SKILL.md` ships with frontmatter `name: reposix-quality-review` and a one-line description. The skill reads the rubrics catalog, dispatches one subagent per stale/unverified row IN PARALLEL, persists per-row JSON artifacts to `quality/reports/verifications/subjective/<rubric-id>/<ts>.json`, updates catalog row.
3. The skill integrates the existing `doc-clarity-review` skill as the implementation of the `cold-reader-hero-clarity` rubric (rubric → skill mapping documented in the catalog row).
4. `quality/runners/run.py --cadence pre-release` invokes the `reposix-quality-review` skill against any rubric whose `last_verified + freshness_ttl < now`, blocks the runner exit on RED, and writes a verdict to `quality/reports/verdicts/pre-release/<ts>.md`.
5. Synthetic regression test: backdate a rubric's `last_verified` past its TTL, run `--cadence pre-release`, confirm the skill auto-dispatches a fresh review and updates the row.
6. **Recurring (catalog-first):** Catalog rows for SUBJ-01..03 land in `quality/catalogs/subjective-rubrics.json` BEFORE the skill commits.
7. **Recurring (CLAUDE.md):** CLAUDE.md "Cold-reader pass on user-facing surfaces" section updated to point at the new skill + the rubric catalog; the §"Subagent delegation rules" gains the `reposix-quality-review` parallel-dispatch pattern. In the same PR.
8. **Recurring (verifier dispatch):** Phase close: unbiased verifier subagent grades all catalog rows GREEN; verdict in `quality/reports/verdicts/p61/<ts>.md`. (Verifier in this case grades catalog mechanics, not the rubrics themselves — the rubrics are graded by the skill it dispatches.)

**Context anchor:** `.planning/REQUIREMENTS.md` § "Subjective gates" SUBJ-01..03, `.planning/research/v0.12.0/naming-and-architecture.md` § "subjective gates" (skill + TTL design), existing `.claude/skills/doc-clarity-review/SKILL.md` (integrated as one rubric implementation), CLAUDE.md "Cold-reader pass" section (the convention the catalog encodes).

### Phase 62: Repo-org-gaps cleanup — close the v0.11.1 audit (v0.12.0)

**Goal:** Audit `.planning/research/v0.11.1/repo-organization-gaps.md` against current state and close out every remaining gap as either a fix + structure-dimension catalog row that prevents recurrence, or an explicit waiver with reason. The repo-organization-gaps doc is a forgotten todo list if not actioned; this phase ensures every gap becomes a tracked catalog row in the new framework (so a future gap audit doesn't have to be a manual document grep). Operating-principle hooks: **OP-4 self-improving infrastructure** — each gap that recurred under v0.11.x is evidence of a missing structure gate, which this phase backfills; **OP-6 ground truth obsession** — "fixed in CLAUDE.md but not enforced" is not a fix; only a catalog row + verifier counts.

**Requirements:** ORG-01

**Depends on:** P57 GREEN (structure dimension must exist), P61 GREEN (so the repo-org gaps can route into the now-mature dimension set, not into a half-built framework). **Gate-state precondition:** P61's subjective-gates catalog shows GREEN in `quality/reports/verdicts/p61/`. (P62 is also independent enough to slot earlier if scheduling demands it, but the natural order is "polish the framework first, then sweep gaps into it.")

**Success criteria:**
1. Every gap in `.planning/research/v0.11.1/repo-organization-gaps.md` has a status: `closed-by-catalog-row` (gap fixed + recurrence prevented by a structure-dimension catalog row), `closed-by-existing-gate` (gap fixed + already covered by an earlier P57/P58/P60 gate), or `waived` (explicit `quality/catalogs/orphan-scripts.json` or `quality/catalogs/waivers.json` row with reason + dimension_owner + RFC3339 `until`).
2. The audit results are committed under `quality/reports/verifications/repo-org-gaps/<ts>.md` with a row per gap and its closure path.
3. The `.planning/research/v0.11.1/repo-organization-gaps.md` document gets a top-banner update naming "fully audited and closed under P62; see `quality/reports/verifications/repo-org-gaps/<ts>.md` for per-gap closure" — the document is no longer a forgotten todo list.
4. New structure-dimension catalog rows added under `quality/catalogs/freshness-invariants.json` (or a new `repo-org.json`) for each gap that needed a recurrence guard.
5. **Recurring (catalog-first):** ORG-01 catalog row + the per-gap closure rows land BEFORE the audit-fix commits.
6. **Recurring (CLAUDE.md):** CLAUDE.md updated to cite the audit closure + new recurrence-guard rows + waivers (if any) in the appropriate freshness-invariant or workspace-layout sections. In the same PR.
7. **Recurring (verifier dispatch):** Phase close: unbiased verifier subagent grades all catalog rows GREEN; verdict in `quality/reports/verdicts/p62/<ts>.md`.

**Context anchor:** `.planning/REQUIREMENTS.md` § "Repo-org cleanup" ORG-01, `.planning/research/v0.11.1/repo-organization-gaps.md` (the audit document being closed), existing `quality/gates/structure/` rows from P57 (the recurrence-guard infrastructure these gaps will route into).

**Plans:** 6 plans
- [ ] 62-01-PLAN.md — Wave 1 catalog-first commit: 3 structure rows + dim README delta (ORG-01 + POLISH-ORG contract)
- [ ] 62-02-PLAN.md — Wave 2 execute audit: render quality/reports/audits/repo-org-gaps.md + scripts/check_repo_org_gaps.py verifier (ORG-01)
- [ ] 62-03-PLAN.md — Wave 3 POLISH-ORG fix wave: relocate top-level audits + archive SESSION-END-STATE + purge __pycache__ + extend structure verifier
- [ ] 62-04-PLAN.md — Wave 4 SURPRISES.md rotation (302→<=200; archive P57+P58 to SURPRISES-archive-2026-Q2.md)
- [ ] 62-05-PLAN.md — Wave 5 CLAUDE.md QG-07 P62 subsection + audit-doc closure banner + STATE/REQUIREMENTS flips
- [ ] 62-06-PLAN.md — Wave 6 verifier subagent dispatch (Path A or Path B) + verdict GREEN

### Phase 63: Retire migrated sources + final CLAUDE.md cohesion + v0.12.1 carry-forward (v0.12.0)

**Goal:** Final close-out. After SIMPLIFY-01..12 complete and the new system has shown parity in `quality/reports/verdicts/` for two pre-push cycles, delete the migrated source files (anything still kept as a thin shim documents the reason in a header comment per OP-5 reversibility — but the reason has to be real). MIGRATE-01 captures this final retirement. MIGRATE-02 is the cohesion pass on CLAUDE.md: the full dimension/cadence/kind taxonomy section + the meta-rule extension ("when an owner catches a miss: fix the issue, update CLAUDE.md, AND tag the dimension"). Cross-references to all `quality/PROTOCOL.md` sections are audited (per-phase QG-07 updates landed throughout — MIGRATE-02 is the final audit, NOT the only update). MIGRATE-03 files the v0.12.1 carry-forward as stub catalog rows + REQUIREMENTS.md placeholders: perf-dimension full implementation (SIMPLIFY-11 stubs become real gates; latency vs headline-copy cross-check, token-economy bench cross-check), security-dimension stubs (allowlist enforcement gate, audit immutability test), cross-platform container rehearsals (windows-2022, macos-14), AND completion of the `Error::Other` 156→144 partial migration (POLISH2-09 carry-forward from v0.11.1). SIMPLIFY-12 is the final scripts/-tree audit: `find scripts/ -maxdepth 1 -type f | grep -v hooks | grep -v install-hooks.sh` returns empty (or every remaining file has an explicit waiver). Operating-principle hooks: **OP-4 self-improving infrastructure** — the framework that started as scripts is now the system that supersedes them; the `scripts/` dir is the leanest it has ever been; **OP-5 reversibility** — every retirement has a one-cycle parallel proof in `quality/reports/verdicts/`; **OP-1 close the feedback loop** — milestone close requires `gh run view` GREEN AND the QG-06 verifier subagent verdict GREEN AND the audit document closed.

**Requirements:** MIGRATE-01, MIGRATE-02, MIGRATE-03, SIMPLIFY-12

**Depends on:** P56, P57, P58, P59, P60, P61, P62 ALL GREEN. **Gate-state precondition:** every prior phase's verdict file exists and shows GREEN; `quality/reports/verdicts/pre-push/` shows two consecutive GREEN runs across the full runner; `quality/reports/verdicts/pre-pr/` and `quality/reports/verdicts/pre-release/` runner contracts are exercised at least once each.

**Success criteria:**
1. **SIMPLIFY-12 audit:** `find scripts/ -maxdepth 1 -type f | grep -v hooks | grep -v install-hooks.sh` returns empty, OR every remaining file has an explicit `quality/catalogs/orphan-scripts.json` waiver row with reason + dimension_owner + RFC3339 `until` date. `examples/*/run.sh` all have catalog rows. The `examples/` README documents that each example is a tracked gate (already landed in P59; this is the final assertion).
2. **MIGRATE-01:** Every source file flagged by SIMPLIFY-01..12 is either deleted, reduced to a one-line shim with a header-comment reason, or has an explicit waiver row. Two pre-push cycles' worth of `quality/reports/verdicts/pre-push/` show parity-or-better vs the v0.11.x baseline (no behaviour regression introduced by the migration).
3. **MIGRATE-02 cohesion pass:** CLAUDE.md gains a full dimension/cadence/kind taxonomy section (cross-referenced from `quality/PROTOCOL.md`) and the meta-rule extension ("when an owner catches a miss: fix the issue, update CLAUDE.md, AND tag the dimension"). All per-phase QG-07 updates land coherently (no orphan paragraphs, no contradictions). A subagent grep audit of CLAUDE.md against `quality/PROTOCOL.md` finds zero stale cross-refs.
4. **MIGRATE-03 v0.12.1 carry-forward:** stub catalog rows ship for perf-dimension (`quality/catalogs/perf-targets.json` with stubs for latency vs headline-copy + token-economy bench cross-check), security-dimension (`quality/catalogs/security-gates.json` with stubs for allowlist enforcement + audit-immutability), cross-platform container rehearsals (`quality/catalogs/cross-platform.json` with stubs for windows-2022 + macos-14). REQUIREMENTS.md (or a v0.12.1-phases file) is updated with placeholders: PERF-*, SEC-*, CROSS-*, and the `Error::Other` 156→144 completion item.
5. **CHANGELOG `[v0.12.0]` finalized:** summarizes Phases 56–63 + lists every shipped REQ-ID + names the v0.12.1 carry-forward.
6. **Tag gate authored:** a milestone-tag script is in place at `.planning/milestones/v0.12.0-phases/tag-v0.12.0.sh` (or equivalent path) mirroring v0.11.x tag-script safety guards (≥6 guards: clean tree, on `main`, version match, CHANGELOG entry exists, tests green, signed tag). Owner runs the tag — orchestrator does NOT push the tag.
7. **SIMPLIFY absorption (P63 close-out):** every script flagged by SIMPLIFY-01..12 has an actioned status (folded / shimmed / waived); no orphan remainder.
8. **Recurring (catalog-first):** Catalog rows for MIGRATE-01..03 + SIMPLIFY-12 land in `quality/catalogs/{orphan-scripts,perf-targets,security-gates,cross-platform}.json` BEFORE the source-file deletions and the CLAUDE.md cohesion edit.
9. **Recurring (CLAUDE.md):** CLAUDE.md final cohesion pass landed (this IS MIGRATE-02 — the per-phase incremental updates from P56–P62 get audited and stitched here). In the same PR.
10. **Recurring (verifier dispatch):** Phase close: unbiased verifier subagent grades all catalog rows GREEN AND audits cross-phase coherence (every phase's verdict file exists and is GREEN, no orphan catalog rows, no expired waivers without follow-up); verdict in `quality/reports/verdicts/p63/<ts>.md`. The milestone-close verdict is the union of P56..P63 verdicts.
11. **Broaden-and-deepen close-out:** Every dimension's catalog has zero NOT-VERIFIED P0+P1 rows; every WAIVED row has a non-expired `until` (RFC3339) and a tracked carry-forward entry under MIGRATE-03 v0.12.1 placeholders; the milestone-close verifier subagent confirms cross-dimension coherence (no dimension shipped a gate without sweeping its first-run REDs per POLISH-* in REQUIREMENTS.md). Owner directive: "after this milestone the codebase is pristine and high quality across all the dimensions."

**Context anchor:** `.planning/REQUIREMENTS.md` § "Migration close-out" MIGRATE-01..03 + § "Aggressive simplification" SIMPLIFY-12 + § "Out of Scope" (the v0.12.1 carry-forward list), `.planning/research/v0.12.0/autonomous-execution-protocol.md` (parallel migration + hard-cut + cohesion-pass design), v0.11.x tag-script precedent at `.planning/milestones/v0.11.0-phases/tag-v0.11.0.sh` (template for v0.12.0 tag script).

### Phase 64: Docs-alignment dimension — framework, CLI, skill, hook wiring (v0.12.0)

**Goal:** Build the docs-alignment quality dimension end-to-end so that P65 can run a backfill on top of it. Ship the `crates/reposix-quality/` workspace crate with a `clap` subcommand surface (`bind`, `propose-retire`, `confirm-retire` env-guarded against agent contexts, `mark-missing-test`, `plan-refresh`, `plan-backfill`, `merge-shards`, `walk`, `status`, `verify`, `run --gate/--cadence`); a `syn`-based hash binary at `quality/gates/docs-alignment/hash_test_fn` that hashes function bodies token-stream (whitespace + comments normalized away); a skill at `.claude/skills/reposix-quality-doc-alignment/` mirroring the P61 `reposix-quality-review` shape with `SKILL.md`, `refresh.md`, `backfill.md`, and `prompts/{extractor,grader}.md`; slash commands `/reposix-quality-refresh <doc-file>` and `/reposix-quality-backfill` (top-level only); pre-push hook integration via `reposix-quality run --cadence pre-push` invoking the deterministic hash walker; the empty-state catalog `quality/catalogs/doc-alignment.json` with summary block + zero rows + floor=0.50; three structure-dimension freshness rows asserting catalog presence/shape/floor-monotonicity; the two project-wide principles in `quality/PROTOCOL.md` ("subagents propose with citations; tools validate and mint" / "tools fail loud, structured, agent-resolvable") with cross-tool examples; CLAUDE.md updates (new `docs-alignment` row in the dimension matrix, "orchestration-shaped vs implementation-shaped phases" note under Subagent delegation rules, P64 H3 subsection ≤40 lines). Source-of-truth handover bundle: `.planning/research/v0.12.0-docs-alignment-design/{README,01-rationale,02-architecture,03-execution-modes,04-overnight-protocol,05-p64-infra-brief,06-p65-backfill-brief}.md`. **Execution mode:** `executor` (delegates to `gsd-executor`).

**Requirements:** DOC-ALIGN-01, DOC-ALIGN-02, DOC-ALIGN-03, DOC-ALIGN-04, DOC-ALIGN-05, DOC-ALIGN-06, DOC-ALIGN-07.

**Depends on:** P56, P57, P58, P59, P60, P61, P62, P63 ALL GREEN. (P64 builds on the v0.12.0 framework; cannot run before P63 cohesion is in place.)

**Success criteria:**
1. **Catalog-first commit (recurring QG-06):** First commit lands `quality/catalogs/doc-alignment.json` (empty-state schema, summary block) + 3 new structure-dimension rows in `quality/catalogs/freshness-invariants.json` (`structure/doc-alignment-catalog-present`, `structure/doc-alignment-summary-block-valid`, `structure/doc-alignment-floor-not-decreased`).
2. **Crate skeleton:** `crates/reposix-quality/` registered in workspace `Cargo.toml`; `#![forbid(unsafe_code)]` + `#![warn(clippy::pedantic)]`; `cargo clippy -p reposix-quality -- -D warnings` clean; `cargo fmt -- --check` clean; `cargo test -p reposix-quality` PASS.
3. **CLI surface complete:** every subcommand from `02-architecture.md` § "Binary surface" present and documented in `--help`; `bind` validates citations against the live filesystem and refuses on invalid; `confirm-retire` exits non-zero when `$CLAUDE_AGENT_CONTEXT` is set (test asserts this).
4. **Hash binary:** `quality/gates/docs-alignment/hash_test_fn` compiles; tests cover comment-edit invariance (hash unchanged) AND rename detection (hash differs).
5. **Hash walker (`walk` subcommand):** reads catalog, computes current hashes from filesystem, sets `last_verdict` to BOUND or one of the STALE_* states; NEVER updates stored hashes; exits non-zero on any blocking state with a stderr message naming the relevant slash command.
6. **`merge-shards` golden tests:** auto-resolve case (same claim cited from two source files → one row, two `source` citations) AND conflict case (same claim, different test bindings → exit non-zero, write `CONFLICTS.md`, do NOT partial-write the catalog).
7. **Skill + slash commands:** `.claude/skills/reposix-quality-doc-alignment/` populated; both slash commands documented as top-level-only with the depth-2 + subscription rationale in `SKILL.md`.
8. **Pre-push integration:** `reposix-quality run --cadence pre-push` invokes the walker; exits non-zero on blocking states; stderr names the slash command; pre-push hook updated; `test-pre-push.sh` PASS for all existing scenarios.
9. **`quality/PROTOCOL.md`** gains the two project-wide principles section with cross-tool examples (the section MUST list the cross-tool examples enumerated in `02-architecture.md`).
10. **CLAUDE.md update (recurring QG-07):** new `docs-alignment` row in the dimension matrix; "orchestration-shaped phases" note added under Subagent delegation rules; P64 H3 subsection ≤40 lines, banned-words clean, total file ≤40 KB.
11. **All cadences GREEN:** `python3 quality/runners/run.py --cadence pre-push` and `--cadence pre-pr` exit 0; new structure-dimension rows PASS; existing rows unchanged.
12. **Verifier subagent dispatch (recurring QG-06):** Path B in-session per project precedent (Task unavailable in executor); verdict at `quality/reports/verdicts/p64/VERDICT.md` GREEN.

**Context anchor:** `.planning/research/v0.12.0-docs-alignment-design/05-p64-infra-brief.md` (full implementation spec consumed by the planner), `.planning/research/v0.12.0-docs-alignment-design/02-architecture.md` (catalog schema + binary surface + skill layout — load-bearing), `.planning/research/v0.12.0-docs-alignment-design/03-execution-modes.md` (orchestration-vs-implementation marker convention), `.planning/research/v0.12.0-docs-alignment-design/04-overnight-protocol.md` (deadline 08:00, suspicion-of-haste rule, cargo-memory-budget reminders), the existing `.claude/skills/reposix-quality-review/` skill (P61 — copy this shape).

### Phase 65: Docs-alignment backfill — surface the punch list (v0.12.0)

**Goal:** Run the doc-alignment extractor across all current docs (`docs/**/*.md`, `README.md`) and archived REQUIREMENTS.md from prior milestones (v0.6.0–v0.11.0) to populate the catalog. Output is a reviewable punch list of `MISSING_TEST` and `RETIRE_PROPOSED` rows clustered by user-facing surface (Confluence backend parity, JIRA shape, ease-of-setup, outbound HTTP allowlist behavior, etc.). The Confluence page-tree-symlink regression that motivated the milestone surfaces here as a `MISSING_TEST` row; v0.12.1's gap-closure phases close it. **Execution mode:** **`top-level`** — NOT `/gsd-execute-phase`. The orchestrator IS the executor for this phase because depth-2 subagent spawning is unreachable and `gsd-executor` lacks the `Task` tool. The autonomous orchestrator follows the protocol verbatim from `06-p65-backfill-brief.md`: run `plan-backfill`, dispatch ~25–35 shard subagents in waves of 8 (Haiku tier; ≤3 files per shard; directory-affinity sharding), run `merge-shards` (deterministic), resolve any `CONFLICTS.md` by editing shard JSONs and re-running, write `PUNCH-LIST.md`, dispatch the verifier subagent (Path A — top-level has Task), commit phase-close, update CLAUDE.md, then dispatch the milestone-close verifier and STOP (owner pushes the tag).

**Requirements:** DOC-ALIGN-08, DOC-ALIGN-09, DOC-ALIGN-10.

**Depends on:** P64 (framework, CLI, skill, hook wiring all live and verifier-graded GREEN).

**Success criteria:**
1. **`MANIFEST.json` committed first:** deterministic chunker output at `quality/reports/doc-alignment/backfill-<ts>/MANIFEST.json` lists every shard, every input file, in stable alphabetical order. Re-running the chunker on the same inputs produces byte-identical output.
2. **Per-shard subagent dispatch:** every shard from MANIFEST gets a Task subagent (Haiku tier). Each subagent's only durable output is `reposix-quality doc-alignment <subcmd>` calls — no hand-written JSON, no prose summary that the orchestrator interprets via LLM. The orchestrator's context only sees per-shard summary stdout strings (`shard 005: 22 rows, 3 MISSING_TEST, 0 RETIRE_PROPOSED`).
3. **`merge-shards` exits 0 OR conflicts resolved before commit:** if `merge-shards` exits non-zero, orchestrator reads `CONFLICTS.md`, edits the relevant shard JSONs, re-runs. If conflicts persist after 2 rounds, halt and write a checkpoint to `.planning/STATE.md`. Do NOT partial-commit the catalog.
4. **Catalog populated:** `quality/catalogs/doc-alignment.json` summary block reflects extracted state — expected `claims_total` in 100–200 range; if outside that envelope, halt and investigate (over-aggressive or under-aggressive extraction). `alignment_ratio` computed correctly.
5. **Floor waiver written if needed:** if `alignment_ratio < 0.50`, a `summary.floor_waiver` block is written with `until=2026-07-31`, `reason="initial backfill; gap closure phased in v0.12.1"`, `dimension_owner="reuben"`. The walker honors floor_waiver.
6. **`PUNCH-LIST.md` generated:** at `quality/reports/doc-alignment/backfill-<ts>/PUNCH-LIST.md`. MISSING_TEST rows clustered by user-facing surface (≥3 clusters expected); RETIRE_PROPOSED rows listed separately for human review; cluster counts + claim citations preserved.
7. **CLAUDE.md update (recurring QG-07):** P65 H3 subsection ≤40 lines summarizing backfill counts + carry-forward to v0.12.1 + the `floor_waiver` rationale; banned-words clean; total file ≤40 KB.
8. **All cadences GREEN:** `python3 quality/runners/run.py --cadence pre-push` and `--cadence pre-pr` exit 0 (with the floor_waiver in place if needed).
9. **Verifier subagent dispatch (recurring QG-06):** Path A (top-level orchestrator HAS Task); verdict at `quality/reports/verdicts/p65/VERDICT.md` GREEN. Verifier checks the catalog matches the shard outputs + the punch list reflects the catalog state + no auto-resolved retirements (every `RETIRE_PROPOSED` is genuinely supersession-cited).
10. **Milestone-close verifier dispatched and GREEN:** `quality/reports/verdicts/milestone-v0.12.0/VERDICT.md` confirms all P56–P65 catalog rows GREEN-or-WAIVED, tag-gate guards in `.planning/milestones/v0.12.0-phases/tag-v0.12.0.sh` all pass (re-run after P64+P65 changes), CHANGELOG `[v0.12.0]` finalized including DOC-ALIGN-* shipped.
11. **STOP at tag boundary:** orchestrator does NOT push the tag. STATE.md cursor is updated to "v0.12.0 ready-to-tag (re-verified after P64+P65); owner pushes tag."

**Context anchor:** `.planning/research/v0.12.0-docs-alignment-design/06-p65-backfill-brief.md` (full top-level execution protocol — normative; orchestrator follows verbatim), `.planning/research/v0.12.0-docs-alignment-design/03-execution-modes.md` (why this is top-level), `.planning/research/v0.12.0-docs-alignment-design/04-overnight-protocol.md` (deadline + suspicion-of-haste rule).

---

## Previously planned milestones

Per CLAUDE.md §0.5 / Workspace layout, each shipped/historical milestone's
ROADMAP.md lives inside its `*-phases/` directory. Top-level ROADMAP.md
holds ONLY the active milestone (currently v0.12.0) + this index.

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
