# Requirements — Active milestone: v0.13.0 DVCS over REST

**Active milestone:** v0.13.0 DVCS over REST (planning_started 2026-04-30).

**Previously validated milestones — see per-milestone REQUIREMENTS.md:**
- v0.12.x Quality Gates + Carry-forwards — `.planning/milestones/v0.12.0-phases/REQUIREMENTS.md` (v0.12.0 SHIPPED 2026-04-28, Phases 56–65) + `.planning/milestones/v0.12.1-phases/REQUIREMENTS.md` (v0.12.1 SHIPPED 2026-04-30, Phases 72–77).
- v0.11.x Polish & Reproducibility — `.planning/milestones/v0.11.0-phases/REQUIREMENTS.md` (v0.11.0 SHIPPED 2026-04-25; v0.11.1 + v0.11.2 polish passes SHIPPED 2026-04-26 / 2026-04-27).
- v0.10.0 Docs & Narrative Shine — `.planning/milestones/v0.10.0-phases/REQUIREMENTS.md` (SHIPPED 2026-04-25, Phases 40–45).
- v0.9.0 Architecture Pivot — `.planning/milestones/v0.9.0-phases/REQUIREMENTS.md` (SHIPPED 2026-04-24, Phases 31–36; ARCH-01..19).
- v0.8.0 and earlier — see `.planning/milestones/v0.X.0-phases/ARCHIVE.md`.

> **Convention.** Per CLAUDE.md §0.5 / Workspace layout, each milestone's REQUIREMENTS.md lives inside its `*-phases/` directory once shipped. The top-level file holds ONLY the active milestone + this index. Enforced by `quality/gates/structure/top-level-requirements-roadmap-scope.sh` (QG-08, shipped P57).

---

## v0.13.0 Requirements — DVCS over REST

**Milestone goal:** Shift the project thesis from "VCS over REST" (one developer, one backend) to "DVCS over REST" — confluence (or any one issues backend) remains the source of truth, but a plain-git mirror on GitHub becomes the universal-read surface for everyone else. Devs `git clone git@github.com:org/repo.git` with **vanilla git, no reposix install**, get all markdown, edit, commit. Install reposix only when they want to write back; `reposix attach` reconciles their existing checkout against the SoT, then `git push` via a bus remote fans out atomically to confluence (SoT-first) and the GH mirror.

The litmus test: the dual-side round-trip in `vision-and-mental-model.md` § "The thing we are building" works end-to-end with no manual sync — vanilla-git clone → attach → edit → bus-push → webhook-driven mirror catch-up after a browser-side confluence edit, with conflict detection in both directions.

**Mental model.** Three roles in a v0.13.0 deployment:
- **SoT-holder** (Dev A) — reposix-equipped, attached via `init`. Reads from confluence (cache-backed). Writes via bus remote.
- **Mirror-only consumer** (Dev B before installing reposix) — vanilla git only. Reads from GH mirror. Cannot write back.
- **Round-tripper** (Dev B after `reposix attach`) — reposix-equipped, attached after the fact. Fast clones from GH mirror; ground-truth reads from confluence; writes via bus remote.

Bus remote: precheck-then-SoT-first-write. Cheap network checks (`ls-remote` mirror, `list_changed_since` on SoT) bail before reading stdin; on success, REST-write to SoT then `git push` to mirror; mirror-write failure leaves "mirror lag" recoverable on next push, not data loss. Mirror-lag observability via plain-git refs (`refs/mirrors/confluence-head`, `refs/mirrors/confluence-synced-at`) — vanilla `git fetch` brings them along; `git log` shows staleness.

