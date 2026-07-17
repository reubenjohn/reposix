# Roadmap: reposix

> **Active milestone: v0.15.0 "Floor"** (roadmap scoped 2026-07-15 via `/gsd-new-milestone`
> → gsd-roadmapper) — Phases **P114–P128** (15 phases), continuing numbering from v0.14.0's
> highest shipped phase (P113). Full detail: § "v0.15.0 Floor (PLANNING)" below +
> `.planning/REQUIREMENTS.md`. The stale v0.13.0 (SHIPPED 2026-07-07) and v0.13.2 (QUEUED,
> not shipped) planning H2 blocks that used to sit further down in this file were retired
> in the 2026-07-14 cleanup pass — git history is the archive; see § "Milestones" and
> § "Previously planned milestones" below, plus `.planning/MILESTONES.md`, for ground truth.

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
- ✅ **v0.12.0 Quality Gates** — Phases 56–65 (shipped 2026-04-29) · [archive](milestones/v0.12.0-phases/ROADMAP.md)
- ✅ **v0.12.1 Polish** — Phases 72–77 (shipped 2026-04-30) · [archive](milestones/v0.12.1-phases/ARCHIVE.md)
- ✅ **v0.13.0 DVCS over REST (extended)** — Phases 78–97 (shipped 2026-07-07) · [milestone roadmap](milestones/v0.13.0-phases/ROADMAP.md)
- ✅ **v0.13.1 Front door hotfix** — Phases 98–101 (shipped 2026-07-08)
- ✅ **v0.14.0 Wave-2 hardening** — Phases 102–113 (shipped + Latest 2026-07-14) · [milestone roadmap](milestones/v0.14.0-phases/ROADMAP.md)
- 📋 **v0.13.2 Cross-link fidelity** — QUEUED, not shipped; placeholder numbering P98–P107 (collides with the now-shipped v0.13.1/v0.14.0 ranges above), pending renumber-on-insertion behind the Arc D launch-readiness arc · [milestone roadmap](milestones/v0.13.2-phases/ROADMAP.md)
- 🚧 **v0.15.0 Floor** — Phases 114–128 (roadmap scoped 2026-07-15, ACTIVE — first planned milestone of ratified Arc D) · full detail in this file, § "v0.15.0 Floor (PLANNING)"

## Phases

## v0.15.0 Floor (PLANNING)

> **Status:** roadmap scoped 2026-07-15 via `/gsd-new-milestone` (gsd-roadmapper), Arc D
> RATIFIED at `6aa734a` (ADDENDUM: `.planning/milestones/audits/2026-07-12-reality-check.md`).
> Phases continue numbering from v0.14.0's highest shipped phase (P113) — this milestone
> runs **P114–P128** (15 phases). Full per-requirement detail:
> `.planning/REQUIREMENTS.md` § v0.15.0 Requirements (41 REQ-IDs — FIX/DOCS/UX/BENCH/ADR/
> DRAIN). Context anchors: `.planning/PROJECT.md` § Current Milestone: v0.15.0 Floor;
> `.planning/milestones/v0.15.0-phases/{SURPRISES-INTAKE,GOOD-TO-HAVES}.md` (row-level
> detail the DRAIN-* requirements route).
>
> **Ordering rationale.** FIX-01 (t4 Confluence oid-drift product defect) runs FIRST as
> the fix-first correctness lane; FIX-02 (its `sync --reconcile` audit) rides the same
> phase since it depends on FIX-01 landing. BENCH-01 runs EARLY — hard waiver-expiry
> deadline **2026-08-15**; spend ceiling **≤50** benchmark sessions on the existing
> subscription (owner-confirmed), escalate to the owner only past 50. The ADR-010
> decision packet (FIX-03 + ADR-01, co-located because both touch
> `docs/decisions/010-l2-l3-cache-coherence.md`) produces options+tradeoffs for an
> owner/manager ruling — it does **NOT** implement a chosen option pre-ruling; FIX-03's
> v0.15 implementation depth is subject to that ruling. Doc-truth launch-blocker purge,
> the post-bench honesty corrections, and docs/planning simplification (the "P112 RAISE"
> — delete stale content outright, git history is the archive, no keep-with-banners)
> follow, in that order (the honesty corrections need BENCH-01's re-measured figure).
> UX error-hardening (Rust-compiler-grade CLI + helper errors, then the `RPX-xxxx` code
> namespace) comes next. DRAIN-01..25 (the already-routed SURPRISES-INTAKE +
> GOOD-TO-HAVES intake) are grouped into five thematic phases rather than 25
> micro-phases. The milestone's **last two phases (P127, P128) are the standard OP-8 "+2
> reservation" absorption slots** — they drain whatever NEW surprises/good-to-haves
> surface *during* this milestone's own execution (distinct from the DRAIN-01..25
> requirements, which are already-known intake being routed now, not new).

