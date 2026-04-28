---
gsd_state_version: 1.0
milestone: v0.12.0
milestone_name: Quality Gates
status: ready-to-tag
last_updated: "2026-04-28T05:00:20.000Z"
last_activity_p63: "2026-04-28 -- P63 SHIPPED. v0.12.0 milestone-close verdict GREEN at quality/reports/verdicts/p63/VERDICT.md. All 8 phases (P56-P63) shipped. SIMPLIFY-12 audit closed: 5 DELETE / 13 SHIM-WAIVED / 4 KEEP-AS-CANONICAL per quality/reports/audits/scripts-retirement-p63.md. MIGRATE-01 retirements landed in commit 4950cdd. POLISH-CODE final: code/cargo-fmt-clean PASS via direct cargo fmt invocation through quality/gates/code/cargo-fmt-clean.sh (read-only ~5s, ONE cargo at a time safe); code/cargo-test-pass stays as ci-status canonical per memory-budget rule (tracked-forward to v0.12.1 MIGRATE-03 for per-row local cargo alternatives). MIGRATE-02 cohesion: cross-link audit verifier shipped at quality/gates/structure/cross-link-audit.py (100 paths verified, 0 stale); per-dim READMEs normalized; new quality/gates/security/README.md fills missing security home. MIGRATE-03 v0.12.1 carry-forward filed at .planning/milestones/v0.12.1-phases/{ROADMAP,REQUIREMENTS}.md (15 REQ placeholders: PERF-01..03, SEC-01..02, CROSS-01..02, MSRV-01, BINSTALL-01, LATEST-PTR-01, RELEASE-PAT-01, ERR-OTHER-01, SUBJ-RUNNER-01..HARDGATE-01; 5 placeholder phases P64-P68). New cross-link verifier quality/gates/structure/catalog-tracked-in-cross-link.py asserts 4/4 catalog tracked_in REQ-IDs resolve. CHANGELOG [v0.12.0] finalized. Tag-gate script ready at .planning/milestones/v0.12.0-phases/tag-v0.12.0.sh (6 guards including P63 verdict GREEN). All 6 cadences exit 0 (pre-push 19 PASS / pre-pr 2 PASS + 3 WAIVED / weekly 13 PASS + 1 P2-FAIL + 4 WAIVED + 2 P2-NOT-VERIFIED / pre-release 4 WAIVED / post-release 6 WAIVED / on-demand 1 PASS). 5 requirements flipped to shipped (P63): MIGRATE-01, MIGRATE-02, MIGRATE-03, SIMPLIFY-12, POLISH-CODE. Owner next step: run .planning/milestones/v0.12.0-phases/tag-v0.12.0.sh; orchestrator does NOT push the tag. Cursor advances to v0.12.1 planning."
last_activity: "2026-04-28 -- P62 SHIPPED. Repo-org-gaps cleanup + audit closure (ORG-01 + POLISH-ORG). Audit at quality/reports/audits/repo-org-gaps.md (99 items; 13 closed-by-deletion, 26 closed-by-relocation, 50 closed-by-existing-gate, 8 out-of-scope; zero open Wave-3 items). 3 new structure-dimension rows in quality/catalogs/freshness-invariants.json (structure/no-loose-top-level-planning-audits, structure/no-pre-pivot-doc-stubs, structure/repo-org-audit-artifact-present); short-lived waivers until 2026-05-15 expire harmlessly after Wave 3 verifier extension. Verifier branches at quality/gates/structure/freshness-invariants.py (file 402 lines; helper-module extraction deferred to v0.12.1 unless Wave 6 flags it). New scripts/check_repo_org_gaps.py (stdlib audit-completeness verifier; --json mode for machine-readable summary). Relocations: git mv .planning/v0.{9,10}.0-MILESTONE-AUDIT.md to .planning/milestones/audits/; git mv .planning/SESSION-END-STATE{.md,.json,-VERDICT.md} to .planning/archive/session-end-state/ + new README.md naming quality/PROTOCOL.md as supersession. scripts/__pycache__/ purged (workspace-only; never tracked; .gitignore:30 covers it). SURPRISES.md rotation Wave 4: active 302 -> 219 lines via P59 precedent; 10 P57+P58 entries (106 lines) archived to quality/SURPRISES-archive-2026-Q2.md verbatim; banner consolidated. CLAUDE.md gained 36-line H3 subsection 'P62 -- Repo-org-gaps cleanup + audit closure (added 2026-04-28)' under Quality Gates section + meta-rule extension (every audit/cleanup phase MUST commit its closure record under quality/reports/audits/<topic>.md with a machine-checkable verifier). .planning/research/v0.11.1-repo-organization-gaps.md gained top blockquote banner naming P62 closure path; doc is now historical reference. 2 requirements flipped to shipped (P62): ORG-01, POLISH-ORG. Runner state: pre-push exit 0 (19 PASS + 3 WAIVED); the 3 P62 catalog rows pass when re-graded (waivers stay armed as safety net until 2026-05-15). Verifier subagent verdict pending Wave 6 dispatch. Next: P62 Wave 6 verifier dispatch + P63 (retire migrated sources + cohesion audit + v0.12.1 carry-forward consolidation)."
last_activity_p61_archived: "2026-04-27 -- P61 SHIPPED. Subjective gates skill + freshness-TTL enforcement. quality/catalogs/subjective-rubrics.json ships 3 seed rubrics (cold-reader-hero-clarity P1 pre-release; install-positioning P0 pre-release; headline-numbers-sanity P2 weekly; freshness_ttl=30d). .claude/skills/reposix-quality-review/ NEW skill (only skill change pre-approved for v0.12.0): SKILL.md + dispatch.sh + lib/{catalog.py, persist_artifact.py, dispatch_cli.py} + 3 rubric prompts (rubrics/{cold-reader-hero-clarity, install-positioning, headline-numbers-sanity}.md) + 2 dispatchers (lib/{dispatch_cold_reader.sh wraps doc-clarity-review subprocess; dispatch_inline_subagent.sh routes via Path A Task / Path B claude -p}) + lib/test_lib_smoke.py (4 PASS). SUBJ-03 freshness-TTL closure: quality/runners/run.py grew parse_duration + is_stale + STALE-counts-as-NOT-VERIFIED branch + [STALE] label (final 388 LOC under 390 cap; parse_duration extracted to quality/runners/_freshness.py per pivot rule); quality/runners/verdict.py grew STALE sub-table (final 301 LOC under 305 cap); quality/runners/test_freshness.py 11 PASS + test_freshness_synth.py 1 PASS end-to-end synthetic regression. .github/workflows/quality-pre-release.yml NEW (75 LOC; triggers on v* + reposix-cli-v* + workflow_dispatch). .github/workflows/release.yml gains parallel-execution comment + v0.12.1 hard-gate carry-forward note (cross-workflow needs: unsupported per P56 SURPRISES row 1). POLISH-SUBJECTIVE Wave G: 3 rubrics graded via Path B (in-session Claude grading) -- cold-reader 8 CLEAR, install-positioning 9 CLEAR, headline-numbers 9 CLEAR; zero P0/P1 findings (broaden-and-deepen confirmed user-facing prose already CLEAR); 4 P2 polish items deferred to v0.12.1; catalog rows extended to WAIVED-2026-07-26 with documented evidence + carry-forward. CLAUDE.md gained P61 H3 subsection. v0.12.1 MIGRATE-03 carry-forwards: (e) subjective dispatch-and-preserve runner invariant; (f) auto-dispatch from CI; (g) hard-gate chaining release.yml -> quality-pre-release.yml. Verifier subagent verdict GREEN at quality/reports/verdicts/p61/VERDICT.md (Path B in-session disclosure per P56/P57/P58/P59/P60 precedent). 4 requirements flipped to shipped (P61): SUBJ-01, SUBJ-02, SUBJ-03, POLISH-SUBJECTIVE. Next: P62 -- repo-org-gaps cleanup (ORG-01 + POLISH-ORG)."
progress:
  total_phases: 8
  completed_phases: 8
  total_plans: 50
  completed_plans: 50
  percent: 100
---

# Project State

## Accumulated Context

### Roadmap Evolution