**Source-of-truth handover bundle (read these BEFORE planning Phase 1):**
- `.planning/research/v0.13.0-dvcs/vision-and-mental-model.md` — the thesis + success gates + risks.
- `.planning/research/v0.13.0-dvcs/architecture-sketch.md` — technical design + open questions; performance subtlety on `list_records` walk.
- `.planning/research/v0.13.0-dvcs/kickoff-recommendations.md` — pre-kickoff readiness moves.
- `.planning/research/v0.13.0-dvcs/decisions.md` — 15 architecture-sketch open questions ratified 2026-04-30.
- `.planning/milestones/v0.13.0-phases/CARRY-FORWARD.md` — `MULTI-SOURCE-WATCH-01`, `GIX-YANKED-PIN-01`, `WAIVED-STRUCTURE-ROWS-03`, `POC-DVCS-01`.
- `.planning/research/v0.14.0-observability-and-multi-repo/vision-and-mental-model.md` — the next milestone's pre-roadmap scope; tells the v0.13.0 ROADMAP what NOT to absorb.

**Operating-principle hooks (non-negotiable, per project CLAUDE.md):**
- **OP-1 Simulator-first.** All v0.13.0 phases run end-to-end against the simulator. Two simulator instances in one process serve as "confluence-shaped SoT" + "GitHub-shaped mirror" for tests. Real-backend tests (TokenWorld + reubenjohn/reposix) gate the milestone close, not individual phase closes.
- **OP-2 Tainted-by-default.** Mirror writes carry tainted bytes from the SoT. The GH mirror's frontmatter must preserve `Tainted<T>` semantics. The `attach` cache marks all materialized blobs as tainted.
- **OP-3 Audit log non-optional.** Every bus-remote push writes audit rows to BOTH tables — cache audit (helper RPC turn) + backend audit (SoT REST mutation). The mirror push writes a cache-audit row noting "mirror lag now zero" or "mirror lag now N." Webhook-driven syncs write cache-audit rows too.
- **OP-7 Verifier subagent dispatch on every phase close.** The DVCS round-trip test is a catalog row in dimension `agent-ux`, kind `subagent-graded`, cadence `pre-pr`.
- **OP-8 +2 phase practice.** v0.13.0 reserves last 2 phases for surprises absorption + good-to-haves polish.
- **Per-phase push cadence (codified 2026-04-30).** Every phase closes with `git push origin main` BEFORE verifier-subagent dispatch. Pre-push gate-passing is part of phase-close criterion. Closes backlog 999.4.

### Active

#### Pre-DVCS hygiene (P0)

- [ ] **HYGIENE-01**: Bump `gix` off yanked `=0.82.0` baseline. `gix-actor 0.40.1` also yanked. Update `crates/*/Cargo.toml` to next non-yanked release; align all gix-family `=`-pins; `cargo check --workspace` GREEN; `cargo nextest run --workspace` GREEN (per-crate if memory pressure); update `CLAUDE.md` § Tech stack to cite the new version. Closes GitHub issues #29 + #30. **P0 — load-bearing pin sitting on a yanked version.**
- [ ] **HYGIENE-02**: Land verifier scripts for the 3 currently-WAIVED structure rows in `quality/catalogs/freshness-invariants.json` BEFORE waivers expire 2026-05-15. Three TINY-shape shell verifiers under `quality/gates/structure/`: (a) `no-loose-top-level-planning-audits.sh` — fail if any audit doc exists outside `.planning/milestones/audits/` or `.planning/archive/`; (b) `no-pre-pivot-doc-stubs.sh` — fail if any `docs/<slug>.md` exists at top-level docs/ with size <500 bytes; (c) `repo-org-audit-artifact-present.sh` — pass if the canonical repo-org-audit artifact exists at the catalog-cited path. Each catalog row flips WAIVED → PASS (waiver block deleted). Tested via `python3 quality/runners/run.py`. **P0/P1 — waiver auto-renewal would defeat catalog-first principle.**

#### POC (pre-Phase-1)

