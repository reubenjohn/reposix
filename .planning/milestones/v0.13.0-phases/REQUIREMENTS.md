# v0.13.0 Requirements — DVCS over REST

**Milestone status:** SHIPPED 2026-05-01 (Phases P78–P88; ready-to-tag).

**Milestone goal:** Shift the project thesis from "VCS over REST" (one developer, one backend) to "DVCS over REST" — confluence (or any one issues backend) remains the source of truth, but a plain-git mirror on GitHub becomes the universal-read surface for everyone else. Devs `git clone git@github.com:org/repo.git` with **vanilla git, no reposix install**, get all markdown, edit, commit. Install reposix only when they want to write back; `reposix attach` reconciles their existing checkout against the SoT, then `git push` via a bus remote fans out atomically to confluence (SoT-first) and the GH mirror.

The litmus test: the dual-side round-trip in `vision-and-mental-model.md` § "The thing we are building" works end-to-end with no manual sync — vanilla-git clone → attach → edit → bus-push → webhook-driven mirror catch-up after a browser-side confluence edit, with conflict detection in both directions.

**Mental model.** Three roles in a v0.13.0 deployment:
- **SoT-holder** (Dev A) — reposix-equipped, attached via `init`. Reads from confluence (cache-backed). Writes via bus remote.
- **Mirror-only consumer** (Dev B before installing reposix) — vanilla git only. Reads from GH mirror. Cannot write back.
- **Round-tripper** (Dev B after `reposix attach`) — reposix-equipped, attached after the fact. Fast clones from GH mirror; ground-truth reads from confluence; writes via bus remote.

Bus remote: precheck-then-SoT-first-write. Cheap network checks (`ls-remote` mirror, `list_changed_since` on SoT) bail before reading stdin; on success, REST-write to SoT then `git push` to mirror; mirror-write failure leaves "mirror lag" recoverable on next push, not data loss. Mirror-lag observability via plain-git refs (`refs/mirrors/confluence-head`, `refs/mirrors/confluence-synced-at`) — vanilla `git fetch` brings them along; `git log` shows staleness.

**Source-of-truth handover bundle:**
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

- [x] **HYGIENE-01**: Bump `gix` off yanked `=0.82.0` baseline (`gix-actor 0.40.1` also yanked). Update `crates/*/Cargo.toml`; align gix-family `=`-pins; workspace check + nextest GREEN; update `CLAUDE.md` § Tech stack. Closes #29 + #30. **P0 — load-bearing pin on yanked version.**
- [x] **HYGIENE-02**: Land verifier scripts for the 3 WAIVED structure rows in `quality/catalogs/freshness-invariants.json` BEFORE waivers expire 2026-05-15. Three TINY-shape verifiers under `quality/gates/structure/`: `no-loose-top-level-planning-audits.sh`, `no-pre-pivot-doc-stubs.sh`, `repo-org-audit-artifact-present.sh`. Each catalog row flips WAIVED → PASS (waiver block deleted). **P0/P1 — waiver auto-renewal would defeat catalog-first principle.**

#### POC (pre-Phase-1)

- [x] **POC-01**: End-to-end POC in `research/v0.13.0-dvcs/poc/` exercising the three innovations against the simulator BEFORE Phase 1 PLAN.md is finalized. Throwaway code (NOT v0.13.0 implementation). Demonstrates: attach against mixed `id`-bearing/`id`-less files; bus-remote push observing mirror lag; cheap-precheck path. Ships with `POC-FINDINGS.md` feeding into Phase 1. **P0 — kickoff-rec #2 readiness move.**

#### `reposix attach` core

- [x] **DVCS-ATTACH-01**: `reposix attach <backend>::<project>` subcommand in `crates/reposix-cli/`. In CWD: builds fresh cache directory derived from `<backend>::<project>` (NOT `remote.origin.url`, per Q1.1); REST-lists backend; populates cache OIDs lazily; reconciles by walking current HEAD tree matching files to backend records by frontmatter `id`; adds remote `reposix::<sot-spec>?mirror=<existing-origin-url>` (or `reposix::<sot-spec>` if `--no-bus`); sets `extensions.partialClone=<remote-name>`. Existing `origin` keeps plain-git semantics.
- [x] **DVCS-ATTACH-02**: Reconciliation cases per `architecture-sketch.md` § "Reconciliation cases": match (OID alignment), backend-deleted (warn+skip+`--orphan-policy={delete-local,fork-as-new,abort}`), no-id (warn+skip), duplicate `id` (hard error), mirror-lag (cache marks for next fetch). Each row has a corresponding test case.
- [x] **DVCS-ATTACH-03**: Re-attach with different SoT spec is REJECTED with clear error per Q1.2 ("multi-SoT not supported in v0.13.0"). Re-attach with same SoT is IDEMPOTENT per Q1.3 — refreshes cache state against current backend without special-casing init-vs-attach origins.
- [x] **DVCS-ATTACH-04**: `Cache::read_blob` (the lazy seam git invokes during `extensions.partialClone` fetches) returns `Tainted<Vec<u8>>` per OP-2. Verified by static type-system assertion + runtime integration test in `crates/reposix-cli/tests/attach.rs`. The Tainted contract belongs to the `read_blob` materialization seam, not to `attach` itself (P79 plan revision per checker B2).

