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

**Plans:**
- **79-01** — POC-01 (throwaway POC at `research/v0.13.0-dvcs/poc/`). **SHIPPED 2026-05-01** (5 commits 660bae0..4e6de2b; SUMMARY at `.planning/phases/79-poc-reposix-attach-core/79-01-SUMMARY.md`; FINDINGS at `research/v0.13.0-dvcs/poc/POC-FINDINGS.md` — 5 INFO + 2 REVISE + 0 SPLIT routing tags).
- **79-02** — DVCS-ATTACH-01..02 (scaffold + cache reconciliation module). PENDING orchestrator's POC-FINDINGS re-engagement decision.
- **79-03** — DVCS-ATTACH-02..04 (tests + idempotency + close). PENDING 79-02.

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