- [x] **POC-01**: End-to-end POC in `research/v0.13.0-dvcs/poc/` exercising the three innovations against the simulator, BEFORE Phase 1 (attach core) PLAN.md is finalized. Throwaway code (NOT v0.13.0 implementation). Specifically demonstrates: (a) `reposix attach` against a working tree with mixed `id`-bearing + `id`-less files (deliberately mangled); (b) bus-remote push observing mirror lag (SoT writes succeed, mirror trailing); (c) cheap-precheck path refusing fast when SoT version mismatches local cache. Ships with `POC-FINDINGS.md` listing algorithm-shape decisions, integration friction, and design questions the architecture sketch did not anticipate — feeds directly into Phase 1's PLAN.md. Time budget: ~1 day; if exceeding 2 days, surface as SURPRISES-INTAKE candidate. **P0 — kickoff-rec #2 readiness move; v0.9.0 precedent saved 3-4 days mid-phase rework.**

#### `reposix attach` core

- [x] **DVCS-ATTACH-01** (shipped P79, 2026-05-01): `reposix attach <backend>::<project>` subcommand exists in `crates/reposix-cli/`. In CWD with no special prerequisites on how the checkout was created: (a) builds a fresh cache directory at the standard location derived from `<backend>::<project>` (NOT from `remote.origin.url`, per Q1.1); (b) REST-lists the backend; populates cache OIDs (filenames + tree structure; blobs lazy on first materialize); (c) reconciles by walking current `HEAD` tree and matching files to backend records by `id` in frontmatter; records matches in cache reconciliation table; (d) adds remote `reposix::<sot-spec>?mirror=<existing-origin-url>` (or `reposix::<sot-spec>` if `--no-bus`); (e) sets `extensions.partialClone=<remote-name>` on the new reposix remote. Existing `origin` (the GH mirror) keeps plain-git semantics.
- [x] **DVCS-ATTACH-02** (shipped P79, 2026-05-01): Reconciliation cases produce the resolutions specified in `architecture-sketch.md` § "Reconciliation cases": match (cache stores OID alignment), backend-record-deleted (warn + skip + offer `--orphan-policy={delete-local,fork-as-new,abort}`), no-id-frontmatter (warn + skip), duplicate `id` (hard error), mirror-lag (cache marks for next fetch). Tested via deliberately-mangled checkouts: each row in the resolution table has a corresponding test case.
- [x] **DVCS-ATTACH-03** (shipped P79, 2026-05-01): Re-attach with different SoT spec is REJECTED with clear error per Q1.2 ("multi-SoT not supported in v0.13.0"). Re-attach with same SoT is IDEMPOTENT per Q1.3 — refreshes cache state against current backend without special-casing init-vs-attach origins.
- [x] **DVCS-ATTACH-04** (shipped P79, 2026-05-01): The cache materialization API used by `attach` (`Cache::read_blob`, the lazy seam git invokes during `extensions.partialClone` fetches) returns `Tainted<Vec<u8>>` per OP-2. Verified by BOTH (a) a static type-system assertion that pins the `Cache::read_blob` signature to `Tainted<Vec<u8>>` (cheap; compile-time guarantee), AND (b) a runtime integration test in `crates/reposix-cli/tests/attach.rs` that exercises `attach` then forces ONE blob materialization via the cache lazy seam and asserts the bytes are tainted. Architectural rationale: `Cache::build_from` is lazy by design (does not pre-materialize blobs); the Tainted contract therefore belongs to the `read_blob` materialization seam, not to `attach` itself. Reframed 2026-05-01 during P79 plan revision per checker B2.

#### Mirror-lag observability

- [x] **DVCS-MIRROR-REFS-01** (shipped P80, 2026-05-01): `refs/mirrors/<sot-host>-head` (direct ref to cache post-write synthesis-commit OID) + `refs/mirrors/<sot-host>-synced-at` (annotated tag with `mirror synced at <RFC3339>` message) helpers shipped in `crates/reposix-cache/src/mirror_refs.rs`. Namespace is `refs/mirrors/...` per Q2.1.
- [x] **DVCS-MIRROR-REFS-02** (shipped P80, 2026-05-01): Existing single-backend push (`handle_export`) wired to update both refs on success + write `audit_events_cache` row with `op = 'mirror_sync_written'` (OP-3 unconditional). Bus push (P83) and webhook sync (P84) will reuse the same cache helpers per Q2.3.
- [x] **DVCS-MIRROR-REFS-03** (shipped P80, 2026-05-01): `handle_export` reject branch reads `read_mirror_synced_at` and emits `(N minutes ago)` rendering when present; first-push case omits the hint cleanly. Test coverage at `crates/reposix-remote/tests/mirror_refs.rs::reject_hint_after_sync_cites_age` + `reject_hint_first_push_omits_synced_at_line` (non-vacuous H3 fix per PLAN-CHECK).