### Phase index (P114–P128)

- [x] **Phase 114: t4 Confluence oid-drift fix-first + reconcile audit** - Confluence checkouts stop aborting on oid-drift; the `sync --reconcile` recovery claim is proven or corrected. (completed 2026-07-15)
- [x] **Phase 115: Live MCP benchmark re-measurement** - Fresh token-economy + latency figures replace the 8 waived hero-number rows before the 2026-08-15 deadline.
- [x] **Phase 116: ADR-010 mirror-fanout decision packet + slug→id durable-create design** - Owner/manager gets a ruling-ready options+tradeoffs packet; nothing is implemented pre-ruling. (completed 2026-07-16)
- [x] **Phase 117: Doc-truth launch-blocker purge** - The 6 identified doc-truth defects no longer mislead a first-time reader or agent. (completed 2026-07-17)
- [x] **Phase 118: Post-bench honesty corrections** - The disputed token-count figure and the stale tag-cut premise are corrected. (completed 2026-07-17)
- [ ] **Phase 119: Docs/planning simplification (the "P112 RAISE")** - Stale legacy planning/doc content is deleted outright; git history is the archive.
- [ ] **Phase 120: CLI + helper error hardening to Rust-compiler-grade** - Every user-facing error teaches the fix, suggests the alternative, gives copy-paste recovery.
- [ ] **Phase 121: RPX error-code namespace + `reposix explain`** - Every error carries a stable code; `reposix explain <code>` looks it up.
- [ ] **Phase 122: `reposix-remote` + `init` hardening** - Two HIGH-severity carry-forward robustness gaps close (modern-git rebase-recovery verification; binary-side self-safety refusal).
- [ ] **Phase 123: Quality-runner & catalog integrity hardening** - `run.py` and the catalog it persists resist false-greens, silent corruption, and misleading errors.
- [ ] **Phase 124: Container-rehearse harness hardening** - Docs-repro container rows are provenance-guaranteed and immune to SIGKILL orphaning + tautological congruence.
- [ ] **Phase 125: Real-backend cadence & mirror-drift resilience** - The `pre-release-real-backend` cadence and milestone-close litmus survive GitHub-mirror drift.
- [ ] **Phase 126: Docs-alignment tooling polish** - The doc-alignment skill/tooling surface is more reliable and less confusing.
- [ ] **Phase 127: Surprises absorption (OP-8 Slot 1)** - Every surprise surfaced during P114–P126's own execution has a terminal STATUS.
- [ ] **Phase 128: Good-to-haves polish + milestone close (OP-9 Slot 2)** - GOOD-TO-HAVES drained, RETROSPECTIVE.md distilled, milestone-close ritual complete, tag script authored.

### Phase Details

