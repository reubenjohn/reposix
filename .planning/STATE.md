---
gsd_state_version: 1.0
milestone: v0.15.0
milestone_name: Floor
status: executing
last_updated: "2026-07-18T05:19:04.000Z"
last_activity: 2026-07-17 -- P121 CLOSED GREEN (gsd-verifier verdict GREEN at quality/reports/verdicts/p121/VERDICT.md, commit 80a37cea): RPX error-code namespace + `reposix explain` — all 6/6 SC + OP-2 verified against reality. 33-code RPX registry (reposix_core::codes) with rustc --explain-grade extended explanations; `reposix explain <code>` shipped; teach_coded emission wired across CLI + helper; agent-ux/rpx_registry_check gate (5 legs incl. reverse-completeness) GREEN, EMISSION_EXEMPT now empty. 5 LOW GTHs filed (GTH-V15-73..77: RPX-0402 url redaction defense-in-depth, ADR-009 clap flag-surface lock, doc-alignment line-span narrowing, exit-codes.md clap code-2 layer, pre-push 92s kcov drift). Impl @ 9b80538c, verdict @ 80a37cea; next = P122
progress:
  total_phases: 15
  completed_phases: 8
  total_plans: 3
  completed_plans: 2
  percent: 53
---

# Project State

## Current Position

Phase: **P121 (RPX error-code namespace + `reposix explain`, v0.15.0 UX-02) — CLOSED GREEN 2026-07-17.**
gsd-verifier verdict GREEN (`quality/reports/verdicts/p121/VERDICT.md`, commit `80a37cea`); all 6/6
success criteria + OP-2 verified against reality. Shipped a 33-code RPX registry
(`reposix_core::codes`) with `rustc --explain`-grade extended explanations, the `reposix explain
<code>` subcommand, and `teach_coded` emission wired across the CLI + helper; the
`agent-ux/rpx_registry_check` gate (5 legs incl. reverse-completeness) grades GREEN and
`EMISSION_EXEMPT` is now empty. Impl landed `9b80538c`, verdict `80a37cea`; 5 LOW GTHs filed
(`GTH-V15-73..77`). **8/15 v0.15.0 "Floor" phases complete** (P114–P121); next = **P122**.

Phase: **P120 (CLI/helper error-message hardening + credential-hygiene fixes, v0.15.0 UX-01) — CLOSED GREEN 2026-07-17.**
gsd-verifier verdict GREEN (`quality/reports/verdicts/p120/VERDICT.md`); SC1–SC3 all PASS. The three
credential-leak fixes each carry a genuine non-leakage assertion (verifier RAN them green): WR-01
`bus_handler` (token scrubbed from BOTH the operator diag AND the persisted `cache.db` audit row — the
OP-3 leg), WR-02 `sync.rs`, WR-03 `worktree_helpers.rs`. Catalog-first ordering held (W0 catalog commit
`142e1278` strictly precedes first impl `15e971bc`). One MEDIUM out-of-scope noticing filed to
`.planning/milestones/v0.15.0-phases/SURPRISES-INTAKE.md`: `doctor.rs:440/444/454/456` echoes raw
`remote.*.url` (mirror creds) into `DoctorFinding`s — same leak class as WR-02/WR-03 but outside
P120's WR charter, so a valid follow-up not an SC gap. GTH-V15-67 (cred-hygiene.sh inline test-fixture
allow-marker) stays deliberately filed-not-implemented pending owner review. Impl landed `859ba0e3`
(CI `29601596722` = success). **7/15 v0.15.0 "Floor" phases complete** (P114–P120); next = P121.