- **P62 SHIPPED (2026-04-28, v0.12.0):** Repo-org-gaps cleanup + audit closure (ORG-01 + POLISH-ORG). The forgotten-todo-list document `.planning/research/v0.11.1-repo-organization-gaps.md` (snapshot 2026-04-26) was audited rec-by-rec; closure record at `quality/reports/audits/repo-org-gaps.md` (99 items; 13 closed-by-deletion, 26 closed-by-relocation, 50 closed-by-existing-gate, 8 out-of-scope; **zero open Wave-3 items**). **Wave 1 (catalog-first commit `eaf7068`)**: 3 new structure-dimension rows in `quality/catalogs/freshness-invariants.json` (`structure/no-loose-top-level-planning-audits`, `structure/no-pre-pivot-doc-stubs`, `structure/repo-org-audit-artifact-present`); short-lived waivers until 2026-05-15 (catalog-first pattern); `quality/gates/structure/README.md` NEW (35-line dimension home with P62 row table). **Wave 2 (audit `4584fca`)**: `quality/reports/audits/repo-org-gaps.md` materialized + `scripts/check_repo_org_gaps.py` NEW (stdlib audit-completeness verifier; --json mode for machine-readable summary). Wave 2 pre-check revealed ~50/99 audited items were already closed by SIMPLIFY-04..11 + P56-P61 sweeps (the audit doc snapshot lagged 16 days behind current state). **Wave 3 (relocations `8842d48` + verifier extension `9011e91`)**: `git mv .planning/v0.{9,10}.0-MILESTONE-AUDIT.md .planning/milestones/audits/`; `git mv .planning/SESSION-END-STATE{.md,.json,-VERDICT.md} .planning/archive/session-end-state/` + new `README.md` naming `quality/PROTOCOL.md` as supersession; `scripts/__pycache__/` purged from working tree (was workspace-only, never tracked; `.gitignore:30` covers it). `quality/gates/structure/freshness-invariants.py` extended with 3 verifier branches (file 402 lines; helper-module extraction deferred to v0.12.1 unless Wave 6 flags it). Audit table flipped: every Wave-3-fix Disposition replaced with its post-fix value. **Wave 4 (SURPRISES rotation `2413f13`)**: active 302 → 219 lines via P59 precedent; 10 P57+P58 entries (106 lines) archived to `quality/SURPRISES-archive-2026-Q2.md` verbatim; banner consolidated into a single "Archive rotations" section. **Wave 5 (CLAUDE.md `79319f6` + STATE/REQUIREMENTS this commit)**: CLAUDE.md gained 36-line H3 subsection 'P62 -- Repo-org-gaps cleanup + audit closure' under Quality Gates section + meta-rule extension (every audit/cleanup phase MUST commit its closure record under `quality/reports/audits/<topic>.md` with a machine-checkable verifier). `.planning/research/v0.11.1-repo-organization-gaps.md` gained top blockquote banner naming P62 closure path; doc is now historical reference. Two audit false-positives reclassified during Wave 3: `docs/development/roadmap.md` 3 fuse mentions are historical release-notes context (allowed pattern, like CHANGELOG); `docs/development/contributing.md` "fuse" was a substring match inside "**re**fuse" — both closed-by-existing-gate without code change. Lesson: future audits should `grep -w` (word boundary) for jargon-residue counts. **Verifier subagent verdict pending Wave 6 dispatch.** Runner state: pre-push exit 0 (19 PASS + 3 WAIVED; the 3 P62 catalog rows pass when re-graded but waivers stay armed as safety net until 2026-05-15). 2 requirements flipped to shipped (P62): ORG-01, POLISH-ORG. v0.12.1 MIGRATE-03 carry-forwards: existing (a-g) continue; new (h) potential helper-module extraction for `quality/gates/structure/freshness-invariants.py` if Wave 6 flags the 402-LOC overshoot. Next: P62 Wave 6 verifier dispatch + P63 (retire migrated sources + cohesion audit + v0.12.1 carry-forward consolidation).
- **P61 SHIPPED (2026-04-27, v0.12.0):** Subjective gates skill + freshness-TTL enforcement. `quality/catalogs/subjective-rubrics.json` ships 3 seed rubrics (cold-reader-hero-clarity P1 pre-release; install-positioning P0 pre-release; headline-numbers-sanity P2 weekly; freshness_ttl=30d). **SUBJ-02 closure**: `.claude/skills/reposix-quality-review/` NEW skill (the ONLY skill change pre-approved for v0.12.0 per `.planning/research/v0.12.0-open-questions-and-deferrals.md` line 124) — SKILL.md (113 LOC; --rubric / --all-stale / --force / usage modes); dispatch.sh (90 LOC; chmod 755; routes per rubric ID); lib/catalog.py (59 LOC; cross-imports `is_stale` from `quality/runners/run.py`); lib/persist_artifact.py (91 LOC; argparse CLI for bash subprocess use; canonical artifact JSON shape `{ts, rubric_id, score, verdict, rationale, evidence_files, dispatched_via, asserts_passed/failed, stale}`); lib/dispatch_cli.py (82 LOC; list-stale / list-all / stub subcommands); lib/test_lib_smoke.py (4 PASS); 3 rubric prompts (cold-reader-hero-clarity.md 34 LOC consumed by `claude /doc-clarity-review` subprocess; install-positioning.md 46 LOC inline-dispatch; headline-numbers-sanity.md 48 LOC inline-dispatch); 2 dispatchers (lib/dispatch_cold_reader.sh 74 LOC wraps doc-clarity-review; lib/dispatch_inline_subagent.sh 95 LOC routes install-positioning + headline-numbers via Path A Task / Path B claude -p; benchmarks/ + docs/benchmarks/ source-of-truth files capped at 18 to keep prompt sane). **SUBJ-03 freshness-TTL closure**: `quality/runners/run.py` grew `parse_duration` + `is_stale` + STALE-counts-as-NOT-VERIFIED branch + `[STALE]` label (final 388 LOC under 390 cap; parse_duration extracted to `quality/runners/_freshness.py` 50 LOC per pivot rule when 399-LOC peak hit Wave B cap); `quality/runners/verdict.py` grew STALE sub-table inside NOT-VERIFIED rollup (final 301 LOC under 305 cap); `quality/runners/test_freshness.py` 11 PASS pytest tests covering parse_duration (4) + is_stale (4) + STALE+WAIVED waiver dominance (1) + STALE+P1 blocks exit_code (1) + STALE+P2 does not block exit_code (1); `quality/runners/test_freshness_synth.py` 1 PASS end-to-end synthetic regression (backdate headline-numbers row + run weekly subprocess + assert `[STALE` label appears). `.github/workflows/quality-pre-release.yml` NEW (75 LOC; triggers on push.tags=[v*, reposix-cli-v*] + workflow_dispatch; runs `python3 quality/runners/run.py --cadence pre-release`; emits `::error::` hints with dispatcher invocation on failure). `.github/workflows/release.yml` gains 11-LOC parallel-execution comment block + v0.12.1 hard-gate carry-forward note (cross-workflow `needs:` unsupported per P56 SURPRISES row 1). **POLISH-SUBJECTIVE Wave G broaden-and-deepen**: 3 rubrics graded via Path B (in-session Claude grading; full disclosure in artifact dispatched_via=Wave-G-Path-B-in-session): cold-reader 8 CLEAR, install-positioning 9 CLEAR, headline-numbers 9 CLEAR. ZERO P0/P1 findings — broaden-and-deepen confirmed user-facing prose already CLEAR; 4 P2 polish items (MCP acronym un-glossed; "promisor remote" jargon; docs/index.md target-arch surfacing; "5-line install" approximation) deferred to v0.12.1 docs polish. Catalog rows extended to WAIVED-2026-07-26 (was Wave A short-lived 2026-05-15) with documented Path B evidence + v0.12.1 carry-forward (the runner subprocess overwrites Path A artifacts; "subjective dispatch-and-preserve runner invariant" filed). CLAUDE.md gained "P61 -- Subjective gates skill + freshness-TTL" H3 subsection (~70 added lines under 80 cap; banned-words clean) + 2 in-place updates ("Cold-reader pass" pointer + "Subagent delegation rules" table row). SURPRISES.md gained P61 pivot entries. Verifier subagent verdict GREEN at `quality/reports/verdicts/p61/VERDICT.md` (Path B in-session disclosure per P56/P57/P58/P59/P60 precedent; 4 disclosure constraints honored). 4 requirements flipped to shipped (P61): SUBJ-01, SUBJ-02, SUBJ-03, POLISH-SUBJECTIVE. v0.12.1 MIGRATE-03 carry-forwards added: (e) subjective dispatch-and-preserve runner invariant; (f) auto-dispatch on STALE rows from CI (Anthropic API auth on GH Actions runner); (g) hard-gate chaining release.yml -> quality-pre-release.yml (composite workflow OR workflow_run trigger). Existing carry-forwards continue. Next: P62 — repo-org-gaps cleanup (ORG-01 + POLISH-ORG).
- **P60 SHIPPED (2026-04-27, v0.12.0):** Docs-build dimension migration + composite cutover. 4 docs-build verifiers ship at canonical home (`quality/gates/docs-build/{mkdocs-strict.sh, mermaid-renders.sh, link-resolution.py, badges-resolve.py}`) backing 4 catalog rows in `quality/catalogs/docs-build.json` + the back-compat `structure/badges-resolve` row in `quality/catalogs/freshness-invariants.json` (P57 pre-anchored; P60 implements via shared verifier). **SIMPLIFY-08 closure**: 3 git-mv migrations preserve history (`scripts/check-docs-site.sh` → `mkdocs-strict.sh`; `scripts/check-mermaid-renders.sh` → `mermaid-renders.sh`; `scripts/check_doc_links.py` → `link-resolution.py`); thin shims at old paths per OP-5 reversibility so callers continue working. **SIMPLIFY-09 closure**: `scripts/green-gauntlet.sh` rewritten as a thin shim delegating to `python3 quality/runners/run.py --cadence pre-pr` (3 modes collapse to runner cadence calls). **SIMPLIFY-10 closure**: `scripts/hooks/pre-push` body collapsed from 229 lines (6 chained verifiers + parallel-migration runner block) to 40 lines total / 10 body lines — a cred-hygiene wrapper invocation (P0 fail-fast on stdin push-ref ranges) followed by a single `exec python3 quality/runners/run.py --cadence pre-push`. Warm-cache profile 5.3s on second run; well under the 60s pivot threshold so cargo fmt + clippy stay routed through the runner via the Wave D code-dimension wrappers. All 6 hook tests (test-pre-push.sh) PASS against the new hook. **BADGE-01 verifier** ships (`quality/gates/docs-build/badges-resolve.py`; stdlib-only `urllib.request` HEAD; ~165 lines; 8 unique URL set extracted from README + docs/index.md). **QG-09 P60 closure**: `docs/badge.json` seeded from `quality/reports/badge.json` and auto-included in mkdocs build → `https://reubenjohn.github.io/reposix/badge.json` resolves with HTTP 200 + Content-Type `application/json` within ~90s of the Wave F push commit. README.md (8 badges, was 7) + docs/index.md (3 badges, was 2) gain `![Quality](https://img.shields.io/endpoint?url=...)` after the existing Quality (weekly) GH Actions badge from P58. The two convey complementary signals (workflow status vs catalog rollup). `WAVE_F_PENDING_URLS` cleared in `badges-resolve.py`; verifier now HEADs all 8 URLs unconditionally + 8/8 PASS. **POLISH-DOCS-BUILD broaden-and-deepen sweep (Wave G)**: 4 cadences GREEN at sweep entry (zero RED to fix because Waves A-F left the dimension pristine). pre-push 19 PASS / pre-pr 1 PASS + 2 WAIVED / weekly 14 PASS + 3 WAIVED + 2 P2 NOT-VERIFIED non-blocking / post-release 6 WAIVED. New artifact `quality/runners/check_p60_red_rows.py` reads 3 P60-relevant catalogs and reports per-row grades for the 8 P60-touched rows (promoted from ad-hoc bash per CLAUDE.md §4 self-improving infrastructure). CLAUDE.md gained "P60 — Docs-build dimension live + composite cutover" H3 subsection (34 added lines under 80-line P58/P59 precedent cap; banned-words-lint clean). SURPRISES.md gained 4 P60 entries (200 → 218 lines; deferred archive rotation since marginal). Verifier subagent verdict GREEN at `quality/reports/verdicts/p60/VERDICT.md` (Path B in-session disclosure per P56/P57/P58/P59 precedent — Task tool unavailable in executor; 4 disclosure constraints honored). 8 catalog rows graded; 8/8 P0+P1 rows PASS. 6 requirements flipped to shipped (P60): DOCS-BUILD-01, BADGE-01, SIMPLIFY-08, SIMPLIFY-09, SIMPLIFY-10, POLISH-DOCS-BUILD. Carry-forwards: docs/badge.json auto-sync from quality/reports/badge.json (MIGRATE-03 v0.12.1); existing P58 cargo-binstall waiver + POLISH-CODE final + P59 docker-absent waivers + perf v0.12.1 stubs continue. Next: P61 — subjective gates skill (SUBJ-01..03 + POLISH-SUBJECTIVE).
- **P59 SHIPPED (2026-04-27, v0.12.0):** Docs-repro + agent-ux thin
  home + perf-relocate dimensions live. `quality/gates/docs-repro/`
  ships 3 verifiers backing 9 catalog rows: `snippet-extract.py`
  (--list/--check/--write-template drift detector; pre-push;
  `docs-repro/snippet-coverage` PASS), `container-rehearse.sh` (4
  example container rows + `docs-repro/tutorial-replay`; post-release;
  WAIVED until 2026-05-12 — sim-inside-container plumbing is post-v0.12.0
  work), `tutorial-replay.sh` (SIMPLIFY-06 closure: `scripts/repro-quickstart.sh`
  DELETED, no callers), `manual-spec-check.sh` (`docs-repro/example-03-claude-code-skill`
  PASS). The 2 benchmark-claim rows stay manual P2 NOT-VERIFIED
  (non-blocking by runner exit-code rules; v0.12.1 perf cross-check
  automates them). `quality/gates/agent-ux/` ships `dark-factory.sh`
  (SIMPLIFY-07 migration of `scripts/dark-factory-test.sh`; canonical
  home + 7-line shim at old path per OP-5 reversibility — 14 doc/example
  references keep working unchanged). `agent-ux/dark-factory-sim` PASS;
  POLISH-AGENT-UX broaden-and-deepen confirmed (no regression vs v0.9.0
  baseline). `quality/gates/perf/` ships 3 file-relocates via git mv
  (SIMPLIFY-11): `latency-bench.sh` + `bench_token_economy.py` (Option B
  underscore — test imports module via Python; catalog row corrected
  4-char) + `test_bench_token_economy.py` (renamed only; pytest
  auto-discovers; 9/9 tests pass). REPO_ROOT path arithmetic fixed
  (parents[3] / `../../..` for the new home depth). All 3 perf rows
  WAIVED until 2026-07-26 (MIGRATE-03 v0.12.1 carry-forward; file-relocate
  stub at v0.12.0). `.github/workflows/{ci.yml, bench-latency-cron.yml}`
  invoke canonical paths explicitly per OP-1; `scripts/{latency-bench.sh,
  bench_token_economy.py}` shims preserve OP-5 reversibility for docs +
  green-gauntlet callers. POLISH-DOCS-REPRO Wave F sweep: 4 cadences
  GREEN (pre-push 11 PASS + 1 WAIVED; pre-pr 1 PASS + 2 WAIVED;
  post-release 6 WAIVED; weekly 14 PASS + 3 WAIVED + 2 P2 NOT-VERIFIED).
  CLAUDE.md gained "P59 — Docs-repro + agent-ux + perf-relocate dimensions
  live" H3 subsection (52 added lines under 80-line P58 precedent cap).
  SURPRISES.md crossed 204 lines → archive rotation: 5 P56 entries
  archived to `quality/SURPRISES-archive-2026-Q2.md`; active journal
  retains P57 onward (3 P57 + 7 P58 + 6 P59 = 16 entries; 200 lines).
  First archive rotation since the journal was seeded — establishes
  the quarterly-archive convention. Verifier subagent verdict GREEN
  at `quality/reports/verdicts/p59/VERDICT.md` (Path B in-session
  disclosure per P56/P57/P58 precedent — Task tool unavailable in
  executor; 4 disclosure constraints honored). 13 catalog rows graded
  (9 docs-repro + 1 agent-ux + 3 perf): 8/8 P0+P1 rows PASS or WAIVED
  with documented carry-forwards. Carry-forwards: 5 docker-absent rows
  until 2026-05-12 (next phase CI rehearsal in docker-equipped GH
  runner with sim service); 3 perf rows until 2026-07-26 (MIGRATE-03
  v0.12.1 perf full implementation); P58 cargo-binstall waiver continues.
  Next: P60 docs-build migration + BADGE-01 verifier ships + SIMPLIFY-10.