### Phase 114: t4 Confluence oid-drift fix-first + reconcile audit
**Goal**: Confluence page checkouts no longer abort on the list-vs-get oid-drift product defect, and the `sync --reconcile` recovery claim is proven true or corrected to its real scope.
**Depends on**: Nothing (first phase of v0.15.0)
**Requirements**: FIX-01, FIX-02
**Success Criteria** (what must be TRUE):
  1. `git checkout -B main` against live Confluence TokenWorld (including page 7766017) completes with zero oid-drift abort.
  2. The P0 gate `agent-ux/t4-conflict-rebase-ancestry-real-backend` passes GREEN against live Confluence TokenWorld.
  3. The root-cause fix lands in either the Confluence adapter (render-parity between `list_records` and `get_record`) or `build_from`'s oid computation (use the get-representation it actually materializes from) — not a workaround.
  4. `builder.rs`/`cache.rs` doc comments accurately describe which oid-drift class `reposix sync --reconcile` actually recovers, verified against a reproduction rather than assumed.
**Plans**: 2 plans (Wave 1: 114-01 · Wave 2: 114-02)
- [x] 114-01-PLAN.md — FIX-01: Confluence adapter render-parity — add `body-format=atlas_doc_format` to the `list_issues_impl` LIST url so list/get bodies decode identically; + a render-parity contract test
- [x] 114-02-PLAN.md — FIX-02: `sync --reconcile` recovery-claim audit — reproduction-backed `oid_drift_reconcile` mock test + scoped doc corrections in error.rs/sync.rs/main.rs (cache.rs cursor-drift doc confirmed accurate)

### Phase 115: Live MCP benchmark re-measurement
**Goal**: Token-economy + latency benchmark figures backing the 8 hero-number doc-alignment-waived rows are re-measured against live conditions before the waiver-expiry deadline.
**Depends on**: Nothing — schedule EARLY, parallel-safe with Phase 114 (hard deadline **2026-08-15**)
**Requirements**: BENCH-01
**Success Criteria** (what must be TRUE):
  1. Fresh token-economy + latency measurements exist for every one of the 8 hero-number doc-alignment-waived rows, captured via live MCP sessions.
  2. Total benchmark-session spend is tracked and stays **≤50** sessions on the existing subscription; any need to exceed 50 is escalated to the owner before further spend.
  3. Re-measured figures + methodology are recorded in a form Phase 118 (DOCS-07) and DOCS-05 can consume directly.
  4. The waived `perf/token-economy-bench` + `perf/headline-numbers-cross-check` catalog rows have a documented path to un-waive using the new figures.
**Plans**: TBD
**Execution mode**: top-level (fan-out live benchmark sessions → gather results → interpret against the 8 waived rows; not a write-code-test-commit shape)

### Phase 116: ADR-010 mirror-fanout decision packet + slug→id durable-create design
**Goal**: The owner/manager has a decision packet enumerating ADR-010 mirror-fanout options+tradeoffs, plus the slug→id durable-create design for the interrupted-create duplicate hazard, ready to RULE on — with no implementation of a chosen option landing yet.
**Depends on**: Nothing
**Requirements**: FIX-03, ADR-01
**Success Criteria** (what must be TRUE):
  1. A decision packet exists alongside `docs/decisions/010-l2-l3-cache-coherence.md` enumerating mirror-fanout options with tradeoffs.
  2. The slug→id durable-create reconciliation design (mint a stable local slug pre-push; model "slug X → (pending) → backend id N") is documented and co-located with the ADR-010 packet, per FIX-03's ask.
  3. An owner/manager ruling is recorded in `.planning/CONSULT-DECISIONS.md` (or equivalent) before any implementation of a chosen option proceeds.
  4. FIX-03's v0.15 implementation depth is explicitly scoped by that ruling, not pre-decided by the executor.
**Plans**: 3 plans (all Wave 1, parallel — zero file overlap; Plan 01 T1 is the catalog-first first commit)
- [x] 116-01-PLAN.md — ADR-01 live-doc mirror-truth blessing (webhook+cron = authoritative; (a)/(b) mirror-sense split; refresh-script = op-recovery-only) in `dvcs-topology.md` + root `CLAUDE.md`, + a doc-alignment regression-guard row/verifier
- [x] 116-02-PLAN.md — ADR-010 §2 amendment (ADR-01: RBF-LR-04 lever CLOSED, Option D rejected) + §3 amendment (FIX-03: Option B SANCTIONED TARGET DESIGN, design-only, waiver stays qualified) + packet co-location cross-link (criterion 1)
- [x] 116-03-PLAN.md — retire the LIVE litmus-non-idempotency SURPRISES-INTAKE row (terminal RESOLVED) + update GOOD-TO-HAVES-09 (FIX-03 next-milestone build proposal)
**Execution mode**: top-level (produce options + tradeoffs, gather owner/manager ruling; explicitly NOT an implement-and-ship phase)