#### Mirror-lag observability

- [x] **DVCS-MIRROR-REFS-01**: `refs/mirrors/<sot-host>-head` (direct ref to cache post-write synthesis-commit OID) + `refs/mirrors/<sot-host>-synced-at` (annotated tag with `mirror synced at <RFC3339>` message) helpers shipped in `crates/reposix-cache/src/mirror_refs.rs`. Namespace is `refs/mirrors/...` per Q2.1.
- [x] **DVCS-MIRROR-REFS-02**: Existing single-backend push (`handle_export`) wired to update both refs on success + write `audit_events_cache` row with `op = 'mirror_sync_written'` (OP-3 unconditional). Bus push (P83) and webhook sync (P84) will reuse the same cache helpers per Q2.3.
- [x] **DVCS-MIRROR-REFS-03**: `handle_export` reject branch reads `read_mirror_synced_at` and emits `(N minutes ago)` rendering when present; first-push case omits the hint cleanly. Test coverage at `crates/reposix-remote/tests/mirror_refs.rs::reject_hint_after_sync_cites_age` + `reject_hint_first_push_omits_synced_at_line` (non-vacuous H3 fix per PLAN-CHECK).

#### Bus remote

- [x] **DVCS-BUS-URL-01**: `crates/reposix-remote/src/bus_url.rs::parse` recognizes `reposix::<sot-spec>?mirror=<mirror-url>` per Q3.3; `Route::{Single,Bus}` enum branches at `argv[2]` parse-time; `parse_remote_url` (single-backend) UNCHANGED; `+`-delimited form rejected with verbatim Q3.3 hint; unknown query keys rejected (D-03).
- [x] **DVCS-BUS-PRECHECK-01**: `crates/reposix-remote/src/bus_handler.rs::handle_bus_export` runs `git ls-remote -- <mirror> refs/heads/main` (with `-`-prefix reject for T-82-01) before reading stdin; on drift emits verbatim `error refs/heads/main fetch first` + Q3.5 hint; NO writes; NO stdin read.
- [x] **DVCS-BUS-PRECHECK-02**: `bus_handler::handle_bus_export` calls `precheck_sot_drift_any(cache, backend, project, rt)` (new 10-line wrapper around L1's `list_changed_since`) before reading stdin; on `Drifted` outcome emits verbatim `error refs/heads/main fetch first` + `git pull --rebase` hint citing mirror-lag refs (when populated); NO writes; NO stdin read.
- [x] **DVCS-BUS-WRITE-01**: `bus_handler::handle_bus_export` calls `write_loop::apply_writes(cache, backend, backend_name, project, rt, proto, &parsed)` (P83-01 T02 refactor lift); SoT writes; audit rows in both tables; `last_fetched_at` advanced.
- [x] **DVCS-BUS-WRITE-02**: Plain `git push <mirror_remote> main` subprocess after SoT success (NO `--force-with-lease` per D-08). On mirror failure: `helper_push_partial_fail_mirror_lag` audit row (NEW op P83-01 T03); `refs/mirrors/<sot>-head` updated; `synced-at` NOT updated; stderr warn; `ok` returned.
- [x] **DVCS-BUS-WRITE-03**: On mirror success: BOTH refs updated; `mirror_sync_written` audit row; `ok refs/heads/main` returned.
- [x] **DVCS-BUS-WRITE-04**: NO helper-side retry per Q3.6. Verifier `bus-write-no-helper-retry.sh` greps for `--force` (none).
- [x] **DVCS-BUS-WRITE-05**: STEP 0 in `bus_handler::handle_bus_export` bails BEFORE `ensure_cache` when no `git remote add` configured for the mirror URL; verbatim Q3.5 hint emitted; NO cache opened (regression test in `bus_write_no_mirror_remote.rs`).
- [x] **DVCS-BUS-WRITE-06**: Three fault-injection tests under `crates/reposix-remote/tests/`:
  - `bus_write_mirror_fail.rs` (`#[cfg(unix)]` failing-update-hook fixture; case a)
  - `bus_write_sot_fail.rs` (mid-stream 5xx; case b)
  - `bus_write_post_precheck_409.rs` (post-precheck 409; case c)
  Plus `bus_write_audit_completeness.rs` for OP-3 dual-table assertion.
- [x] **DVCS-BUS-FETCH-01**: Capabilities branching at `crates/reposix-remote/src/main.rs:150-172` omits `stateless-connect` for `Route::Bus`; `tests/bus_capabilities.rs` confirms single-backend advertises it AND bus URLs DO NOT.

#### L1 perf migration

- [x] **DVCS-PERF-L1-01**: Unconditional `list_records` walk in `handle_export` replaced with `crates/reposix-remote/src/precheck.rs::precheck_export_against_changed_set` (narrow-deps; bus handler reuses). Cursor-present hot path: one `list_changed_since` REST call + per-id GETs bounded by `changed_set ∩ push_set`. L1-strict delete: cache trusted as prior; backend deletes surface as 404 on PATCH at write time.
- [x] **DVCS-PERF-L1-02**: `reposix sync --reconcile` clap subcommand at `crates/reposix-cli/src/sync.rs` (3-test smoke suite at `crates/reposix-cli/tests/sync.rs`). On-demand full `list_records` walk + cache reconciliation for users who suspect cache desync. Documentation defers to P85 (`docs/concepts/dvcs-topology.md`); CLAUDE.md § Commands carries the inline bullet now.
- [x] **DVCS-PERF-L1-03**: N=200 wiremock perf regression at `crates/reposix-remote/tests/perf_l1.rs::l1_precheck_uses_list_changed_since_not_list_records` (+ positive-control sibling). Single-backend and bus paths share `precheck.rs`. L2/L3 cache-desync hardening defers to v0.14.0.

#### Webhook-driven mirror sync

- [x] **DVCS-WEBHOOK-01**: Reference workflow at `docs/guides/dvcs-mirror-setup-template.yml` (template) + live copy at `reubenjohn/reposix-tokenworld-mirror/.github/workflows/reposix-mirror-sync.yml` (commit 09dda47 in mirror repo). Triggers: `repository_dispatch` (event type `reposix-mirror-sync`) + literal cron `'*/30 * * * *'` (D-06; GH Actions can't template `${{ vars.* }}` in `schedule:`).
- [x] **DVCS-WEBHOOK-02**: Workflow uses `git push --force-with-lease=refs/heads/main:$(git rev-parse mirror/main) origin main`; race-protection regression test at `quality/gates/agent-ux/webhook-force-with-lease-race.sh`.
- [x] **DVCS-WEBHOOK-03**: First-run branches via `git show-ref --verify --quiet refs/remotes/mirror/main` (D-07); both Q4.3 sub-cases (4.3.a fresh-but-readme + 4.3.b truly-empty) covered by `quality/gates/agent-ux/webhook-first-run-empty-mirror.sh`.
- [x] **DVCS-WEBHOOK-04**: Latency artifact `quality/reports/verifications/perf/webhook-latency.json`; p95=5s ≤ 120s threshold (D-02). Synthetic-dispatch n=1 shipped; real-TokenWorld n=10 deferred (awaits v0.13.x release). Owner-runnable `scripts/webhook-latency-measure.sh`.

#### DVCS docs

- [x] **DVCS-DOCS-01**: `docs/concepts/dvcs-topology.md` (164 lines) — three roles (SoT-holder / mirror-only consumer / round-tripper) + Q2.2 verbatim phrase at line 63 + when-to-choose-which-pattern guidance.
- [x] **DVCS-DOCS-02**: `docs/guides/dvcs-mirror-setup.md` (198 lines) — prereqs + mirror-repo creation + secrets config + webhook setup + cron-only fallback + cleanup procedure.
- [x] **DVCS-DOCS-03**: `docs/guides/troubleshooting.md` "DVCS push/pull issues" section (~120 lines) — bus-remote fetch-first + attach reconciliation cases + webhook race + cache-desync via `reposix sync --reconcile`.
- [◐] **DVCS-DOCS-04** (owner-graded): `subjective/dvcs-cold-reader` rubric row in `quality/catalogs/subjective-rubrics.json` with full criteria + reader-profile + 10-pt scale; status NOT_VERIFIED by design — owner runs `/reposix-quality-review --rubric dvcs-cold-reader` per Path B (CLAUDE.md "Cold-reader pass on user-facing surfaces"). Owner's PASS verdict completes DVCS-DOCS-04 closure.

#### Dark-factory regression — third arm

- [x] **DVCS-DARKFACTORY-01**: Extended `quality/gates/agent-ux/dark-factory.sh` with `dvcs-third-arm` scenario covering the agent UX surface (5 static teaching-string greps + 5 `--help` token greps + bus URL composition + cache materialization + `attach_walk` audit row + wire-path delegation cite). Wire-path round-trip delegated to `crates/reposix-remote/tests/bus_write_happy.rs::happy_path_writes_both_refs_and_acks_ok` (rationale per 86-01-SUMMARY.md "Deviations from plan").
- [x] **DVCS-DARKFACTORY-02**: Catalog row `agent-ux/dvcs-third-arm` minted in `quality/catalogs/agent-ux.json` with kind `subagent-graded`, cadence `pre-pr`, freshness_ttl 30d, blast_radius P1. Status PASS post-T02 with `last_verified=2026-05-01T21:43:24Z`; 17 asserts passing.

#### Carry-forward

- [x] **MULTI-SOURCE-WATCH-01**: From v0.12.1 P75. Walker hashes every source in `Source::Multi`, ANDs results; row enters `STALE_DOCS_DRIFT` on ANY drift. Schema migration: `source_hash: Option<String>` → `source_hashes: Vec<String>` parallel-array. `verbs::bind` preserves all entries. Regression tests at `crates/reposix-quality/tests/walk.rs::walk_multi_source_*`.

### +2 reservation (per OP-8)

- [x] **DVCS-SURPRISES-01**: Surprises-absorption phase drains `.planning/milestones/v0.13.0-phases/SURPRISES-INTAKE.md`. Each entry → RESOLVED | DEFERRED | WONTFIX with commit SHA or rationale. Verifier honesty spot-check on previous phases' plans + verdicts (empty intake acceptable IF phases produced explicit `Eager-resolution` decisions).
- [x] **DVCS-GOOD-TO-HAVES-01**: Good-to-haves polish phase drains `GOOD-TO-HAVES.md`. XS items always close; M items default-defer to v0.14.0.

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
| HYGIENE-01 | P78 | shipped |
| HYGIENE-02 | P78 | shipped |
| MULTI-SOURCE-WATCH-01 | P78 | shipped |
| POC-01 | P79 | complete |
| DVCS-ATTACH-01 | P79 | shipped |
| DVCS-ATTACH-02 | P79 | shipped |
| DVCS-ATTACH-03 | P79 | shipped |
| DVCS-ATTACH-04 | P79 | shipped |
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
| DVCS-WEBHOOK-01 | P84 | shipped |
| DVCS-WEBHOOK-02 | P84 | shipped |
| DVCS-WEBHOOK-03 | P84 | shipped |
| DVCS-WEBHOOK-04 | P84 | shipped |
| DVCS-DOCS-01 | P85 | shipped |
| DVCS-DOCS-02 | P85 | shipped |
| DVCS-DOCS-03 | P85 | shipped |
| DVCS-DOCS-04 | P85 | rubric-pending-owner |
| DVCS-DARKFACTORY-01 | P86 | shipped |
| DVCS-DARKFACTORY-02 | P86 | shipped |
| DVCS-SURPRISES-01 | P87 | shipped |
| DVCS-GOOD-TO-HAVES-01 | P88 | shipped |

### Recurring success criteria across every v0.13.0 phase

These are part of every phase's definition-of-done and are NOT separate REQ-IDs (they are recurring expressions of OP-7 + the autonomous-execution protocol):
- **Catalog-first**: phase's first commit writes catalog rows BEFORE implementation.
- **CLAUDE.md update in same PR** (per QG-07 carry-over from v0.12.0).
- **Unbiased verifier-subagent dispatch on phase close** (per OP-7).
- **Per-phase push** — `git push origin main` BEFORE verifier-subagent dispatch; pre-push gate-passing is part of close criterion (codified 2026-04-30, closes 999.4).
- **Eager-resolution preference** per OP-8 — items < 1hr / no new dependency get fixed in the discovering phase; else appended to `SURPRISES-INTAKE.md` or `GOOD-TO-HAVES.md`.
- **Goal: pristine codebase across all dimensions** — every dimension's gates GREEN-or-WAIVED at milestone close.