Phase: **P119 (docs/planning simplification, the "P112 RAISE") — CLOSED GREEN 2026-07-17.**
gsd-verifier verdict GREEN (`quality/reports/verdicts/p119/VERDICT.md`, committed `835d65dd`);
CI green on main tip `c5e1195b`. Shipped an **intention-preserving DP-4 pivot**: the 5-day-stale
literal SC-1/SC-2 deletion plan was NOT executed blind — the deletion targets were found still-live
at execution time and DEFERRED with evidence to
`.planning/milestones/v0.15.0-phases/SURPRISES-INTAKE.md` (RAISE-2a `catalog.py` /
`v0.11.1-catalog.json` = a live KEEP-AS-CANONICAL orphan-scripts quality row; RAISE-2b
`docs_reproducible_catalog.json` = 8 live provenance breadcrumbs into release/docs-repro catalogs).
SC-3 (ORCHESTRATION.md §3 handover-template split into a sibling REFERENCE doc, both halves
< 20000B) + SC-4 (six broken P79–P84 `**Plan:**` links repointed to the real `index.md` targets) +
the clean edits (OP-8 intake reword, ROADMAP progress-table reconcile, PROJECT self-flag de-stale,
P117 parenthetical honesty reword) all LANDED; SC-5 held-by-design (bloat files intentionally left
for the v0.17 meta-milestone). **6/15 v0.15.0 "Floor" phases complete** (P114–P119); next =
P120 (CLI + helper error hardening to Rust-compiler-grade).

Phase: **P118 (post-bench honesty corrections) — CLOSED GREEN 2026-07-17.**
gsd-verifier verdict GREEN on all 3 SCs (`quality/reports/verdicts/p118/VERDICT.md`);
CI green on main tip `a2b0186` (run `29575437132`, `success`). DOCS-07 (SC1) re-cited
`.planning/PROJECT.md`'s "Why this exists" figure to P115's live token-economy
measurement (~94% fewer output tokens) and dropped the FUSE-era `/mnt/jira/PROJ-123.md`
path for a git-native `cat issues/<id>.md` example; DOCS-09 (SC2) annotated the Arc D
reality-check audit's stale "cut two stalled tags" premise as dated-SUPERSEDED
(v0.13.0/v0.13.1/v0.14.0 are all already tagged AND released, v0.14.0 = Latest). Both are
prose-only — Arc D scope is NOT re-opened and no literal tag was cut (SC3). Shipped
`e510d4f`·`59f932a`·`246a2d1`·`a2b0186`. **Next = P119** (docs/planning simplification, the
"P112 RAISE"; `Execution mode: top-level`).