#### Bus remote

- [x] **DVCS-BUS-URL-01** (shipped P82, 2026-05-01): `crates/reposix-remote/src/bus_url.rs::parse` recognizes `reposix::<sot-spec>?mirror=<mirror-url>` per Q3.3; `Route::{Single,Bus}` enum branches at `argv[2]` parse-time; `parse_remote_url` (single-backend) UNCHANGED; `+`-delimited form rejected with verbatim Q3.3 hint; unknown query keys rejected (D-03).
- [x] **DVCS-BUS-PRECHECK-01** (shipped P82, 2026-05-01): `crates/reposix-remote/src/bus_handler.rs::handle_bus_export` runs `git ls-remote -- <mirror> refs/heads/main` (with `-`-prefix reject for T-82-01) before reading stdin; on drift emits verbatim `error refs/heads/main fetch first` + Q3.5 hint; NO writes; NO stdin read.
- [x] **DVCS-BUS-PRECHECK-02** (shipped P82, 2026-05-01): `bus_handler::handle_bus_export` calls `precheck_sot_drift_any(cache, backend, project, rt)` (new 10-line wrapper around L1's `list_changed_since`) before reading stdin; on `Drifted` outcome emits verbatim `error refs/heads/main fetch first` + `git pull --rebase` hint citing mirror-lag refs (when populated); NO writes; NO stdin read.
- [x] **DVCS-BUS-WRITE-01** (shipped P83, 2026-05-01): `bus_handler::handle_bus_export` calls `write_loop::apply_writes(cache, backend, backend_name, project, rt, proto, &parsed)` (P83-01 T02 refactor lift); SoT writes; audit rows in both tables; `last_fetched_at` advanced.
- [x] **DVCS-BUS-WRITE-02** (shipped P83, 2026-05-01): Plain `git push <mirror_remote> main` subprocess after SoT success (NO `--force-with-lease` per D-08). On mirror failure: `helper_push_partial_fail_mirror_lag` audit row (NEW op P83-01 T03); `refs/mirrors/<sot>-head` updated; `synced-at` NOT updated; stderr warn; `ok` returned.
- [x] **DVCS-BUS-WRITE-03** (shipped P83, 2026-05-01): On mirror success: BOTH refs updated; `mirror_sync_written` audit row; `ok refs/heads/main` returned.
- [x] **DVCS-BUS-WRITE-04** (shipped P83, 2026-05-01): NO helper-side retry per Q3.6. Verifier shell `bus-write-no-helper-retry.sh` greps for `--force` tokens (none present).
- [x] **DVCS-BUS-WRITE-05** (shipped P83, 2026-05-01): STEP 0 in `bus_handler::handle_bus_export` bails BEFORE `ensure_cache` when no `git remote add` configured for the mirror URL; verbatim Q3.5 hint emitted; NO cache opened (regression test in `bus_write_no_mirror_remote.rs`).
- [x] **DVCS-BUS-WRITE-06** (shipped P83-02, 2026-05-01): Three fault-injection tests under `crates/reposix-remote/tests/`:
  - `bus_write_mirror_fail.rs` (`#[cfg(unix)]` failing-update-hook fixture; case a)
  - `bus_write_sot_fail.rs` (mid-stream 5xx; case b)
  - `bus_write_post_precheck_409.rs` (post-precheck 409; case c)
  Plus `bus_write_audit_completeness.rs` for OP-3 dual-table assertion.
- [x] **DVCS-BUS-FETCH-01** (shipped P82, 2026-05-01): Capabilities branching at `crates/reposix-remote/src/main.rs:150-172` omits `stateless-connect` for `Route::Bus`; `tests/bus_capabilities.rs` confirms single-backend advertises it AND bus URLs DO NOT.

#### L1 perf migration

- [x] **DVCS-PERF-L1-01** (shipped P81, 2026-05-01): Unconditional `list_records` walk in `handle_export` replaced with `crates/reposix-remote/src/precheck.rs::precheck_export_against_changed_set` (narrow-deps signature; bus handler P82+ reuses without `State` coupling). On the cursor-present hot path the precheck makes one `list_changed_since` REST call + per-id GETs bounded by `changed_set ∩ push_set`. L1-strict delete trade-off RATIFIED: cache is trusted as the prior; backend-side deletes surface as REST 404 on PATCH at write time.
- [x] **DVCS-PERF-L1-02** (shipped P81, 2026-05-01): `reposix sync --reconcile` clap subcommand at `crates/reposix-cli/src/sync.rs` (3-test smoke suite at `crates/reposix-cli/tests/sync.rs`). On-demand full `list_records` walk + cache reconciliation for users who suspect cache desync. Documentation defers to P85 (`docs/concepts/dvcs-topology.md`); CLAUDE.md § Commands carries the inline bullet now.
- [x] **DVCS-PERF-L1-03** (shipped P81, 2026-05-01): N=200 wiremock perf regression at `crates/reposix-remote/tests/perf_l1.rs::l1_precheck_uses_list_changed_since_not_list_records` (+ positive-control sibling that confirms wiremock matcher fails RED when violated). Both single-backend and bus push paths use the same `precheck.rs` module per M1 narrow-deps signature. L2/L3 cache-desync hardening explicitly defers to v0.14.0 per `architecture-sketch.md` § "Performance subtlety" + CLAUDE.md § Architecture L1 paragraph.

#### Webhook-driven mirror sync

- [ ] **DVCS-WEBHOOK-01**: Reference GitHub Action workflow ships at `.github/workflows/reposix-mirror-sync.yml` per `architecture-sketch.md` § "Webhook-driven mirror sync". Triggers: `repository_dispatch` (event type `reposix-mirror-sync`) + cron safety net (default `*/30`, configurable via workflow `vars`).
- [ ] **DVCS-WEBHOOK-02**: Workflow runs `reposix init confluence + git push <mirror>` and updates `refs/mirrors/...` refs. Uses `--force-with-lease` against last known mirror ref so a concurrent bus-push's race doesn't corrupt mirror state.
- [ ] **DVCS-WEBHOOK-03**: First-run handling (no existing mirror refs) is graceful per Q4.3. Empty-mirror case populates refs on first run. Verified by sandbox test against TokenWorld.
- [ ] **DVCS-WEBHOOK-04**: Latency target: < 60s p95 from confluence edit to GH ref update. Measured in sandbox during this phase; if p95 > 120s, document the constraint and tune ref semantics.

#### DVCS docs

- [ ] **DVCS-DOCS-01**: `docs/concepts/dvcs-topology.md` exists. Three roles (SoT-holder, mirror-only consumer, round-tripper) explained with the diagram from `vision-and-mental-model.md`. Mirror-lag refs explained — explicitly: *"`refs/mirrors/confluence-synced-at` is the timestamp the mirror last caught up to confluence, NOT a 'current SoT state' marker"* (per Q2.2). When-to-choose-which-pattern guidance.
- [ ] **DVCS-DOCS-02**: `docs/guides/dvcs-mirror-setup.md` exists. Walk-through of webhook + Action setup for an owner installing v0.13.0 against a confluence space. Backends-without-webhooks fallback documented (cron-only sync; per Q4.2). Cleanup procedure documented.
- [ ] **DVCS-DOCS-03**: Troubleshooting matrix entries cover: bus-remote `fetch first` rejection messages (cite mirror-lag refs as the diagnostic); attach reconciliation warnings; webhook race conditions; cache-desync recovery via `reposix sync --reconcile`.
- [ ] **DVCS-DOCS-04**: Cold-reader pass via `doc-clarity-review` against a reader who has read only `docs/index.md` + `docs/concepts/mental-model-in-60-seconds.md`. Zero critical-friction findings before milestone close.

#### Dark-factory regression — third arm

- [ ] **DVCS-DARKFACTORY-01**: Extend `quality/gates/agent-ux/dark-factory.sh` (formerly `scripts/dark-factory-test.sh`) to add a third subprocess-agent transcript: a fresh agent given only the GH mirror URL + a goal completes vanilla-clone + `reposix attach` + edit + bus-push end-to-end with zero in-context learning beyond what the helper's stderr teaches. Reuses the existing dark-factory test harness; no in-prompt instruction beyond the goal statement.
- [ ] **DVCS-DARKFACTORY-02**: Catalog row in dimension `agent-ux`, kind `subagent-graded`, cadence `pre-pr`. Verifier grades from artifacts with zero session context per OP-7.

#### Carry-forward

- [ ] **MULTI-SOURCE-WATCH-01**: From v0.12.1 P75. Walker hashes every source citation in a `Source::Multi` row, ANDs results; row enters `STALE_DOCS_DRIFT` on ANY index drift. Schema migration: `source_hash: Option<String>` → `source_hashes: Vec<String>` parallel-array on `Source::Multi` rows. `verbs::bind` writes/preserves all entries on the parallel array. `serde(default)` + one-time backfill migrates the populated 388-row catalog. Regression tests at `crates/reposix-quality/tests/walk.rs::walk_multi_source_*` exercise the path-(b) "non-first source drift fires STALE" case. Acceptance per `.planning/milestones/v0.13.0-phases/CARRY-FORWARD.md`.

### +2 reservation (per OP-8)

- [ ] **DVCS-SURPRISES-01**: Surprises-absorption phase drains `.planning/milestones/v0.13.0-phases/SURPRISES-INTAKE.md`. Each entry → RESOLVED | DEFERRED | WONTFIX with commit SHA or rationale. Verifier honesty spot-check on previous phases' plans + verdicts (empty intake acceptable IF phases produced explicit `Eager-resolution` decisions).
- [ ] **DVCS-GOOD-TO-HAVES-01**: Good-to-haves polish phase drains `GOOD-TO-HAVES.md`. XS items always close; M items default-defer to v0.14.0.

### Out of Scope (deferred to v0.14.0)

- **OTel / `reposix tail` / multi-project helper.** Operational maturity for an existing thesis. Doesn't depend on DVCS shipping; equally, DVCS doesn't depend on it. Lives at `.planning/research/v0.14.0-observability-and-multi-repo/`.
- **Origin-of-truth frontmatter enforcement.** Only matters when bus pattern fans out across **multiple issues backends** (e.g., GH Issues + JIRA simultaneously). v0.13.0 bus pattern is "one issues backend (SoT) + one plain-git mirror" where this can't go wrong.
- **L2/L3 cache-desync hardening.** L2 = background reconcile job; L3 = transactional cache writes wired into every adapter. Decision rate-of-incidence collected via v0.14.0 OTel work.
- **Multi-SoT attach.** Covered by v0.14.0 origin-of-truth scope.
- **Sync daemon as long-running process.** Webhook-driven CI is the v0.13.0 default.
- **Atomic two-phase commit across backends.** Bus remote is "SoT-first, mirror-best-effort with lag tracking," not 2PC. Document the asymmetry; don't try to hide it.
- **Bus remote with N > 2 endpoints.** Algorithm generalizes; URL scheme generalizes; but v0.13.0 implementation hardcodes 1+1.
- **Bidirectional bus** (mirror → SoT propagation). Mirror is read-only from confluence's perspective; vanilla `git push origin` from Dev B to mirror creates commits SoT never sees, lost on next webhook sync via `--force-with-lease`. Documented loudly in `dvcs-topology.md`.
- **Conflict resolution UI / interactive merge against confluence-side edits.** Standard `git pull --rebase` flow handles it.
- **RETROSPECTIVE.md backfill** for v0.9.0 → v0.12.0 (multi-hour synthesis from per-milestone `*-phases/` artifacts). v0.14.0 candidate.

### Traceability

Drafted 2026-04-30 by `gsd-roadmapper`. Coverage: **36/36 v0.13.0 REQ-IDs mapped to exactly one phase** (no orphans, no duplicates). Phases P78–P88; v0.13.0 starts at P78 (continuing from v0.12.1 P77 close 2026-04-30).

| REQ-ID | Phase | Status |
|--------|-------|--------|
| HYGIENE-01 | P78 | planning |
| HYGIENE-02 | P78 | planning |
| MULTI-SOURCE-WATCH-01 | P78 | planning |
| POC-01 | P79 | complete |
| DVCS-ATTACH-01 | P79 | planning |
| DVCS-ATTACH-02 | P79 | planning |
| DVCS-ATTACH-03 | P79 | planning |
| DVCS-ATTACH-04 | P79 | planning |
| DVCS-MIRROR-REFS-01 | P80 | shipped |
| DVCS-MIRROR-REFS-02 | P80 | shipped |
| DVCS-MIRROR-REFS-03 | P80 | shipped |
| DVCS-PERF-L1-01 | P81 | shipped |
| DVCS-PERF-L1-02 | P81 | shipped |
| DVCS-PERF-L1-03 | P81 | shipped |
| DVCS-BUS-URL-01 | P82 | shipped |
| DVCS-BUS-PRECHECK-01 | P82 | shipped |
| DVCS-BUS-PRECHECK-02 | P82 | shipped |
| DVCS-BUS-FETCH-01 | P82 | shipped |
| DVCS-BUS-WRITE-01 | P83 | shipped |
| DVCS-BUS-WRITE-02 | P83 | shipped |
| DVCS-BUS-WRITE-03 | P83 | shipped |
| DVCS-BUS-WRITE-04 | P83 | shipped |
| DVCS-BUS-WRITE-05 | P83 | shipped |
| DVCS-BUS-WRITE-06 | P83 | shipped |
| DVCS-WEBHOOK-01 | P84 | planning |
| DVCS-WEBHOOK-02 | P84 | planning |
| DVCS-WEBHOOK-03 | P84 | planning |
| DVCS-WEBHOOK-04 | P84 | planning |
| DVCS-DOCS-01 | P85 | planning |
| DVCS-DOCS-02 | P85 | planning |
| DVCS-DOCS-03 | P85 | planning |
| DVCS-DOCS-04 | P85 | planning |
| DVCS-DARKFACTORY-01 | P86 | planning |
| DVCS-DARKFACTORY-02 | P86 | planning |
| DVCS-SURPRISES-01 | P87 | planning |
| DVCS-GOOD-TO-HAVES-01 | P88 | planning |

### Recurring success criteria across every v0.13.0 phase

These are part of every phase's definition-of-done and are NOT separate REQ-IDs (they are recurring expressions of OP-7 + the autonomous-execution protocol):
- **Catalog-first**: phase's first commit writes catalog rows BEFORE implementation.
- **CLAUDE.md update in same PR** (per QG-07 carry-over from v0.12.0).
- **Unbiased verifier-subagent dispatch on phase close** (per OP-7).
- **Per-phase push** — `git push origin main` BEFORE verifier-subagent dispatch; pre-push gate-passing is part of close criterion (codified 2026-04-30, closes 999.4).
- **Eager-resolution preference** per OP-8 — items < 1hr / no new dependency get fixed in the discovering phase; else appended to `SURPRISES-INTAKE.md` or `GOOD-TO-HAVES.md`.
- **Goal: pristine codebase across all dimensions** — every dimension's gates GREEN-or-WAIVED at milestone close.