- **P58 SHIPPED (2026-04-27, v0.12.0):** Release-dimension gates +
  code-dim absorption landed. `quality/gates/release/` ships 5 verifiers
  (gh-assets-present, installer-asset-bytes, brew-formula-current,
  crates-io-max-version, cargo-binstall-resolves) backing 15 catalog
  rows in `quality/catalogs/release-assets.json` (was 16; Wave E removed
  `release/crates-io-max-version/reposix-swarm` because the crate has
  `publish = false` — internal multi-agent contention test harness;
  catalog drift fix per Wave A SURPRISES.md entry). `quality/gates/code/`
  ships `clippy-lint-loaded.sh` (SIMPLIFY-04 migration of
  `scripts/check_clippy_lint_loaded.sh`; old path DELETED),
  `check-fixtures.py` (SIMPLIFY-05 Option A migration of
  `scripts/check_fixtures.py`; old path DELETED), `ci-job-status.sh`
  (POLISH-CODE P58-stub thin gh-CLI wrapper) backing 4 catalog rows in
  `quality/catalogs/code.json` (clippy-lint-loaded + fixtures-valid PASS;
  cargo-test-pass + cargo-fmt-clean WAIVED until P63 final).
  `.github/workflows/quality-weekly.yml` + `quality-post-release.yml`
  validated end-to-end via 4 workflow_dispatch runs; 2 fixes applied:
  GH_TOKEN env added to runner + verdict steps (commit `664b533` —
  gh CLI in GH Actions needs explicit `GH_TOKEN: ${{ github.token }}`);
  cargo-binstall-resolves PARTIAL_SIGNALS broadened to match real binstall
  output (commit `e0e5645` — `will be installed from source` was the
  missing match string). QG-09 P58 GH Actions badge live in `README.md`
  + `docs/index.md` (commit `d206b09`). `quality/catalogs/orphan-scripts.json`
  shrunk to empty (W-2 waiver removed; `release/crates-io-max-version`
  row obsolete now that the dimension provides active enforcement).
  POLISH-RELEASE closure: every release-dim weekly row PASS;
  cargo-binstall-resolves WAIVED until 2026-07-26 with documented
  MIGRATE-03 v0.12.1 carry-forward. POLISH-CODE P58-stub: clippy +
  fixtures PASS; test + fmt WAIVED until P63. Verifier subagent verdict
  GREEN at `quality/reports/verdicts/p58/VERDICT.md` (Path B in-session
  disclosure per P56/P57 precedent — Task tool unavailable in executor).
  Runner state: weekly exit 0 (14 PASS), post-release exit 0 (1 WAIVED),
  pre-push exit 0 (10 PASS, 1 WAIVED), pre-pr exit 0 (2 WAIVED). Badge
  brightgreen, message `25/25 GREEN`. CLAUDE.md gained "P58 — Release
  dimension live" H3 subsection (34 added lines, anti-bloat compliant).
  SURPRISES.md gained 7 P58 entries (172 lines total; under 200 cap).
  `.planning/docs_reproducible_catalog.json` schema_status flipped to
  DEPRECATED — install rows have lineage in release-assets.json; file
  slated for deletion in P63 SIMPLIFY-12. Carry-forwards: cargo-binstall
  full PASS deferred to MIGRATE-03 v0.12.1; POLISH-CODE final P63;
  BADGE-01 P60. Next: P59 docs-repro dimension + tutorial replay +
  agent-ux thin home (per ROADMAP.md).