Phase: **P117 (doc-truth launch-blocker purge) — CLOSED GREEN.** Launch-animation
publish (`gh release create docs-assets` + mp4 upload + live `animation-renders`
playwright verify — the second, owner-gated half of 117-07) is manager-deferred
2026-07-17 under standing doctrine (outward publishing = owner-only) — NOT a failure;
`animation-renders` staying NOT-VERIFIED is a pending gate, not an owner-accepted
deferral. OWNER APPROVAL PENDING — the E1 ask remains open, surfaced to the owner; see
`.planning/CONSULT-DECISIONS.md` 2026-07-17 entry.
P116 (ADR-010 mirror-fanout decision packet + slug→id durable-create) CLOSED GREEN
2026-07-16 — gsd-verifier verdict 12/12 must-haves PASS, 0 gaps, 0 blockers
(`116-VERIFICATION.md`); CI green on tip `6825d13` (run `29544462493`, `success`).
P117 is `Execution mode: top-level` (coordinator-run 117-07 close wave; per
`.planning/CLAUDE.md` orchestration-shaped-phase pattern). **Waves 1-4 SHIPPED GREEN**:
W1 (`117-01`, SC3/SC4) banked; W2 (`117-02`∥`117-03`, docs-truth) shipped `c028d4c`,
CI `29559891747`; W3 (`117-04`∥`117-06`, IA propagation + CLAUDE.md sweep) shipped
`0a5f620`, CI `29563263605`; W4 (`117-05`, launch-animation embed) shipped `644763a`,
CI `29565137211` — all post-push `code/ci-green-on-main` P0 probes PASS.
**W5 (`117-07`, coordinator phase close) is IN PROGRESS.** Its NON-BLOCKED
close checklist (catalog audit-bump fold, stray-artifact disposition, PROGRESS/STATE
refresh, non-cargo gate sweep) is done. **Remaining, ALL owner/coordinator-gated —
phase 117 is NOT fully COMPLETE:** Task 1 — owner-approved `gh release upload` of the
launch-animation mp4 (HELD-E1, external mutation, RAISED to top level, do not
self-authorize); Task 2 — post-upload live playwright artifact generation +
`docs-build/animation-renders.sh` authoring (both intentionally deferred until the
asset is live, per the row's own `owner_hint`); Task 3 — cold-reader
`/doc-clarity-review` + `/reposix-quality-review --rubric` + badge resolution
(top-level slash commands); Task 4 — final pre-push, `git push origin main`, post-push
CI-green confirmation, unbiased gsd-verifier dispatch. See
`.planning/phases/117-doc-truth-launch-blocker-purge/PROGRESS.md` `## NOW`/`## NEXT`
for the live briefing.
**6/15 v0.15.0 "Floor" phases complete** (P114, P115, P116, P117, P118, P119); 9 remain; next =
P120.

> **Milestone plan (v0.15.0 Floor — scoped 2026-07-15 gsd-roadmapper; full detail
> `.planning/ROADMAP.md`).** 15 phases P114–P128 (continuing from v0.14.0's highest shipped
> P113), 41 REQ-IDs (FIX/DOCS/UX/BENCH/ADR/DRAIN), 100% coverage, no orphans/duplicates.
> Order: FIX-01 (t4 oid-drift) first → BENCH-01 early (2026-08-15 hard waiver deadline,
> ≤50-session spend ceiling) → ADR-010 decision packet (FIX-03+ADR-01, options-only, no
> pre-ruling implementation) → doc-truth purge → post-bench honesty corrections →
> docs/planning simplification (P112 RAISE) → UX Rust-compiler-grade hardening → 5
> DRAIN-grouping phases → P127/P128 OP-8 "+2 reservation" absorption slots. Phases 115
> (BENCH-01), 116 (ADR-010 packet), 119 (docs/planning simplification) marked `Execution
> mode: top-level`.

### Workstream C — v0.14.0 wave-2 hardening — 11/11 phases GREEN — SHIPPED + Latest

> **SHIPPED + Latest (2026-07-14).** v0.14.0 is tagged, released, and marked "Latest" on
> GitHub (crates.io 0.14.0); the b773c04 RED-main incident is CLOSED (fix @ `8e2aae5`).
> Nothing is tag-blocked. The `make_latest` back-tag hazard for FUTURE releases is handled
> in post-tag queue item 1 (`release.yml` `--latest` hardening,
> `.planning/quick/260713-mlh-make-latest-hardening/`). The B1–B5 tag-remediation cursor
> below is superseded by the ship + b773c04 closure — historical record only.

Phase: **P112** (OD-4 launch-readiness SCOPE-BUT-DO-NOT-START stub) — LANDED. **11/11
phases complete** as of 2026-07-12. **P102** (D2 self-safe dark-factory hardening

+ emergent Phase-0 re-seal), **P103, P104, P105** (RBF-LR-03 rebase recovery), **P106**

(waived tutorials/examples — 5 `docs-repro` rows PASS), **P107** (RUSTSEC memmap2/quinn-proto
cleared), **P108** (prune-completeness gate), **P109** (RBF-FW-11 grandfather rule), **P110**
(OP-8 Slot 1 surprises drain — 17 terminal entries), **P111** (OP-8 Slot 2 good-to-haves +
OP-9 milestone-close, graded GREEN at commit `c259718`: CHANGELOG `[v0.14.0]` + `RETROSPECTIVE.md`
v0.14.0 OP-9 distillation + intake prune; **GTH-09** ADR-010 slug→id DEFERRED-TO-v0.15.0 by an
owner scope call), and the out-of-band **P113** (lost-update shared-cursor guard) ALL shipped
GREEN. **P112** now landed: a scope stub at
`.planning/milestones/v0.14.0-phases/112-od-4-launch-readiness-scope-stub/PLAN.md` naming the four
OD-4 launch-readiness pillars (asciinema hero demo, CI-verified honest headline numbers,
install-path excellence, Show-HN positioning kit), one line each, marked **DO-NOT-START** and
deferred to a post-tag `/gsd-new-milestone` session — zero implementation, no verifier dispatch
(lightweight owner ack suffices per ROADMAP P112).
The aggregate `v0.14.0` tag was subsequently CUT — **v0.14.0 SHIPPED + Latest 2026-07-14**
(crates.io 0.14.0, GitHub release "Latest"); the b773c04 RED-main incident is CLOSED (fix
@ `8e2aae5`). No tag pending.

> **Superseded (2026-07-14) — historical record.** The B1–B5 tag-remediation cursor that
> formerly sat here (B1 mirror-refresh + B2 p93 CREATE-recovery "awaiting owner decision";
> B3/B4/B5 status; 2 owed orphan `p93 smoke A` TokenWorld page sweeps) is moot for the tag
> now that v0.14.0 shipped. Any residual product gaps + the owed teardown are tracked in the
> v0.15.0 intake (`.planning/milestones/v0.15.0-phases/SURPRISES-INTAKE.md` /
> `GOOD-TO-HAVES.md`); prior evidence + diagnosis pointers live in
> `.planning/SESSION-HANDOVER.md` + git history.

## Current Focus

**Active milestone: v0.15.0 Floor — ROADMAP scoped 2026-07-15 (15 phases, P114–P128).**
Arc D ratified at `6aa734a`; this is the first PLANNED milestone of the ratchet-first arc.
Next step: `/gsd-plan-phase 122`. Carried
owner-gate still OPEN (NOT a P119/P120 blocker): the
P117 launch-animation publish (`gh release upload` mp4 + live `animation-renders` verify,
GTH-V15-37 / E1 ask) remains manager-deferred with owner approval PENDING — tracked in
`.planning/CONSULT-DECISIONS.md`.

**Serial workstream history (OD-3 — A then C then B, per OD-4 resequencing):**

- **Workstream A — v0.13.0 extended.** **CLOSED GREEN 2026-07-05 (P78–P97, 20/20 phases).** Shipped P78–P88 2026-05-01; extended 2026-05-08 with P89–P97 (real-backend frictions); milestone-close verdict at `quality/reports/verdicts/milestone-v0.13.0/VERDICT.md`. Tag v0.13.0 landed; v0.13.1 onboarding hotfix (P98–P101) additionally SHIPPED 2026-07-07 (tag `04640d5`). ROADMAP at `.planning/milestones/v0.13.0-phases/ROADMAP.md`.
- **Workstream C — v0.14.0 wave-2 hardening.** **SHIPPED + Latest 2026-07-14** — **11/11 phases GREEN** (P102–P112 + out-of-band P113; see § Workstream C above). v0.14.0 tagged/released (crates.io 0.14.0, GitHub release "Latest"); b773c04 RED-main incident CLOSED (@ `8e2aae5`). P112 OD-4 launch-readiness scope stub LANDED and now superseded by the v0.15.0 ROADMAP above. ROADMAP at `.planning/milestones/v0.14.0-phases/ROADMAP.md`.
- **Workstream B — v0.13.2 cross-link-fidelity.** QUEUED behind workstream C AND the now-scoped v0.15.0 launch-readiness milestone. Original placeholder range P98–P107 shifts again when eventually replanned (renumber-on-insertion convention — collides with the now-shipped v0.13.1/v0.14.0 ranges). ROADMAP at `.planning/milestones/v0.13.2-phases/ROADMAP.md`.

**Last shipped milestone:** v0.14.0 (SHIPPED + Latest 2026-07-14).

**Cargo serialization rule (CLAUDE.md memory budget):** only ONE cargo invocation at a time. The separate-worktrees caveat is moot under OD-3 serial execution (single working branch: main); doc-only / planning-only subagents can still run truly concurrent with one cargo subagent.

> Superseded phase-decomposition + pre-kickoff scaffolding narrative →
> `.planning/STATE-history.md`.

### Quick Tasks Completed

| # | Description | Date | Commit | Directory |
|---|-------------|------|--------|-----------|
| 260501-mgn | Polish 5 cold-reader nits in DVCS docs | 2026-05-01 | 2b9e9c9 | [260501-mgn-polish-5-cold-reader-nits-in-dvcs-docs-b](./quick/260501-mgn-polish-5-cold-reader-nits-in-dvcs-docs-b/) |
| 260706-rbf | RBF-LR-03 honest WAIVED known-limitation across ADR-010 §3 + troubleshooting + dvcs-topology | 2026-07-06 | dfc3a9b | [260706-rbf-rbf-lr-03-known-limitation](./quick/260706-rbf-rbf-lr-03-known-limitation/) |
| 260706-crf | DVCS cold-reader fixes — 7 findings across dvcs-topology + dvcs-mirror-setup + troubleshooting (findings 1 & 6 verified against code) | 2026-07-06 | (this commit) | [260706-crf-dvcs-cold-reader-fixes](./quick/260706-crf-dvcs-cold-reader-fixes/) |
| 260706-idp | v0.13.0 intake OP-8 disposition + bound-to-live-state sweep — carry-forward banners; 2 terminal SURPRISES + 4 completed RESOLVING-P97 rows deleted; 5 HIGHs confirmed live; 1 new MEDIUM filed (troubleshooting.md >20k) | 2026-07-06 | (this commit) | [260706-idp-v0.13.0-intake-disposition](./quick/260706-idp-v0.13.0-intake-disposition/) |
| 260712-bgv | Non-blocking timing-budget warning in pre-commit/pre-push hooks (SECONDS-based, stderr-only, never touches exit code) + CLAUDE.md/quality/CLAUDE.md cadence+scaling documentation | 2026-07-12 | b4e96d8 | [260712-bgv-add-non-blocking-timing-budget-warning-t](./quick/260712-bgv-add-non-blocking-timing-budget-warning-t/) |
| 260712-oa9 | 75% file-size early-warning tier in structure/file-size-limits.sh — non-blocking print-only WARN summary for the 75–99% band (top-12 by pct DESC + overflow), always emitted independent of --warn-only, never touches exit code; ≥100% block/waiver semantics unchanged. + catalog asserts + quality/CLAUDE.md § File-size limits + committed selftest | 2026-07-12 | (this commit) | [260712-oa9-file-size-75pct-warn](./quick/260712-oa9-file-size-75pct-warn/) |
| 260712-oke | Landed all 7 v0.14.0 DEFERRED surprises-intake entries + 1 hygiene row onto the v0.15.0 surface — new v0.15.0-phases/GOOD-TO-HAVES.md (GTH-V15-01..08, severity + fix-sketch each; shell-coverage cross-refs 999.5/999.6 not duplicated; release-plz code.json blocker noted cleared) + ROADMAP.md § Hardening candidates with 2 HIGH `Phase (candidate)` stubs (RBF-LR-03 modern-git verify, subprocess-bypass binary-side refusal). UX Phase TBD stubs untouched; part-file back-pointers skipped (already >20k ceiling). | 2026-07-12 | (this commit) | [260712-oke-land-v0-14-deferred-onto-v0-15](./quick/260712-oke-land-v0-14-deferred-onto-v0-15/) |
| 260712-phc | Author the two missing pre-release-real-backend verifier scripts (B4 t4-conflict-rebase-ancestry-real-backend P0 + B5 github-front-door-real-backend P1) that blocked the v0.14.0 tag — ported sim-arm topology to confluence::TokenWorld / init github::reubenjohn/reposix, env-gate-first→exit75 NOT-VERIFIED, hermetic self-test (4/4), kcov harness (coverage 15.72%≥floor13), B5 catalog RFC3339 fix-twice. Both rows now grade instead of 'verifier not found'. | 2026-07-13 | fe8febb | [260712-phc-author-two-missing-pre-release-real-back](./quick/260712-phc-author-two-missing-pre-release-real-back/) |
| 260713-arc | Durably archive owner's 2026-07-12 reality-check audit (verbatim `cp`, 43492 bytes, byte-identical) to `.planning/milestones/audits/2026-07-12-reality-check.md` per "uncommitted = didn't happen" | 2026-07-13 | (this commit) | [260713-arc-archive-reality-check](./quick/260713-arc-archive-reality-check/) |
| 260713-q0e | Fix RED main (HONEST-REWORK, Manager Ruling #5 Option A) — post-release `quality-post-release` (run 29298424648, v0.14.0) went RED on 4 P1 docs-repro example gates (01/02/04/05); root cause = harness gap, NOT product: containers exit 0 but the generic `container-rehearse.sh` emitted one generic `asserts_passed` line, which F-K4b (`_audit_field.py::asserts_congruent`) rejects. The first fix (`0f2b7c5`, emit `expected.asserts` verbatim on exit 0) failed adversarial verification as a SYMPTOM-FIX — example-05's asserts overclaimed. Reworked: `git reset --soft d68fa8a` un-stacked the held commits; KEPT the emission (verified fail-loud for 01/02/04) and REWORDED example-05 asserts #2/#3 to the truth (pre-emptive `git sparse-checkout` pattern + `BLOB_LIMIT_EXCEEDED_FMT` source-constant presence — NOT a runtime-error observation; #3 scoped to the `ls issues/*.md` ≥1-file check). NO F-K4b weakening, NO waivers. Filed ONE v0.15.0 SURPRISES-INTAKE (MEDIUM): F-K4b container-tautology redesign + example-05 real-runtime-error deeper fix. post-release re-run: 6 PASS / 0 FAIL / exit 0. No push (orchestrator-gated). | 2026-07-13 | 03e7a6f (fix), 3775075 (intake) | [260713-q0e-fix-red-main-container-rehearse-sh-emits](./quick/260713-q0e-fix-red-main-container-rehearse-sh-emits/) |
| 260713-rug | Green RED-main `docs-repro/example-04-conflict-resolve` (FAILED at exactly 300.00s in `quality-post-release` run 29301412750, sha 05aa23c) via TIMEOUT-BUDGET fix (ruling b773c04). Diagnosis (opus repro): not a hang — the example runs ~0.5s and passes all 3 asserts; the 300s cap was eaten by per-container-row SETUP `apt-get install ... build-essential pkg-config libssl-dev ...`, compile-time deps NEVER exercised (examples run the host-mounted pre-built `target/debug/reposix`; no in-container cargo build). Two clean edits: (a) `container-rehearse.sh` SETUP drops `build-essential pkg-config libssl-dev`, keeps `curl ca-certificates python3 git sqlite3` + fix-it-twice comment; (b) `docs-reproducible.json` bumps `timeout_s` 300→600 symmetrically on all 4 `kind:container` rows (01/02/04/05), non-container rows untouched (tutorial-replay stays 300), JSON revalidated. Prove-before-fix: all 4 container rows rc=0 locally (01:16s, 02:15s, 04:16s, 05:19s), asserts_failed []. NO assert/waiver/example-proof touched (honesty guard CLEAN). No push (orchestrator-gated). | 2026-07-13 | (this commit) | [260713-rug-example04-timeout-budget](./quick/260713-rug-example04-timeout-budget/) |
| 260714-rcv | Post-tag cursor refresh + carried-noticing intake filing (L0 rotation #21: STATE.md cursor → post-tag queue 0-5 CLOSED green + Arc D RATIFIED at 6aa734a + re-anchor ACTIVE; 2 carried noticings filed to v0.15.0 SURPRISES-INTAKE — GTH oversize masked by 08-08 waiver + v0.13.0 ROADMAP broken plan links) | 2026-07-14 | (this commit) | [260714-rcv-cursor-refresh-intake](./quick/260714-rcv-cursor-refresh-intake/) |
| 260715-h1d | Directive 2 (5-rotation starvation ended): record `reposix-scope-test-DELETEME` scratch-repo KEEP-policy in `docs/reference/testing-targets.md` — throwaway private GitHub scratch *remote-target*; NEVER-delete / reset-via-force-push (URL stays stable across sessions); currently ARCHIVED per live `gh api` (`archived:true, private:true, pushed 2026-07-14`); unarchive via `gh api -X PATCH ... archived=false` before first reuse. + eager-fixed a stale "Phase 36 cleanup automation will handle this" lying-doc forward-ref (verified no such automation ever shipped) → present-tense manual cleanup. mkdocs-strict GREEN, one-file diff. | 2026-07-15 | a165d48 | [260715-h1d-scratch-repo-keep-policy](./quick/260715-h1d-scratch-repo-keep-policy/) |
| 260715-mk5 | Public birds-eye roadmap diagram — new `docs/roadmap.md` (ONE color-coded mermaid: shipped → active Floor → future arcs converging on OD-4 launch; arcs/capabilities, no phase numbers/dates), registered in mkdocs nav. Bi-directional `<!-- SYNC: -->` cross-links docs/roadmap.md ↔ .planning/PROJECT.md (docs→PROJECT via GitHub URL, PROJECT→docs via `../docs/roadmap.md`) + OP-9 distill-checklist reminder in PRACTICES.md. Extended `link-resolution.py` DEFAULT_GLOBS (catalog-first: docs-build.json assert) to cover `docs/*.md` + `.planning/PROJECT.md` so both directions are mechanically link-checked. Gates GREEN: mkdocs-strict, mermaid-renders (playwright roadmap.json, 0 console errors), link-resolution (0/30), banned-words. mmdc render-review ×3. Optional SYNC-marker gate FILED as GTH-V15-24. No push (orchestrator-gated). | 2026-07-15 | (this commit) | [260715-mk5-public-birds-eye-roadmap-diagram-author-](./quick/260715-mk5-public-birds-eye-roadmap-diagram-author-/) |
| 260716-f6o | Fix-it-twice for owner ruling 5a5dd29 — the perf-gate GENERATOR (`bench_token_economy_captures.py::render_token_economy_markdown`) still templated the "## What retired the old 89.1% / 85.5% figures" section that 5a5dd29 stripped from `docs/benchmarks/token-economy.md`; the P115 phase-close gate-run regen re-added it in place (dirty +12 lines). Manager-established provenance: accidental regression vector, not a deliberate override. Stripped the section from the template; offline regen now byte-for-byte matches committed HEAD (sha256 `5620699b...364fcf`, empty diff); stray working-tree re-add discarded (belt-and-suspenders `git checkout --`). Verified no doc-alignment catalog rebind needed (BOUND rows are the live four-axis claims; catalog untouched). Pushed, all 61 pre-push gates PASS (1 pre-existing WAIVED). | 2026-07-16 | 19f9ae2 | [20260716-token-economy-generator-strip](./quick/20260716-token-economy-generator-strip/) |
| 260716-fmt | GTH-V15-35 docs/index.md install-IA fix (both addenda) — relocated "Build from source (advanced)" `<details>` block from L120-136 to under the 30-second install tabs (new L69-85, install-leads GREEN); surfaced `reposix sim &` / `reposix init sim::demo` / `git checkout -B main` bootstrap commands in visible prose so the "After — one commit" demo's `/tmp/reposix-demo` has a visible creation step; split the stale two-claim L93 into two lines, replacing "Real-backend cells fill in once CI secret packs are wired (Phase 36)" with the real GitHub 320 ms / Confluence 202 ms get-one-record figures (latency.md:42). Mechanically rebound all 11 shifted/changed doc-alignment rows via `doc-alignment bind` (no hand-edit, no fan-out, no cargo); `walk.sh` exit 0, zero STALE_DOCS_DRIFT. Filed one MEDIUM SURPRISES-INTAKE row (token-economy regen test's missing byte-compare-against-committed-doc coverage). GTH-V15-35 STATUS → DONE. Pushed, main CI green. | 2026-07-16 | 97fad0d | [20260716-gth-v15-35-docs-index-install-ia](./quick/20260716-gth-v15-35-docs-index-install-ia/) |

## Session Continuity

Frontmatter (above) is the machine-readable cursor. Resume via `/gsd-resume-work`; current live cursor is "**v0.15.0 Floor — P121 CLOSED GREEN (verdict GREEN); 8/15 phases complete (P114–P121). Next: `/gsd-plan-phase 122`.**" (see § Current Position / Current Focus above). Workstream A (v0.13.0-extension) and Workstream C (v0.14.0) are CLOSED/SHIPPED historically (tags landed). Workstream B (v0.13.2) stays queued behind v0.15.0 per OD-3/OD-4.

Top-level session handover: `.planning/SESSION-HANDOVER.md` (whole-session rotation handover for session 7e2a4cf2, 2026-07-04/05; distinct from per-phase relief handovers under `.planning/phases/`).

> Closed/historical cursor detail (Workstreams A & B, per-milestone cross-refs,
> Project Reference, resolved Blockers/Concerns) → `.planning/STATE-history.md`.