> **Planner note (2026-07-16, P116 planning).** The Goal + Success Criteria 1–4 + Execution-mode
> text above describe the OLD pre-ruling shape and are now STALE: BOTH rulings already
> happened (`.planning/CONSULT-DECISIONS.md`, 2026-07-16, commit `8212373`; criteria 1–4 are
> materially satisfied — packet exists, ruling recorded, FIX-03 depth scoped design-only). The
> 3 planned plans EXECUTE the rulings' follow-through (a write→verify→commit shape); they still
> run **top-level** (coordinator dispatches leaf subagents), so the Execution-mode marker holds
> even though its parenthetical rationale is stale. Surfaced, not silently rewritten (RESEARCH
> §5.5 / Noticed #4). A future reader diffing ROADMAP-vs-shipped should read the phase's
> `116-0N-PLAN.md` + `116-VALIDATION.md` for the actual delivered scope.

### Phase 117: Doc-truth launch-blocker purge
**Goal**: The 6 identified doc-truth launch-blockers no longer mislead a first-time reader or agent.
**Depends on**: Nothing
**Requirements**: DOCS-01, DOCS-02, DOCS-03, DOCS-04, DOCS-05, DOCS-06
**Success Criteria** (what must be TRUE):
  1. `docs/index.md` correctly describes Confluence as a wiki (not an issue tracker) and the bootstrap verb as `reposix init` (not `git clone`).
  2. `docs/how-it-works/filesystem-layer.md` (or its renamed successor) no longer claims `cat` secretly triggers a network call; every propagated cross-link (`index.md`, `git-layer.md`, `time-travel.md`, `trust-model.md`) is fixed.
  3. `reposix list` and `reposix refresh` connection-refused errors teach the fix, suggest the sim alternative, and give copy-paste recovery, matching `init.rs:365-370`.
  4. `reposix detach` either exists as a real subcommand or `attach.rs`'s multi-SoT-conflict error no longer references a nonexistent one.
  5. `docs/benchmarks/token-economy.md` no longer claims a fabricated demo-script provenance for `reposix_session.txt`, and `docs/social/twitter.md` no longer describes the deleted FUSE architecture.
**Plans**: 7 plans (6 waves; executors serialize — one tree-writer; the sole cargo wave runs alone)
- [ ] 117-01-PLAN.md — Error-UX (cargo): SC3 list/refresh teach-fix bail! + SC4 Option-B attach.rs reword
- [ ] 117-02-PLAN.md — Doc-truth: SC1 index.md hero (Confluence-wiki + reposix init) + SC2 filesystem-layer.md + propagation
- [ ] 117-03-PLAN.md — Doc-truth: SC5 benchmarks/README.md provenance rewrite + twitter.md de-FUSE + new catalog row
- [ ] 117-04-PLAN.md — Furnished-product IA/polish (GTH-V15-36): index.md progressive-disclosure + how-it-works quartet
- [ ] 117-05-PLAN.md — Animation embed baseline (GTH-V15-37): mp4 click-to-play + docs/assets/animation convention
- [ ] 117-06-PLAN.md — Fix-twice: docs/social/** freshness gate (catalog-first) + dead-code + CLAUDE.md sweep
- [ ] 117-07-PLAN.md — COORDINATOR-RUN close: doc-alignment refresh + mp4 release upload + cold-reader review + push

> **Owner mandate (2026-07-16) — REQUIRED planning input.** The docs site should read as
> a FURNISHED PRODUCT, not merely doc-truth-correct — owner verbatim: *"Its good, but we
> can do so much better!"* This phase's plan MUST fold in a cold-reader/IA polish pass, not
> just clear the Success Criteria above. Full text: `GOOD-TO-HAVES.md` GTH-V15-36.
> Also: owner-approved animation-embed lane (80s launch animation on the mkdocs home page)
> — see `GOOD-TO-HAVES.md` GTH-V15-37.

### Phase 118: Post-bench honesty corrections
**Goal**: The disputed token-count figure and the stale tag-cut premise are corrected with accurate, current information.
**Depends on**: Phase 115 (BENCH-01's re-measured figure feeds DOCS-07)
**Requirements**: DOCS-07, DOCS-09
**Success Criteria** (what must be TRUE):
  1. `.planning/PROJECT.md`'s Context section no longer cites the disputed "~150k→~2k (98.7%)" figure or the FUSE-era `/mnt/jira/PROJ-123.md` path; it cites the CI-verified 89.1% figure or BENCH-01's re-measured value with a git-native example path.
  2. The Arc D ADDENDUM's stale "cut two stalled tags" premise is corrected to state that v0.13.0/v0.13.1/v0.14.0 are all already tagged and released as GitHub releases.
  3. Neither correction re-opens Arc D scope or performs a literal tag cut — both are prose corrections only, per DOCS-09's explicit scope note.
**Plans**: 1 plan (Wave 1: 118-01) — CLOSED GREEN 2026-07-17 (verdict `quality/reports/verdicts/p118/VERDICT.md`, 3/3 SCs)
- [x] 118-01-PLAN.md — DOCS-07: re-cite `.planning/PROJECT.md`'s token figure to P115's live-benchmark measurement (~94% fewer output tokens) + drop the FUSE-era `/mnt/jira/PROJ-123.md` path for a git-native `cat issues/<id>.md` example; DOCS-09: annotate the 2026-07-12 reality-check audit's stale "cut two stalled tags" premise as dated-SUPERSEDED (prose-only, no Arc D reopen, no literal tag cut)

### Phase 119: Docs/planning simplification (the "P112 RAISE")
**Goal**: Stale/superseded legacy planning and doc content is deleted outright — git history is the archive, not kept-with-banners.
**Depends on**: Nothing (sequenced after the doc-truth purge for narrative cleanliness, not a hard dependency)
**Requirements**: DOCS-08, DRAIN-11, DRAIN-25
**Success Criteria** (what must be TRUE):
  1. The loose phase-dir archival cascade (now unblocked by v0.13.0/v0.14.0 tags landing) is inventoried and resolved — stale artifacts deleted, not banner-wrapped.
  2. Stale top-level catalog JSONs (`v0.11.1-catalog.json`, `docs_reproducible_catalog.json`) and loose MANAGER/SESSION-HANDOVER transients are removed.
  3. `.planning/ORCHESTRATION.md` is back under its 20000B `structure/file-size-limits` ceiling (split to a sibling doc) before the 2026-08-08 waiver lapses.
  4. The six broken `**Plan:**` markdown links in `.planning/milestones/v0.13.0-phases/ROADMAP.md` (P79–P84) point at the real `NN-PLAN-OVERVIEW/` directory form.
  5. The SURPRISES-INTAKE/GOOD-TO-HAVES progressive-disclosure bloat files are explicitly left untouched (deferred to the v0.17 meta-milestone per Arc D — different mechanic, split vs. delete, would compete with this phase's work).
**Plans**: TBD
**Execution mode**: top-level (full inventory + audit of what's stale across `.planning/` before any deletion — fan-out/gather/interpret, not a pre-scoped write-code-test-commit plan)

> **Owner mandate (2026-07-16) — REQUIRED planning input.** Beyond deletion of stale
> content, the docs site should read as a FURNISHED PRODUCT — owner verbatim: *"Its good,
> but we can do so much better!"* Fold a cold-reader/progressive-disclosure polish pass
> into this phase's plan. Full text: `GOOD-TO-HAVES.md` GTH-V15-36.

### Phase 120: CLI + helper error hardening to Rust-compiler-grade
**Goal**: Every user-facing `reposix` CLI subcommand error and every `reposix-remote` git helper error teaches the fix, suggests the alternative, and gives copy-paste recovery.
**Depends on**: Nothing
**Requirements**: UX-01
**Success Criteria** (what must be TRUE):
  1. Every `reposix-cli` subcommand error (init/attach/list/sync/doctor/spaces/refresh/etc.) meets the 3-part `init.rs::refuse_existing_repo_root` standard.
  2. Every `reposix-remote` git helper error (`main.rs`, `stateless_connect.rs`) meets the same 3-part standard.
  3. The `agent-ux` catalog contract rows for this surface are written in the phase's first commit, before any implementation commit (catalog-first).
**Plans**: TBD

### Phase 121: RPX error-code namespace + `reposix explain`
**Goal**: Every user-facing error carries a stable, documented code; `reposix explain <code>` looks it up, mirroring `rustc --explain E0308`.
**Depends on**: Phase 120 (shares the same error-message inventory built during the UX-01 audit)
**Requirements**: UX-02
**Success Criteria** (what must be TRUE):
  1. Every user-facing error across the CLI + helper surface emits a structured `RPX-xxxx` code.
  2. `reposix explain <code>` exists and, for every code emitted anywhere in the CLI or helper, prints a non-empty cause + fix + copy-paste recovery.
  3. The output pattern is verified side-by-side against `rustc --explain E0308` for parity of shape.
**Plans**: TBD

### Phase 122: `reposix-remote` + `init` hardening (HIGH carry-forwards)
**Goal**: The git helper and `reposix init` close two HIGH-severity carry-forward robustness gaps from the v0.14.0 intake.
**Depends on**: Nothing
**Requirements**: DRAIN-07, DRAIN-08, DRAIN-09
**Success Criteria** (what must be TRUE):
  1. `quality/gates/agent-ux/rebase-recovery-reconciles.sh` exits 0 on a modern-git (≥2.34) CI runner, or the `stateless-connect` divergence from the 2.25.1 `import`-path behavior is documented + gated.
  2. `resolve_import_parent()` (`crates/reposix-remote/src/main.rs:400-419`) errors loudly — not silently degrading to the parentless path — on a non-ref-absence git failure, proven by a regression test.
  3. A subprocess/worktree `reposix init` targeting the shared source tree is refused with a Rust-compiler-grade error, while the sanctioned `/tmp` dark-factory flow and legitimate `attach` adoption still succeed.
**Plans**: TBD

### Phase 123: Quality-runner & catalog integrity hardening
**Goal**: `quality/runners/run.py` and the catalog it persists resist false-greens, silent corruption, and misleading errors.
**Depends on**: Nothing
**Requirements**: DRAIN-01, DRAIN-03, DRAIN-04, DRAIN-05, DRAIN-06, DRAIN-10
**Success Criteria** (what must be TRUE):
  1. `run.py` self-sources `.env` (or every documented `pre-release-real-backend` invocation bakes in `set -a; . ./.env; set +a`), closing the false-green-preflight / silent-skip gap.
  2. `--persist` refuses to downgrade a committed-GREEN catalog row without an explicit `--allow-downgrade` opt-in.
  3. Concurrent `--persist` runners cannot race-corrupt the shared catalog JSON (advisory `flock` or a single locked persist lane).
  4. A catalog row with a missing or non-executable `verifier.script` is caught by a dedicated `quality/gates/structure/verifier-script-exists.sh` gate.
  5. `code/ci-green-on-main`'s phase-close probe watches a required-workflow list (not a hardcoded `ci.yml` only), and the t4 gate surfaces the real oid-drift stderr instead of a misleading git-version fallback.
**Plans**: TBD

### Phase 124: Container-rehearse harness hardening
**Goal**: `container-rehearse.sh`'s docs-repro rows are provenance-guaranteed and immune to SIGKILL orphaning, exit-code disagreement, and tautological assertion-congruence.
**Depends on**: Nothing
**Requirements**: DRAIN-13, DRAIN-14, DRAIN-22, DRAIN-23, DRAIN-24
**Success Criteria** (what must be TRUE):
  1. Container-row congruence is EARNED via per-step `ASSERT-PASS:` harvesting, not emitted verbatim from `expected.asserts`; example-05 exercises a real runtime blob-limit-exceeded error + `git sparse-checkout` recovery cycle.
  2. Sim teardown survives an outer SIGKILL (an internal `timeout` shorter than the row's `timeout_s`, and/or a killed process group); a stale orphaned sim on port 7878 is detected fail-loud, not silently reused.
  3. `target/debug/reposix`'s provenance on the `quality-post-release.yml` runner is confirmed via an explicit build/artifact-download step, or documented inline if one already exists.
  4. The harness exit code is derived strictly from the persisted `exit_code` (not a possibly-disagreeing rc=0), and a `.sim-*.log` `.gitignore` pattern is added under `quality/reports/verifications/docs-repro/`.
**Plans**: TBD

### Phase 125: Real-backend cadence & mirror-drift resilience
**Goal**: The `pre-release-real-backend` cadence and the milestone-close vision-litmus survive GitHub-mirror drift instead of false-negatives.
**Depends on**: Nothing
**Requirements**: DRAIN-02, DRAIN-12
**Success Criteria** (what must be TRUE):
  1. A documented mandatory mirror-refresh pre-step (`scripts/refresh-tokenworld-mirror.sh`) — or a self-reconciling litmus — prevents a second-run false-negative caused by the litmus's own prior push re-staling the GitHub mirror.
  2. The milestone-close vision-litmus fixture self-heals for BOTH backend drift (trashed protected pages) and GitHub mirror drift, reconciling the mirror to backend-current through the reposix bus remote before the marker push.
  3. The helper's `git pull --rebase` teaching string is corrected for the mirror-drift case specifically (not just the backend-drift case).
**Plans**: TBD

### Phase 126: Docs-alignment tooling polish
**Goal**: The doc-alignment skill/tooling surface (grader, walk, status, plan-refresh, README) is more reliable and less confusing.
**Depends on**: Nothing
**Requirements**: DRAIN-15, DRAIN-16, DRAIN-17, DRAIN-18, DRAIN-19, DRAIN-20, DRAIN-21
**Success Criteria** (what must be TRUE):
  1. The `doc-clarity-review` skill hard-fails on a canary probe (or carries the subscription-caveat note) instead of returning a confusing non-error when it can't see file content.
  2. `README.md` expands "MCP" to "Model Context Protocol (MCP)" on first use; the pre-push docs-alignment block message names the specific blocking row-STATE(s), not just the ratio.
  3. The doc-alignment grader (`.claude/skills/reposix-quality-doc-alignment/prompts/grader.md`) only binds a row if the test fails when the cited number drifts, and greps `src/` unit tests broadly rather than just the currently-cited test.
  4. `plan-refresh <doc>` warns when invoked cold (before a `walk`); `status` surfaces a `waived_active` counter for MISSING_TEST rows.
  5. The 16 pre-existing "cites out-of-eligible-file" coverage warnings are resolved — allowlist expanded, or the rows re-cite eligible files.
**Plans**: TBD

### Phase 127: Surprises absorption (OP-8 Slot 1)
**Goal**: Every surprise that surfaced DURING Phases 114–126's own execution has a resolved, terminal home — per OP-8, this is the milestone's first reservation slot, not a continuation of the already-routed DRAIN-01..25 work.
**Depends on**: Phases 114–126 ALL GREEN
**Requirements**: None of the 41 v0.15.0 REQ-IDs (all are already mapped to Phases 114–126) — reserved OP-8 absorption slot for newly-surfaced intake only.
**Success Criteria** (what must be TRUE):
  1. Every entry added to `.planning/milestones/v0.15.0-phases/SURPRISES-INTAKE.md` during Phases 114–126's execution has a terminal STATUS (RESOLVED + commit SHA / DEFERRED + target milestone / WONTFIX + rationale) — no `STATUS: TBD` at phase close.
  2. Verifier honesty spot-check samples ≥3 Phase 114–126 plan/verdict pairs.
  3. Phase close: `git push origin main`; verifier subagent GREEN.
**Plans**: TBD

### Phase 128: Good-to-haves polish + milestone close (OP-9 Slot 2)
**Goal**: GOOD-TO-HAVES entries newly surfaced during v0.15.0's own execution are drained, the milestone's RETROSPECTIVE.md section is distilled BEFORE archive, and the milestone-close ritual (CHANGELOG, tag script, non-skippable 9th probe) completes GREEN.
**Depends on**: Phase 127
**Requirements**: None of the 41 v0.15.0 REQ-IDs — reserved OP-8/OP-9 milestone-close slot.
**Success Criteria** (what must be TRUE):
  1. Every entry added to `.planning/milestones/v0.15.0-phases/GOOD-TO-HAVES.md` during v0.15.0's own execution has a terminal STATUS.
  2. SURPRISES + GOOD-TO-HAVES + per-phase verdicts are distilled into a new `.planning/RETROSPECTIVE.md` v0.15.0 section BEFORE archive (OP-9; verifier grades RED if missing).
  3. `CHANGELOG.md` `[v0.15.0]` block is finalized; the milestone-close verifier subagent is dispatched and GREEN, including the non-skippable 9th probe (`python3 quality/runners/run.py --cadence pre-release-real-backend`, catalog row `agent-ux/milestone-close-vision-litmus-real-backend`, never waived).
  4. `.planning/milestones/v0.15.0-phases/tag-v0.15.0.sh` is authored (owner gate; owner runs it).
**Plans**: TBD

## Progress

**Execution order:** Phases execute in numeric order: 114 → 115 → … → 128.

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 114. t4 oid-drift fix-first + reconcile audit | 2/2 | Complete   | 2026-07-15 |
| 115. Live MCP benchmark re-measurement | 0/TBD | Not started | - |
| 116. ADR-010 decision packet + slug→id design | 0/3 | Planned | - |
| 117. Doc-truth launch-blocker purge | 0/7 | Planned | - |
| 118. Post-bench honesty corrections | 1/1 | Complete | 2026-07-17 |
| 119. Docs/planning simplification | 0/TBD | Not started | - |
| 120. CLI + helper error hardening | 0/TBD | Not started | - |
| 121. RPX error-code namespace + explain | 0/TBD | Not started | - |
| 122. reposix-remote + init hardening | 0/TBD | Not started | - |
| 123. Quality-runner & catalog integrity | 0/TBD | Not started | - |
| 124. Container-rehearse harness hardening | 0/TBD | Not started | - |
| 125. Real-backend cadence & mirror-drift resilience | 0/TBD | Not started | - |
| 126. Docs-alignment tooling polish | 0/TBD | Not started | - |
| 127. Surprises absorption (OP-8 Slot 1) | 0/TBD | Not started | - |
| 128. Good-to-haves polish + milestone close (OP-9 Slot 2) | 0/TBD | Not started | - |

## Previously planned milestones

Per CLAUDE.md §0.5 / Workspace layout, each shipped/historical milestone's
ROADMAP.md lives inside its `*-phases/` directory. Top-level ROADMAP.md
holds ONLY the active milestone (currently v0.15.0) + this index.

- **v0.14.0** Wave-2 hardening — `.planning/milestones/v0.14.0-phases/ROADMAP.md` (Phases 102–113, SHIPPED + Latest 2026-07-14).
- **v0.13.1** Front door hotfix — `.planning/milestones/v0.13.1-phases/ROADMAP.md` (Phases 98–101, SHIPPED 2026-07-08).
- **v0.13.0** DVCS over REST — `.planning/milestones/v0.13.0-phases/ROADMAP.md` (Phases 78–97, SHIPPED 2026-07-07).
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