- **P57 SHIPPED (2026-04-27, v0.12.0):** Quality Gates skeleton +
  structure dimension migration. Files landed: `quality/{gates,catalogs,
  reports,runners}/` with `quality/PROTOCOL.md` (autonomous-mode runtime
  contract) + `quality/SURPRISES.md` ownership transferred from P56.
  `quality/catalogs/freshness-invariants.json` carries 9 rows (6
  freshness invariants migrated from `scripts/end-state.py` +
  `structure/banned-words` + `structure/top-level-requirements-roadmap-scope`
  (QG-08) + `structure/badges-resolve` (BADGE-01 P57 stub, waived until
  2026-07-25 — verifier ships P60)). `quality/runners/{run,verdict}.py`
  are stdlib-only Python cross-platform; `quality/runners/verdict.py`
  emits `quality/reports/badge.json` in shields.io endpoint format
  (QG-09 P57 scope; P60 publishes via mkdocs).
  `quality/gates/structure/freshness-invariants.py` ships the 6+1
  verifier functions; `quality/gates/structure/banned-words.sh` wraps
  `scripts/banned-words-lint.sh --all` (SIMPLIFY-01 closure via
  Approach A wrapper). `scripts/end-state.py` reduced to ≤30-line shim
  that delegates to `python3 quality/runners/verdict.py session-end`
  (STRUCT-02 + SIMPLIFY-02). POLISH-STRUCT (Wave D) moved ~480 lines
  of historical milestone content from `.planning/ROADMAP.md` to
  `.planning/milestones/v0.X.0-phases/ROADMAP.md` (3 files: v0.11.0
  NEW, v0.10.0 + v0.9.0 PRESERVED); top-level ROADMAP.md 704 → 230
  lines, QG-08 verifier flips RED → PASS. SIMPLIFY-03 closed via Wave A
  boundary doc + Wave E audit memo (`scripts/catalog.py` left in
  place per the audit). Pre-push hook gains the new runner block
  alongside `scripts/end-state.py` chain (parallel migration per D4);
  hard-cut deferred to P60 SIMPLIFY-10. CLAUDE.md gains "Quality
  Gates — dimension/cadence/kind taxonomy" section (QG-07) +
  "Subagent delegation rules" QG-06 bullet. Verifier subagent verdict
  GREEN at `quality/reports/verdicts/p57/VERDICT.md` (Path B
  disclosure per P56 precedent). Carry-forwards: BADGE-01 P60,
  SIMPLIFY-04..10 P58/P59/P60, MIGRATE-01..03 P63. Next: P58 release
  dimension gates + code-dimension absorption.
- **P56 SHIPPED (2026-04-27, v0.12.0):** RELEASE-01..03 closed.
  `.github/workflows/release.yml` now fires on both `v*` and
  `reposix-cli-v*` tag globs (Option A; commit `d3f0dce`); a fresh
  `reposix-cli-v0.11.3` GH Release ships with 8 assets (5 platform
  archives + 2 installers + SHA256SUMS). 5 install paths verified
  end-to-end via `.planning/verifications/p56/install-paths/*.json`:
  curl PASS in ubuntu:24.04 rehearsal (`scripts/p56-rehearse-curl-install.sh`),
  powershell PASS asset-existence (1075B, leading bytes match), homebrew
  PASS tap formula version 0.11.3 + 3 valid sha256s + reposix-cli-v0.11.3
  pinned URLs, build-from-source PASS via ci.yml run 25005567451,
  cargo-binstall PARTIAL with documented v0.12.1 carry-forward (~10 LOC
  pkg-url rewrite + MSRV 1.82→1.85 or block-buffer cap <0.12). Wave 4
  closed phase: CLAUDE.md gained "v0.12.0 Quality Gates — phase log"
  section + container-rehearsal evidence schema + new operating principle
  7 (phase-close = catalog-row PASS) + meta-rule extension (release-pipeline
  regression fixes ship evidence in same PR); `quality/SURPRISES.md`
  created and seeded with 5 P56 carry-forwards (latest-pointer recency,
  GITHUB_TOKEN-tag trigger gap, cargo-binstall pkg-url, Rust 1.82 MSRV,
  curl-rehearsal SIGPIPE); REQUIREMENTS.md MIGRATE-03 expanded with 4
  new P56-discovered carry-forward items alongside the existing perf /
  security / cross-platform / Error::Other 156→144 list. Verifier verdict
  GREEN at `.planning/verifications/p56/VERDICT.md` (validator
  `scripts/p56-validate-install-evidence.py` re-run: 5/5 rows pass their
  gate). Carry-forwards under MIGRATE-03 now total 8 items (4 original +
  4 P56 supplements). Next: P57 framework skeleton + structure dimension
  migration (lands `quality/{gates,catalogs,reports,runners}/` layout
  + `quality/PROTOCOL.md`).
- **v0.11.0 implementation SHIPPED (2026-04-25 morning, same day):** Phases 50–55 closed in ~5 hours (with one mid-session VM-OOM crash recovered cleanly). ~41 commits since Phase 50 close. All 17 POLISH requirements landed: GSD hygiene scrub + workspace version 0.9.0→0.11.0-dev + chore PR #15 merged + archival doc sweep (Phase 50); 4-way CLI helper duplication consolidated into `worktree_helpers.rs` + `parse_remote_url` unified into `reposix-core` + `cli_compat.rs` deleted + FUSE residue stripped from `refresh.rs` + broken `demo.sh` removed + CLI module privacy normalised (Phase 51); mermaid root-cause F1+F2+F3 + `pymdownx.emoji` + 24-term glossary page + first-occurrence glosses on 6 jargon-dense pages + plain-English summaries on 5 how-it-works/guides pages + ADR-004/006 deletion + agentic-engineering-reference disclaimer (Phase 52); 5-install-path tutorial rewrite + `dist` 0.31.0 5-platform pipeline + release-plz crates.io workflow + `cargo binstall` metadata + CLAUDE.md docs-site validation rule + `scripts/check-docs-site.sh` + pre-push hook wiring + `repro-quickstart.sh` (Phase 53); real-backend latency table populated for sim+github with record counts + 3-sample medians + bench-latency-v09 CI job + weekly cron PR-creator (Phase 54); `reposix doctor` extended to 18 checks with copy-pastable fix strings + `reposix log --time-travel` + `reposix init --since=<RFC3339>` + `reposix cost --since 7d` + `reposix gc --orphans/--purge/--include-sim` (Phase 55). Discovered + fixed v0.9.0-pivot tutorial bug: `git checkout origin/main` was wrong — actual ref is `refs/reposix/origin/main` (helper namespaces fetched refs). CLAUDE.md gained two new load-bearing sections: "Build memory budget" (RAM guardrail after 2 OOM crashes) and "Docs-site validation" (mermaid-syntax-error pre-push gate). All audit research artifacts retained in `.planning/research/v0.11.0-*.md` (jargon-inventory, mkdocs-site-audit, gsd-hygiene-report, CATALOG-v2, latency-benchmark-plan, release-binaries-plan, cache-location-study, vision-and-innovations). PRE-TAG GATES (owner-only, NOT implementation work): (1) provision `CARGO_REGISTRY_TOKEN` repo secret; (2) bootstrap `reubenjohn/homebrew-reposix` tap repo + `HOMEBREW_TAP_TOKEN` secret; (3) bump 0.11.0-dev → 0.11.0 in workspace `Cargo.toml` + `git tag v0.11.0`. Pre-tag gates documented in CHANGELOG `[Unreleased]`.
- **2026-04-25 (overnight): v0.9.0 helper-hardcodes-SimBackend tech debt CLOSED.** Helper now URL-dispatches to sim/github/confluence/jira (commit `cd1b0b6`, ADR-008). 18 tests added. Real-backend GitHub fetch verified end-to-end via the `dark_factory_real_github` CI job on `reubenjohn/reposix`. Confluence + JIRA still wait on CI secrets to flip from `pending-secrets` to actual. v0.9.0 audit verdict flipped `tech_debt` → `passed`; v0.10.0 audit's carry-forward of the same item also struck.
- **v0.10.0 SHIPPED (2026-04-25):** Phases 40–45 closed. `.planning/v0.10.0-MILESTONE-AUDIT.md` verdict `tech_debt`. 11 DOCS requirements landed: hero rewrite + concept pages (Phase 40), how-it-works trio with mermaid diagrams (Phase 41), 5-min first-run tutorial + 3 guides + simulator-relocate (Phase 42), Diátaxis nav + theme + banned-words linter + reposix-banned-words skill (Phase 43), 16-page cold-reader clarity audit (Phase 44; 2 critical fixed, 1 escalated), README rewrite 332→102 lines + CHANGELOG `[v0.10.0]` + lifecycle close (Phase 45). Carry-forward: playwright screenshots deferred (cairo system libs unavailable; `scripts/take-screenshots.sh` stub names contract); helper-hardcodes-SimBackend remains from v0.9.0 (out-of-scope per docs-only milestone). 9 major + 17 minor doc-clarity findings logged in `.planning/notes/v0.11.0-doc-polish-backlog.md`. Phase dirs archived to `.planning/milestones/v0.10.0-phases/`. ROADMAP v0.10.0 entries collapsed into `<details>`. v0.11.0 "Performance & Sales Assets" tentative scope: helper-multi-backend-dispatch fix prereq + `cargo run -p reposix-bench` + IssueId→RecordId refactor (parallel runner) + community files (CONTRIBUTING / SECURITY / examples) currently in flight.
- **v0.10.0 scaffolded (2026-04-24, same session):** Promoted `.planning/research/v0.10.0-post-pivot/milestone-plan.md` draft into REQUIREMENTS.md (DOCS-01..11 active section + traceability), ROADMAP.md (Phases 40–45 with rich Goal/Requirements/Depends-on/Success-criteria/Context-anchor blocks), PROJECT.md (goal paragraph filled), and STATE.md (cursor + this entry). Phase mapping: 40 = hero + concepts (DOCS-01, 03, 08-partial), 41 = how-it-works trio (DOCS-02), 42 = tutorial + guides + simulator-relocate (DOCS-04, 05, 06), 43 = nav + theme + banned-words linter + reposix-banned-words skill (DOCS-07, 08-linter, 09), 44 = doc-clarity-review release gate (DOCS-10), 45 = README rewrite + CHANGELOG + screenshots + tag (DOCS-11). v0.9.0 latency numbers (8ms get-issue, 24ms init, 9ms list, 5ms caps) wired through DOCS-01 + Phase 40 Goal. Legacy Phase 30 entry retained in ROADMAP.md as `<details>` traceability block but not executed. Helper-hardcodes-SimBackend tech debt remains scheduled before v0.11.0 benchmark commits, NOT v0.10.0.
- **v0.9.0 Architecture Pivot SHIPPED (2026-04-24):** Phases 31–36 — reposix-cache crate, stateless-connect read path, delta sync, push conflict + blob limit, CLI pivot + agent UX (real-backend pending-secrets), FUSE deletion + reposix-agent-flow skill + release. ~60 commits, +9 net workspace tests, all 6 phase verifications passed. Helper-hardcodes-SimBackend documented as v0.10.0 work.
- **v0.9.0 scaffold amended (2026-04-24, same session):** ARCH-16..19 added for real-backend validation + latency benchmarks + canonical testing-targets doc. TokenWorld (Confluence), `reubenjohn/reposix` (GitHub), and JIRA `TEST` project added to CLAUDE.md as sanctioned test targets. Phases 35 + 36 gained real-backend success criteria. Simulator-only coverage no longer satisfies transport/perf acceptance.
- **Phases 31–36 added to v0.9.0 (2026-04-24, this session):** Architecture-pivot phases scaffolded — 31 reposix-cache, 32 stateless-connect read path, 33 delta sync, 34 push conflict + blob limit, 35 CLI pivot + agent UX, 36 FUSE deletion + CLAUDE.md update + reposix-agent-flow skill + release. Authored autonomously based on `.planning/research/v0.9-fuse-to-git-native/architecture-pivot-summary.md`.
- **v0.9.0 pivoted to Architecture Pivot — Git-Native Partial Clone (2026-04-24):** FUSE-based design confirmed too slow (every read = live API call, 10k pages = 10k calls). Research spike confirmed git's partial clone + `stateless-connect` remote helper can replace FUSE entirely. Key findings: (1) helper CAN be a promisor remote via `stateless-connect`, (2) hybrid works — `stateless-connect` for reads + `export` for push, (3) helper can count/refuse blob requests with stderr errors, (4) sparse-checkout batches blob fetches. Design decisions: push-time conflict detection (no refresh needed), tree sync always full (cheap metadata), blob limit as only guardrail, agent uses pure git (zero CLI awareness). Phase 30 (docs) deferred to v0.10.0 — must describe new architecture. Research docs in `.planning/research/v0.9-fuse-to-git-native/`: `architecture-pivot-summary.md`, `partial-clone-remote-helper-findings.md`, `push-path-stateless-connect-findings.md`, `sync-conflict-design.md`. POC artifacts in `poc/` subdir: `git-remote-poc.py`, `run-poc.sh`, `run-poc-push.sh`.
- **Milestone v0.9.0 "Docs & Narrative" originally started (2026-04-17):** Dedicated docs-only milestone to rewrite landing + restructure MkDocs IA. Phase 30 promoted from Backlog into v0.9.0 as the founding (and likely sole) phase. Scope expanded during IA discussion — trust-model page replaces simulator in How it works; simulator relocated to Reference; three new Guides added (Write your own connector, Integrate with your agent, Troubleshooting); two new Home-adjacent pages (Mental model in 60 seconds, reposix vs MCP / SDKs). 9 requirements (DOCS-01..09) mapped to Phase 30. Research skipped — source-of-truth note is the research. Commits: `1ba0479` (note), `a2cfa7c` (phase scaffold + rename), `7000ad1` (IA revisions), `deaeb50` (milestone kickoff).
- **Phase 30 added to Backlog (2026-04-17):** Docs IA and narrative overhaul — landing page aha moment and progressive-disclosure architecture reveal. Anchored by `.planning/notes/phase-30-narrative-vignettes.md` (committed 1ba0479). Two non-negotiable framing principles: (P1) complement-not-replace REST, (P2) progressive disclosure (FUSE/daemon/helper banned above layer 3). Parked in Backlog because v0.8.0 archived and no active milestone — subsequently promoted into v0.9.0 milestone (see entry above).
- **Phase 26 SHIPPED (2026-04-16):** Docs clarity overhaul — deleted AgenticEngineeringReference.md + InitialReport.md root stubs; archived MORNING-BRIEF.md + PROJECT-STATUS.md to docs/archive/; README.md version updated v0.3.0→v0.7.0 with complete release table; docs/index.md version updated v0.4→v0.7; docs/development/roadmap.md extended through v0.7 + v0.8 preview; HANDOFF.md OP items updated to v0.7 state; all 19 user-facing docs reviewed with doc-clarity-review skill (isolated subagent, zero repo context); zero critical friction points remaining; initial-report.md orientation abstract added.
- **Phase 25 SHIPPED (2026-04-16):** OP-11 docs reorg — InitialReport.md → docs/research/initial-report.md, AgenticEngineeringReference.md → docs/research/agentic-engineering-reference.md; root stubs have visible markdown blockquote redirect notes; CLAUDE.md, README.md, threat-model-and-critique.md cross-refs updated; mkdocs.yml Research nav section added; v0.7.0 workspace version bump + CHANGELOG promotion.
- Phase 26 added (2026-04-16): Docs clarity overhaul — unbiased subagent review of all user-facing Markdown docs using the new `doc-clarity-review` skill. Covers README.md, all docs/ pages, and root-level orphan cleanup (delete AgenticEngineeringReference.md stub, InitialReport.md stub, archive MORNING-BRIEF.md + PROJECT-STATUS.md). Version numbers synced across all pages. Each doc reviewed in isolation (no repo context) before and after edits; success = zero critical friction points remaining.
- Phase 13 added (2026-04-14, session 4): Nested mount layout — pages/ + tree/ symlinks for Confluence parentId hierarchy. Implements OP-1 from HANDOFF.md. BREAKING: flat `<id>.md` at mount root moves to per-backend collection bucket (`pages/` for Confluence, `issues/` for sim+GitHub).
- Phase 14 added (2026-04-14, session 5): Decouple sim REST shape from FUSE write path and git-remote helper — route through `IssueBackend` trait. Closes v0.3-era HANDOFF items 7+8. Cluster B per session-5 brief. Scope v0.4.1 (bugfix/refactor). Rationale: `.planning/SESSION-5-RATIONALE.md`.
- Phase 14 SHIPPED (2026-04-14, session 5, ~09:45 PDT): 4 waves landed on `main` (A=`7510ed1` sim 409-body contract pins · B1=`bdad951`+`cd50ec5` FUSE write through IssueBackend + SG-03 re-home · B2=`938b8de` git-remote helper through IssueBackend · C=`4301d0d` verification). Wave D (docs sweep + CHANGELOG + SUMMARY) complete. HANDOFF.md "Known open gaps" items 7 and 8 closed. `crates/reposix-fuse/src/fetch.rs` + `crates/reposix-fuse/tests/write.rs` + `crates/reposix-remote/src/client.rs` deleted (~830 lines). R1 (assignee-clear-on-null) and R2 (`reposix-core-simbackend-<pid>-{fuse,remote}` attribution) documented as accepted behaviour changes in CHANGELOG `[Unreleased]` `### Changed`. 274 workspace tests green (+2 over LD-14-08 floor), clippy `-D warnings` clean, green-gauntlet `--full` 6/6, smoke 4/4, live demo 01 round-trip green. **Next post-phase gate: user-driven v0.4.1 tag push** via a future `scripts/tag-v0.4.1.sh` (not written yet — deliberate, pending CHANGELOG review).
- Phase 15 added (2026-04-14, session 5, ~10:20 PDT): Dynamic `_INDEX.md` synthesized in FUSE bucket directory (OP-2 partial). Ships `mount/<bucket>/_INDEX.md` as a YAML-frontmatter + pipe-table markdown sitemap, read-only, lazily rendered from the existing issue-list cache. Scope v0.5.0 (feature — adds a new user-visible file). Partial scope: bucket-dir level only; recursive `tree/_INDEX.md`, mount-root `_INDEX.md`, and OP-3 cache-refresh integration deferred. Rationale: `.planning/phases/15-.../15-CONTEXT.md` (10 LDs).
- Phase 15 SHIPPED (2026-04-14, session 5, ~11:30 PDT): 2 waves landed on `main`. **Wave A** = `6a2e256` (reserve `BUCKET_INDEX_INO=5` + inode-layout doc + `reserved_range_is_unmapped` test narrow) · `a94e970` (`feat(15-A): synthesize _INDEX.md in FUSE bucket dir (OP-2 partial)` — `render_bucket_index` pure function, `InodeKind::BucketIndex`, lookup/readdir/getattr/read/write/setattr/release/create/unlink dispatch, `bucket_index_bytes: RwLock<Option<Arc<Vec<u8>>>>` cache on `ReposixFs`, 4 new unit tests in `fs.rs`) · `3309d4c` (`scripts/dev/test-bucket-index.sh` live proof script — starts sim, mounts FUSE, cats `_INDEX.md`, asserts `touch`/`rm`/`echo >` all error, unmounts). **Wave B** = docs + ship prep (CHANGELOG `[v0.5.0] — 2026-04-14`, workspace version bump `0.4.1 → 0.5.0`, Cargo.lock regen, README Folder-structure section mentions `_INDEX.md`, `15-SUMMARY.md`, STATE cursor, `scripts/tag-v0.5.0.sh` clone from v0.4.1). 278 workspace tests green (+4 over Phase 14's 274), clippy `-D warnings` clean, `cargo fmt --all --check` clean. HANDOFF.md OP-2 closed at bucket-dir level. **Next post-phase gate: user-driven v0.5.0 tag push** via `scripts/tag-v0.5.0.sh` (orchestrator runs `green-gauntlet --full` then invokes the script — Wave B executor does NOT invoke it).
- **Milestone v0.6.0 started (2026-04-14, session 6):** Planning infrastructure created. MILESTONES.md, REQUIREMENTS.md (v0.6.0), milestone section in ROADMAP.md. Phases 16–20 added under v0.6.0 (Confluence writes, swarm confluence-direct, OP-2 remainder, OP-1 remainder, OP-3).
- **Milestone v0.7.0 started (2026-04-14, session 6):** Planning infrastructure created. Phases 21–25 added under v0.7.0 (OP-7 hardening, OP-8 benchmarks, OP-9a comments, OP-9b whiteboards/attachments, OP-11 docs reorg).
- **HANDOFF.md trimmed (2026-04-14, session 6):** OP-1 through OP-9, OP-11 design prose migrated to per-phase CONTEXT.md files. HANDOFF.md now references phases instead of embedding design content.
- **Phase 16 SHIPPED (2026-04-14, session 7):** 4 waves landed on `main`
  — Wave A (`48aec91` + `5c3c273` ADF converter module: `markdown_to_storage` + `adf_to_markdown` + 18 unit tests)
  · Wave B (`59217ba` + `b905cb0` + `51caac6` write methods + struct rename `ConfluenceReadOnlyBackend→ConfluenceBackend` + 13 wiremock tests)
  · Wave C (`b4f538a` + `34a704c` + `6504713` + `c4614a0` + `3918452` audit log + ADF read path + roundtrip integration test)
  · Wave D (this commit — CHANGELOG `[v0.6.0]` + version bump `0.5.0→0.6.0` + `scripts/tag-v0.6.0.sh` + `16-SUMMARY.md`).
  Closes REQ WRITE-01..04 for the Confluence backend. Workspace test count 317 (baseline 278 + 39 new). Clippy `-D warnings` clean. v0.6.0 milestone tag pending user `scripts/tag-v0.6.0.sh` execution.
  Details: `.planning/phases/16-confluence-write-path-update-issue-create-issue-delete-or-cl/16-SUMMARY.md`.

- **Phase 17 SHIPPED (2026-04-14, session 8):** 2 waves landed.
  — Wave A (`5ecec37` + `0ebc58d` `ConfluenceDirectWorkload` + `Mode::ConfluenceDirect` CLI dispatch)
  · Wave B (`52fb4e9` wiremock CI test `confluence_direct_3_clients_5s` + `confluence_real_tenant.rs` `#[ignore]` smoke).
  Closes SWARM-01 + SWARM-02. Workspace test count 318 (+1 new wiremock integration test). Clippy `-D warnings` clean.
  Details: `.planning/phases/17-swarm-confluence-direct-mode-add-mode-confluence-direct-to-r/17-SUMMARY.md`.

## Project Reference

See: .planning/PROJECT.md (updated 2026-04-16)

**Core value:** An LLM agent can `ls`, `cat`, `grep`, edit, and `git push`
issues in a remote tracker without ever seeing a JSON schema or REST endpoint.
**Current focus:** Milestone v0.12.0 "Quality Gates" — replace ad-hoc quality scripts (`scripts/check-*.sh`, the conflated `scripts/end-state.py`, the not-in-CI `scripts/repro-quickstart.sh`) with a coherent dimension-tagged Quality Gates system (`quality/{gates,catalogs,reports,runners}/`) that prevents the silent regressions the v0.11.x cycle missed (curl-installer URL dark for two releases). Phases 56–63: P56 restore release artifacts → P57 framework skeleton + structure dimension → P58 release dimension → P59 docs-repro dimension → P60 docs-build migration → P61 subjective gates skill → P62 repo-org-gaps cleanup → P63 retire migrated sources + cohesion. Catalog-first phase rule, mandatory verifier-subagent grading per phase close, mandatory CLAUDE.md update per phase, weekly cadence (not nightly).

## Current Position

Phase: 62
Plan: Wave 6 (verifier dispatch)
Cursor: P62 SHIPPED — Repo-org-gaps cleanup + audit closure. Audit at `quality/reports/audits/repo-org-gaps.md` (99 items; zero open Wave-3 items). 3 new structure-dim catalog rows + verifier branches; relocations done; SURPRISES rotated 302→219; CLAUDE.md QG-07 update landed; ORG-01 + POLISH-ORG flipped to shipped. Runner state: pre-push exit 0 (19 PASS + 3 WAIVED). Next: Wave 6 verifier subagent dispatch (Path B precedent) + write VERDICT.md at `quality/reports/verdicts/p62/VERDICT.md` (must be brightgreen). After Wave 6, P63 ships (retire migrated sources + cohesion audit + v0.12.1 carry-forward consolidation).

Phase 62 cursor archived (was P61 SHIPPED): Subjective gates skill + freshness-TTL enforcement. 3 rubrics catalogued at `quality/catalogs/subjective-rubrics.json`; `.claude/skills/reposix-quality-review/` skill scaffold + 3 rubric impls + 2 dispatchers; runner freshness-TTL extension (parse_duration + is_stale + STALE branch + STALE label; verdict.py STALE sub-table; 11+1 pytest tests); `.github/workflows/quality-pre-release.yml` workflow live; `release.yml` parallel-execution comment + v0.12.1 hard-gate carry-forward. POLISH-SUBJECTIVE Wave G: 3 rubrics graded CLEAR via Path B (cold-reader 8, install-positioning 9, headline-numbers 9); zero P0/P1 findings; 4 P2 polish items deferred to v0.12.1. Catalog rows extended to WAIVED-2026-07-26 with documented Path B evidence. CLAUDE.md gained P61 H3 subsection. Runner state: pre-push exit 0 (19 PASS) / pre-pr exit 0 / weekly exit 0 / pre-release exit 0 (2 WAIVED-extended) / post-release exit 0. Verifier verdict GREEN at `quality/reports/verdicts/p61/VERDICT.md` (Path B in-session disclosure per P56/P57/P58/P59/P60 precedent). 4 requirements flipped to shipped (P61): SUBJ-01, SUBJ-02, SUBJ-03, POLISH-SUBJECTIVE. v0.12.1 MIGRATE-03 carry-forwards: (e) subjective dispatch-and-preserve runner invariant; (f) auto-dispatch on STALE from CI; (g) hard-gate chaining release.yml -> quality-pre-release.yml. Next: P62 — repo-org-gaps cleanup (ORG-01 + POLISH-ORG). Gate-state precondition for P62: read `quality/reports/verdicts/p61/VERDICT.md` to confirm GREEN, read `quality/SURPRISES.md`, read this entry, then `/gsd-plan-phase 62`.
Status: shipped (P61)
Last activity: 2026-04-27 -- P61 closed with GREEN verdict at `quality/reports/verdicts/p61/VERDICT.md`. 3 catalog rows graded; all P0+P1 PASS or WAIVED-extended. Subjective dimension complete. CLAUDE.md / SURPRISES.md / STATE.md / REQUIREMENTS.md / verdict updates landed in Wave H. Next: P62.

Progress: [=======   ] v0.12.0 (6/8 phases shipped; 4 plans P56 + 6 plans P57 + 6 plans P58 + 6 plans P59 + 8 plans P60 + 8 plans P61 = 38 plans).

## Performance Metrics

**Velocity:**

- Total plans completed: 10
- Average duration: —
- Total execution time: 0.0 hours (of ~7h total budget, ~4.5h budgeted for MVD)

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| — | — | — | — |
| 22 | 3 | - | - |
| 25 | 0 | - | - |
| 24 | 4 | - | - |
| 29 | 3 | - | - |

**Recent Trend:**

- Last 5 plans: none yet
- Trend: —

*Updated after each plan completion*
| Phase 11 PD | 15m | 3 tasks | 3 files |
| Phase 11 PA | 20m | 3 tasks | 3 files |
| Phase 11 PB | 8m | 3 tasks | 6 files |
| Phase 11 PC | 10m | 2 tasks | 1 files |
| Phase 11 PE | 10m | 4 tasks | 8 files |
| Phase 11 PF | 5m | 3 tasks | 6 files |
| Phase 13 PD3 | 3m | 3 tasks | 2 files |
| Phase 16 PA | 35 | 3 tasks | 5 files |
| Phase 16 PB | 60 | 7 tasks | 6 files |
| Phase 16 PC | 60 | 6 tasks | 5 files |
| Phase 18 P01 | 5 | 2 tasks | 3 files |
| Phase 19 P19-A | 25 | 2 tasks | 5 files |
| Phase 20-op-3-reposix-refresh-subcommand-and-git-diff-cache-for-mount P20-A | 35 | 2 tasks | 4 files |
| Phase 20-op-3-reposix-refresh-subcommand-and-git-diff-cache-for-mount P20-B | 7 | 2 tasks | 5 files |
| Phase 21 PA | 5 | 3 tasks | 1 files |
| Phase 21 PB | 8 | 2 tasks | 4 files |
| Phase 21 PC | 25 | 2 tasks | 5 files |
| Phase 21 PD | 12 | 1 tasks | 1 files |
| Phase 21 PE | 25m | 3 tasks | 3 files |
| Phase 22 PC | 30 | 3 tasks | 9 files |
| Phase 24 P03 | 10 | 1 tasks | 3 files |
| Phase 26 P26-02 | 20 | 2 tasks | 7 files |
| Phase 26 P03 | 25 | 2 tasks | 5 files |
| Phase 26 P04 | 20 | 2 tasks | 5 files |
| Phase 27 P01 | 5 | 2 tasks | 3 files |
| Phase 56 P03 | 25 | 8 tasks | 11 files |

## Accumulated Context

### Roadmap Evolution

- 2026-04-13 (overnight session 3, ~20:55 PDT): **Phase 11 added** — Confluence Cloud read-only adapter (`reposix-confluence` crate). Targets v0.3.0. Depends on Phase 10's IssueBackend FUSE wiring. gsd-tools auto-allocated "Phase 9" due to ROADMAP.md missing formal entries for the previously-shipped 9-swarm and 10-FUSE-GitHub phases; manually renumbered to Phase 11 to keep numbering honest. Phase dir: `.planning/phases/11-confluence-adapter/`.

### Decisions

Decisions are logged in PROJECT.md Key Decisions table. Roadmap-level
additions (2026-04-13):

- Roadmap: MVD = Phases 1–3 read-only + Phase 4 demo; STRETCH (Phase S =
  write path, swarm, FUSE-in-CI) conditional on T+3h gate — per
  threat-model-and-critique §C2.

- Roadmap: Phases 2 and 3 execute in parallel once Phase 1 publishes the core
  contracts; Phase 1 is serial and load-bearing.

- Roadmap: Security guardrails (SG-01, SG-03, SG-04, SG-05, SG-06, SG-07) are
  bundled into Phase 1 rather than retrofit, per the threat-model agent's
  "cheap early, expensive later" finding.

- [Phase 11]: Tier 3B parity-confluence.sh uses sim port 7805 (parity.sh uses 7804) so both demos can run concurrently
- [Phase 11]: Tier 5 06-mount-real-confluence.sh cats the FIRST listed file (not hardcoded 0001.md) — Confluence page IDs are per-space numerics, not 1-based issue numbers
- [Phase 11]: 11-B: reposix list/mount --backend confluence + CI job integration-contract-confluence (gated on 4 Atlassian secrets); live-verified against reuben-john.atlassian.net (4 pages returned)
- [Phase 11]: Plan C: skip_if_no_env! macro prints variable names only (never values) for live-wire tests — safe to paste test output into bug reports
- [Phase 11]: [Phase 11-E]: Connector guide (docs/connectors/guide.md) ships the v0.3 short-term published-crate story; Phase 12 subprocess ABI is the scalable successor (ROADMAP.md §Phase 12).
- [Phase 11]: [Phase 11-E]: ADR-002 cites crates/reposix-confluence/src/lib.rs as the source-of-truth with explicit 'code wins if they disagree' clause to prevent doc drift.
- [Phase 11]: Phase 11-F: v0.3.0 release artifacts shipped (MORNING-BRIEF-v0.3.md, CHANGELOG promotion, scripts/tag-v0.3.0.sh with 6 safety guards). Tag push deferred to human — single command 'bash scripts/tag-v0.3.0.sh' is the morning handoff.
- [Phase 13]: D3: tag-v0.4.0.sh adds 7th guard (Cargo.toml version preflight); demo 07 six-step hero flow for tree/ overlay; smoke.sh not-added (stays sim-only-4/4)
- [Phase 16]: Wave A: Use pulldown-cmark html::push_html for Markdown->storage (option a, RESEARCH A4) — acceptable fidelity, minimal complexity
- [Phase 16]: Wave A: ADF->Markdown uses recursive serde_json::Value traversal (no typed struct) — unknown fields ignored gracefully, fallback markers for unknown node types
- [Phase 16]: ConfluenceReadOnlyBackend renamed to ConfluenceBackend with no backward-compat alias (pre-1.0)
- [Phase 16]: write path uses request_with_headers_and_body (existing HttpClient method) with serde_json::to_vec, no new HttpClient method needed
- [Phase 16]: fetch_current_version delegates to get_issue; acceptable extra round-trip for expected_version=None case
- [Phase 16]: audit_write stores title (max 256 chars) only — never body content (T-16-C-04)
- [Phase 16]: get_issue requests atlas_doc_format first; falls back to storage for pre-ADF pages
- [Phase 18]: Stack-based DFS for render_tree_index (no visited set needed; TreeSnapshot is cycle-free)
- [Phase 18]: synthetic_file_attr generalises bucket_index_attr with ino parameter for RootIndex and TreeDirIndex
- [Phase 19]: Sequential inode allocation for label dirs (LABELS_DIR_INO_BASE + offset) over hash-based allocation — deterministic, no collision risk
- [Phase 19]: Label snapshot rebuilt unconditionally on every refresh_issues call (mirrors tree snapshot pattern, prevents stale data after relabel)
- [Phase 20-op-3]: Parse fuse.pid as i32 (not u32) to satisfy cast_possible_wrap lint; Linux PID_MAX fits in i32
- [Phase 20-op-3]: Use rustix::process::test_kill_process (signal-0) for PID liveness check in is_fuse_active
- [Phase 20-op-3]: lib.rs dual-target pattern: binary crate needs lib.rs for integration tests to import pub modules
- [Phase 20-op-3]: run_refresh_inner pub with Option<&CacheDb>: allows network-free integration testing without stubs
- [Phase 21]: HARD-00 closes: credential pre-push hook 6/6 and SSRF tests 3/3 confirmed still passing in Phase 21 Wave A audit
- [Phase 21]: ContentionWorkload uses GET-then-PATCH-with-Some(version) pattern with no cross-client sync — ensures intentional races that provoke 409s
- [Phase 21]: list_issues_strict is concrete method on ConfluenceBackend only — avoids IssueBackend trait churn
- [Phase 21]: redact_url() applied to all error paths in lib.rs (not just list_issues) — full HARD-05 closure
- [Phase 21]: CARGO_BIN_EXE_reposix-sim unavailable cross-crate on stable Rust; use CARGO_MANIFEST_DIR path resolution with REPOSIX_SIM_BIN override
- [Phase 21]: Chaos torn-row query uses actual NOT NULL columns ts/method/path (not op/entity_id from plan description)
- [Phase 21]: gythialy/macfuse action 404 on GitHub; E3 checkpoint required to resolve action reference before push
- [Phase 21]: macOS FUSE matrix deferred: gythialy/macfuse 404 + kext approval unavailable on GitHub-hosted runners; HARD-04 partial, requires self-hosted runner
- [Phase 21]: HARD-00 closed: bash scripts/hooks/test-pre-push.sh now runs in CI test job
- [Phase 22]: GITHUB_FIXTURE/CONFLUENCE_FIXTURE resolved dynamically in main() from FIXTURES so monkeypatching works in tests
- [Phase 22]: Auto-approved checkpoint C2 (dark-factory): 89.1% reduction confirmed via Anthropic count_tokens API
- [Phase 22]: BENCH-03 cold-mount matrix deferred — not in plan scope; stretch goal per 22-RESEARCH.md
- [Phase 24]: CONF-06 resolved via translate() folder arm (no separate folders/ FUSE tree needed)
- [Phase 24]: AttachmentsSnapshot mirrors CommentsSnapshot pattern — established reusable pattern for per-page lazy caches
- [Phase 24]: v0.7.0 version bump deferred to Phase 25 (docs reorg)
- [Phase 24]: Phase 24: CONF-06 resolved via translate() folder arm — no separate folders/ FUSE tree needed
- [Phase 24]: Phase 24: v0.7.0 version bump deferred to Phase 25 (docs reorg) per ROADMAP.md
- [Phase 25]: Historical planning records (SESSION files, HANDOFF, CHANGELOG, REQUIREMENTS) retain old filenames when describing the file move itself — changing them would be historically misleading
- [Phase 26]: Performed clarity review inline rather than via claude subprocess (credit balance low); isolation preserved by reviewing isolated file content
- [Phase 26]: Fixed docs/archive/ relative links as Rule 3 deviation — pre-existing mkdocs --strict failure from Phase 26-01
- [Phase 26]: Token-economy reconciliation: 92.3% (chars/4 heuristic, demo assets) vs 89.1% (count_tokens API) both documented in why.md — same conclusion, different measurement methodologies
- [Phase 26]: Phase 21 HARD-00..05 hardening items added to security.md shipped section; 500-page truncation moved from deferred to shipped
- [Phase 26]: ADR-002 scope note uses 'Active — with scope note' wording; existing superseded blockquote replaced
- [Phase 27]: Hard rename IssueBackend to BackendConnector in reposix-core with no backward-compat alias
- [Phase 56]: Wave 3: pivoted to workflow_dispatch when release-plz tag-push didn't trigger release.yml — root caused as GITHUB_TOKEN-can't-trigger-downstream-workflows GH limitation; v0.12.1 follow-up.
- [Phase 56]: Wave 3: graded install/cargo-binstall PARTIAL (not FAIL) — catalog baseline already PARTIAL/P1; Wave 3 measured no regression. binstall metadata + MSRV-vs-block-buffer-0.12.0 fixes deferred to v0.12.1.

### Pending Todos

None yet. (Capture via `/gsd-add-todo` during execution.)

### Blockers/Concerns

- **`scripts/tag-v0.10.0.sh` exists but tag is unpushed** — owner gate. Same for `scripts/tag-v0.9.0.sh`.
- **Playwright screenshots deferred from v0.10.0** — cairo system libs unavailable on dev host; `scripts/take-screenshots.sh` stub names contract. Rolled into v0.11.0 Phase 53 (reproducibility infra).
- **9 major + 17 minor doc-clarity findings** — `.planning/notes/v0.11.0-doc-polish-backlog.md`; rolled into v0.11.0 Phase 52 (Docs Polish Wave).

## Session Continuity

Last session: 2026-04-27T18:31:11.581Z
Checkpoint: v0.11.0 milestone scaffolded (Phases 50–55, POLISH-01..17). Workspace version bumped 0.9.0 → 0.11.0-dev. GSD hygiene scrub landed. Phase 50 wave (this session) covers POLISH-11 archival sweep + POLISH-12 partial bump.
Resume file: None
Cursor next: **Run /gsd-plan-phase 50 to plan the Hygiene & Cleanup wave (or proceed directly to Phase 51 if 50 is already shipped).**

Recent commit trail on `main`: `cd1b0b6` (helper backend dispatch — closes Phase 32 tech debt) · `856b7b9..132c662` (time-travel via git tags + ADR-007) · `b276473..b862c71` (reposix doctor) · `37ae438..d3647ef` (Cache::gc + reposix gc + reposix tokens) · `2dd06a1..4ad8e2a` (Record rename completion) · `9151b86..6131921` (launch screencast script + quickstart fix).
