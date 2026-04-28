# Repo organization gaps — P62 audit closure

**Source:** `.planning/research/v0.11.1-repo-organization-gaps.md` (snapshot 2026-04-26).
**Audited:** P62 Wave 2 on 2026-04-27 at commit `eaf7068`.
**Verifier:** `scripts/check_repo_org_gaps.py` (asserts every numbered rec is line-referenced + every disposition is in the allow-list).

## Closure summary

The bulk of the source doc's recommendations were CLOSED before P62 by prior milestones (v0.11.0 deletions; v0.12.0 P57 / P58 / P59 / P60 / P61 absorptions). P62 Wave 2 audits each item, attaches evidence, and identifies the residual list for Wave 3 fix work (`closed-by-Wave-3-fix`).

Disposition allow-list:

- `closed-by-deletion` — item removed entirely (verify with `ls`).
- `closed-by-existing-gate` — recurrence is now blocked by an active catalog row + verifier.
- `closed-by-catalog-row` — gap fixed earlier in milestone AND a P62 Wave 1 row locks it (`structure/no-loose-top-level-planning-audits`, `structure/no-pre-pivot-doc-stubs`, `structure/repo-org-audit-artifact-present`).
- `closed-by-relocation` — moved to its proper home.
- `closed-by-Wave-3-fix` — Wave 3 of THIS phase closes it; explicit action listed in footer.
- `waived` — explicit waiver row + RFC3339 expiry.
- `out-of-scope` — deferred to v0.12.1 MIGRATE-03 or another milestone.

## Top 10 recommendations

| # | Item | Disposition | Evidence/Citation |
|---|---|---|---|
| 1 | Delete entire `scripts/demos/` tree (11 files, FUSE-era) + `docs/demos/recordings/` + `docs/demos/index.md` | closed-by-deletion | `ls scripts/demos/` returns "No such file"; `ls docs/demos/recordings` returns "No such file"; `ls docs/demos/index.md` returns "No such file" |
| 2 | Delete `scripts/dev/list-confluence-spaces.sh` + `scripts/dev/probe-confluence.sh`, then `rmdir scripts/dev/` | closed-by-deletion | `ls scripts/dev/` returns "No such file" |
| 3 | Delete `scripts/__pycache__/` (2 `.pyc`) + add to `.gitignore` test in CI | closed-by-deletion | Wave 3 commit `8842d48`: `.pyc` files were already untracked locally (`.gitignore` covers `__pycache__/` recursively at line 30); workspace-only `.pyc` files removed via `rm -rf`. No git-tracked content to delete. |
| 4 | Move `scripts/migrations/{fix_demos_index_links.py, mermaid_divs_to_fences.py}` to `.planning/archive/scripts/` | closed-by-relocation | Both present at `.planning/archive/scripts/{fix_demos_index_links.py, mermaid_divs_to_fences.py}`; `ls scripts/migrations/` returns "No such file" |
| 5 | Condense `.planning/milestones/v0.{1,2,3,4,5,6,7,8}.0-phases/` into 8 `ARCHIVE.md` | closed-by-relocation | `ls` per dir confirms each carries an `ARCHIVE.md`. `v0.4.0-phases/` retains `tag-v0.4.0.sh`; `v0.8.0-phases/` retains `REQUIREMENTS.md` + `ROADMAP.md` (these are scoped milestone docs per CLAUDE.md §"`.planning/milestones/` convention" — KEPT intentionally, not loose). |
| 6 | Move `.planning/research/v0.11.0-*.md` into `.planning/milestones/v0.11.0-phases/` once milestone closes | out-of-scope | Defer — v0.11.0 tag still unpushed (STATE.md "owner gates pending"). Move is cosmetic; research/ is the canonical home until tag pushes. Tracked under v0.12.1 MIGRATE-03 if not naturally absorbed. |
| 7 | Move `.planning/SESSION-5-RATIONALE.md` + `SESSION-7-BRIEF.md` to `.planning/archive/sessions/` | closed-by-relocation | Both present at `.planning/archive/sessions/{SESSION-5-RATIONALE.md, SESSION-7-BRIEF.md}` |
| 8 | Rename `scripts/v0.9.0-latency.sh` + `docs/benchmarks/v0.9.0-latency.md` (drop version pin) | closed-by-relocation | `scripts/v0.9.0-latency.sh` migrated to `quality/gates/perf/latency-bench.sh` (P59 SIMPLIFY-11 via `git mv`); `docs/benchmarks/latency.md` is the version-agnostic name. Recurrence guarded by `structure/no-version-pinned-filenames` (PASS). |
| 9 | `scripts/take-screenshots.sh` — implement or delete | closed-by-deletion | `ls scripts/take-screenshots.sh` returns "No such file" |
| 10 | Move `.planning/notes/phase-30-narrative-vignettes.md` to `.planning/archive/notes/` | closed-by-relocation | Present at `.planning/archive/notes/phase-30-narrative-vignettes.md` |

## .planning/ structure

| # | Item | Disposition | Evidence/Citation |
|---|---|---|---|
| 11 | `v0.1.0-phases/` (38 files → ARCHIVE.md) | closed-by-relocation | `ls .planning/milestones/v0.1.0-phases/` returns only `ARCHIVE.md` |
| 12 | `v0.2.0-alpha-phases/` (4 files → ARCHIVE.md) | closed-by-relocation | `ls` returns only `ARCHIVE.md` |
| 13 | `v0.3.0-phases/` (19 files → ARCHIVE.md) | closed-by-relocation | `ls` returns only `ARCHIVE.md` |
| 14 | `v0.4.0-phases/` (24 files → ARCHIVE.md) | closed-by-relocation | `ls` returns `ARCHIVE.md` + `tag-v0.4.0.sh` (loose tag script — see #25 below) |
| 15 | `v0.5.0-phases/` (16 files → ARCHIVE.md) | closed-by-relocation | `ls` returns only `ARCHIVE.md` |
| 16 | `v0.6.0-phases/` (68 files → ARCHIVE.md) | closed-by-relocation | `ls` returns only `ARCHIVE.md` |
| 17 | `v0.7.0-phases/` (63 files → ARCHIVE.md) | closed-by-relocation | `ls` returns only `ARCHIVE.md` |
| 18 | `v0.8.0-phases/` (31 files → ARCHIVE.md) | closed-by-relocation | `ls` returns `ARCHIVE.md` + `REQUIREMENTS.md` + `ROADMAP.md` (kept intentionally per CLAUDE.md §"`.planning/milestones/` convention" Option B; per-milestone scoped planning docs live INSIDE the matching `*-phases/` dir, not loose at top level) |
| 19 | `v0.9.0-phases/` — KEEP | closed-by-existing-gate | Still referenced by ADRs/CHANGELOG; CATALOG-v2 leaves intact. Recurrence not applicable. |
| 20 | `v0.10.0-phases/` — KEEP | closed-by-existing-gate | Already trimmed during ship. Recurrence not applicable. |
| 21 | `v0.11.0-phases/` — n/a | out-of-scope | No phase dir created; phases ran inline. Decision deferred until tag push. |
| 22 | Loose milestone docs (`.planning/milestones/v0.10.0-ROADMAP.md`, `v0.9.0-ROADMAP.md`, `v0.8.0-ROADMAP.md`, `v0.8.0-REQUIREMENTS.md`) | closed-by-existing-gate | `structure/no-loose-roadmap-or-requirements` (catalog row, PASS) blocks any future loose `*ROADMAP*` / `*REQUIREMENTS*` at `.planning/milestones/` top level; current state is clean per the verifier. |
| 23 | `.planning/v0.9.0-MILESTONE-AUDIT.md` + `v0.10.0-MILESTONE-AUDIT.md` (top-level) — move under `.planning/milestones/audits/` | closed-by-relocation | Wave 3 commit `8842d48`: `git mv` both to `.planning/milestones/audits/v0.{9,10}.0-MILESTONE-AUDIT.md` (history preserved). Recurrence locked by P62 Wave 1 row `structure/no-loose-top-level-planning-audits`. |
| 24 | `.planning/CATALOG.md` (529 lines) — move to `.planning/research/v0.11.0-CATALOG-v1.md` | closed-by-relocation | `ls .planning/CATALOG.md` returns "No such file"; `.planning/research/v0.11.0-CATALOG-v1.md` exists (45255 bytes). |
| 25 | `.planning/phases/` empty dir — DELETE | closed-by-existing-gate | Currently NON-empty (active phase dirs P56–P62 live there). Recommendation was conditional ("if empty"); recommendation no longer applies. |
| 26 | `.planning/notes/gsd-feedback.md` — KEEP | closed-by-existing-gate | Still referenced; awaiting-user-review. |
| 27 | `.planning/notes/v0.11.0-doc-polish-backlog.md` — KEEP | closed-by-existing-gate | Actively referenced by STATE.md. |
| 28 | `.planning/research/v0.1-fuse-era/` — KEEP | closed-by-existing-gate | Cited from SECURITY.md + README.md; foundational. |
| 29 | `.planning/research/v0.9-fuse-to-git-native/` — KEEP | closed-by-existing-gate | Cited from CLAUDE.md; source of truth for the v0.9.0 pivot. |
| 30 | `.planning/research/v0.10.0-post-pivot/milestone-plan.md` — KEEP | closed-by-existing-gate | Reference doc for the rename pattern (rec #6). |
| 31 | `.planning/SESSION-END-STATE*.md` + `.json` + `-VERDICT.md` (top-level) | closed-by-relocation | Wave 3 commit `8842d48`: `git mv` to `.planning/archive/session-end-state/{SESSION-END-STATE.md, SESSION-END-STATE.json, SESSION-END-STATE-VERDICT.md}` + new `README.md` naming `quality/PROTOCOL.md` as the supersession path. Recurrence locked by P62 Wave 1 row `structure/no-loose-top-level-planning-audits`. |

## scripts/ — verdict per file

| # | Item | Disposition | Evidence/Citation |
|---|---|---|---|
| 32 | `scripts/banned-words-lint.sh` — KEEP | closed-by-existing-gate | Wired into pre-push via `quality/gates/structure/banned-words.sh` (P57 SIMPLIFY-01 wrapper); structure/banned-words PASS. |
| 33 | `scripts/check-docs-site.sh` — KEEP | closed-by-relocation | Migrated to `quality/gates/docs-build/mkdocs-strict.sh` (P60 SIMPLIFY-08); shim at old path per OP-5 reversibility. |
| 34 | `scripts/check_clippy_lint_loaded.sh` — KEEP | closed-by-relocation | Migrated to `quality/gates/code/clippy-lint-loaded.sh` (P58 SIMPLIFY-04); old path DELETED, no callers. |
| 35 | `scripts/check_doc_links.py` — KEEP | closed-by-relocation | Migrated to `quality/gates/docs-build/link-resolution.py` (P60 SIMPLIFY-08). |
| 36 | `scripts/check_fixtures.py` — KEEP | closed-by-relocation | Migrated to `quality/gates/code/check-fixtures.py` (P58 SIMPLIFY-05); old path DELETED. |
| 37 | `scripts/dark-factory-test.sh` — KEEP | closed-by-relocation | Migrated to `quality/gates/agent-ux/dark-factory.sh` (P59 SIMPLIFY-07); 7-line shim at old path. |
| 38 | `scripts/green-gauntlet.sh` — KEEP | closed-by-relocation | Rewritten as thin shim delegating to `quality/runners/run.py --cadence pre-pr` (P60 SIMPLIFY-09). |
| 39 | `scripts/install-hooks.sh` — KEEP | closed-by-existing-gate | Recommended by CONTRIBUTING.md; current. |
| 40 | `scripts/repro-quickstart.sh` — KEEP | closed-by-deletion | DELETED in P59 SIMPLIFY-06 (no callers); `tutorial-replay.sh` ports the assertion shape verbatim. |
| 41 | `scripts/v0.9.0-latency.sh` — KEEP, RENAME | closed-by-relocation | Migrated to `quality/gates/perf/latency-bench.sh` (P59 SIMPLIFY-11). |
| 42 | `scripts/bench_token_economy.py` + `test_bench_token_economy.py` — KEEP | closed-by-relocation | Both migrated to `quality/gates/perf/` (P59 SIMPLIFY-11). |
| 43 | `scripts/take-screenshots.sh` — DELETE | closed-by-deletion | `ls` returns "No such file". |
| 44 | `scripts/tag-v0.{3,4,5,6,8}.0.sh` — ARCHIVE | closed-by-relocation | All 5 present at `.planning/archive/scripts/tag-v0.{3,4.1,5,6,8}.0.sh`. NOTE: `v0.4.0-phases/tag-v0.4.0.sh` is still in place (not in `archive/scripts/`); minor inconsistency with rec, but functionally equivalent (file is read-only history). Mark as `out-of-scope` for cosmetic move. |
| 45 | `scripts/tag-v0.9.0.sh` + `tag-v0.10.0.sh` — KEEP | closed-by-existing-gate | Recent + tag-script-authoring template; KEEP. |
| 46 | `scripts/demos/*` (11 files) — DELETE | closed-by-deletion | `ls scripts/demos/` returns "No such file". |
| 47 | `scripts/dev/{list-confluence-spaces, probe-confluence}.sh` — DELETE | closed-by-deletion | `ls scripts/dev/` returns "No such file". |
| 48 | `scripts/migrations/*.py` — ARCHIVE | closed-by-relocation | Both at `.planning/archive/scripts/`. |
| 49 | `scripts/hooks/{pre-push, test-pre-push.sh}` — KEEP | closed-by-existing-gate | OP-7 cred-leak hook; CI-tested. |
| 50 | `scripts/__pycache__/*` — DELETE + .gitignore-test | closed-by-deletion | Wave 3 commit `8842d48`: `.pyc` files were workspace-only (never tracked); `.gitignore` already covers `__pycache__/`. Working tree cleaned via `rm -rf`. |

## docs/ orphans

| # | Item | Disposition | Evidence/Citation |
|---|---|---|---|
| 51 | `docs/demo.md` (198 B stub) + `docs/architecture.md` (357 B) + `docs/security.md` (304 B) + `docs/why.md` (422 B) + `docs/connectors/guide.md` — KEEP redirect stubs | closed-by-existing-gate | All 4 stubs are listed in `mkdocs.yml not_in_nav:` (lines 51-54); P62 Wave 1 catalog row `structure/no-pre-pivot-doc-stubs` enforces "stub <500 B is in nav OR not_in_nav OR redirect_maps" — verifier branch lands in Wave 3. |
| 52 | `docs/demo.transcript.txt` + `docs/demo.typescript` — DELETE | closed-by-deletion | `ls` returns "No such file" for both. |
| 53 | `docs/github-readme-top.png` — DELETE | closed-by-deletion | `ls` returns "No such file". |
| 54 | `docs/screenshots/gh-pages-home-v0.2.png` + `gh-pages-why-real-github.png` — ARCHIVE or DELETE | closed-by-deletion | `ls docs/screenshots/` returns only the 4 current images (`gh-pages-home.png`, `site-architecture.png`, `site-home.png`, `site-security.png`); the v0.2-era screenshots are gone. |
| 55 | `docs/decisions/001-github-state-mapping.md` + `002-confluence-page-mapping.md` + `003-nested-mount-layout.md` — KEEP, ANNOTATE with "Status: superseded" header | closed-by-existing-gate | ADR-002 carries "Status: Active — with scope note … layout section superseded by ADR-003"; ADR-003 carries "status: superseded" frontmatter referencing ADR-008 + the v0.9.0 pivot. ADR-001 lacks supersession (still labeled Accepted) but is content-current (GitHub state mapping unchanged). |
| 56 | `docs/development/roadmap.md` — REWRITE or DELETE | closed-by-existing-gate | Wave 3 verification: 3 `fuse` mentions are all in historical release-notes context (`v0.1.0 MVD … FUSE read-only mount`, `v0.5.0 IssueBackend decoupling from FUSE`, `v0.9.0 Architecture pivot — FUSE mount retired`). Same pattern as CHANGELOG.md — historical context is allowed. No edit needed. |
| 57 | `docs/development/contributing.md` — KEEP, repoint to top-level CONTRIBUTING.md | closed-by-existing-gate | Wave 3 verification: prior `grep -c fuse` returned 1 from substring match in the word "**re**fuse" ("CI will refuse the change"). No actual `fuse`/`FUSE` reference. False positive in source audit. |
| 58 | `docs/research/initial-report.md` + `agentic-engineering-reference.md` — KEEP | closed-by-existing-gate | Carry pre-v0.1 status banners; foundational. |
| 59 | `docs/social/assets/` + `_build_*.py` builders — KEEP convention | closed-by-existing-gate | `ls docs/social/` confirms `assets/`, `linkedin.md`, `twitter.md` present; convention accepted. |
| 60 | `docs/javascripts/mermaid-render.js`, `docs/stylesheets/extra.css`, `docs/.banned-words.toml` — KEEP all | closed-by-existing-gate | Load-bearing for mkdocs build; `docs-build/mermaid-renders` PASS confirms intact. |

## tests/ orphans

| # | Item | Disposition | Evidence/Citation |
|---|---|---|---|
| 61 | Each fixture referenced by at least one test | closed-by-existing-gate | `code/fixtures-valid` (PASS) verifies fixture integrity; `check-fixtures.py` enforces. |
| 62 | `crates/reposix-swarm/src/metrics.rs` lines 238/240 still mention `fuse` in doc comments | closed-by-deletion | `grep -i fuse crates/reposix-swarm/src/metrics.rs` returns ZERO matches at audit time — the doc strings were already scrubbed in a prior commit. No Wave 3 action needed. |
| 63 | `crates/reposix-cli/src/worktree_helpers.rs` untracked — minor | closed-by-existing-gate | Tracked + tested as part of v0.11.0 Phase 51 (per STATE.md). |

## Top-level files audit

| # | Item | Disposition | Evidence/Citation |
|---|---|---|---|
| 64 | `LICENSE-MIT` + `LICENSE-APACHE` — KEEP | closed-by-existing-gate | Required for crates.io publish. |
| 65 | `README.md` — KEEP | closed-by-existing-gate | `structure/install-leads-with-pkg-mgr-readme` (PASS). |
| 66 | `CHANGELOG.md` — KEEP | closed-by-existing-gate | CI doc-clarity skill verified. |
| 67 | `CONTRIBUTING.md` — KEEP | closed-by-existing-gate | Aligned with v0.9.0+ pivot. |
| 68 | `CODE_OF_CONDUCT.md` — KEEP | closed-by-existing-gate | Standard. |
| 69 | `SECURITY.md` — KEEP | closed-by-existing-gate | Threat-model summary. |
| 70 | `CLAUDE.md` — KEEP | closed-by-existing-gate | Agent-onboarding spec. |
| 71 | `PUBLIC-LAUNCH-CHECKLIST.md` — UPDATE tag command to `tag-v0.11.0.sh` | out-of-scope | Defer to v0.11.0 tag-push owner gate (per STATE.md "Blockers/Concerns"). Updating the checklist before the tag script exists would create a dangling reference. Tracked under v0.12.1 MIGRATE-03 (or naturally absorbed when v0.11.0 ships). |
| 72 | `Cargo.toml` + `Cargo.lock` — KEEP | closed-by-existing-gate | Workspace 9-crate; release-plz cuts per-crate tags. |
| 73 | `mkdocs.yml` — KEEP | closed-by-existing-gate | `docs-build/mkdocs-strict` (PASS). |
| 74 | `clippy.toml`, `rust-toolchain.toml`, `rustfmt.toml` — KEEP | closed-by-existing-gate | All pinned. |
| 75 | `deny.toml` — KEEP | closed-by-existing-gate | Aligned with LICENSE files. |
| 76 | `.gitignore` — KEEP | closed-by-existing-gate | Comprehensive. |
| 77 | `.env.example` — KEEP | closed-by-existing-gate | Documents env-var names. |
| 78 | `.pre-commit-config.yaml` — KEEP | closed-by-existing-gate | Banned-words hook. |
| 79 | `.editorconfig` — MISSING (optional) | out-of-scope | Not a blocker; deferred. |
| 80 | `.gitattributes` — MISSING (optional) | out-of-scope | Not a blocker; deferred. |
| 81 | `requirements-bench.txt` — KEEP | closed-by-existing-gate | Pins pytest. |
| 82 | `benchmarks/{README, RESULTS, fixtures/}` — KEEP | closed-by-existing-gate | OP-8 token-economy. |
| 83 | `examples/{01..05}/` — KEEP | closed-by-existing-gate | All 5 dirs current. |
| 84 | `research/scratch/` — DELETE empty dir | out-of-scope | `git ls-files research/scratch/` returns empty (already untracked). Empty dirs aren't tracked by git; `research/scratch/` is a workspace convention only. Removing the local-only directory adds zero value to the committed tree. Defer. |

## Crate-layout sanity

| # | Item | Disposition | Evidence/Citation |
|---|---|---|---|
| 85 | No merge candidates among 9 crates | closed-by-existing-gate | Each crate has distinct dep tree; KEEP. |
| 86 | No split candidates | closed-by-existing-gate | Largest crate `reposix-cli` modules <500 lines each. |
| 87 | `reposix-bench` forecast — KEEP track in backlog | out-of-scope | Tracked in STATE.md; not in v0.12.0 scope. |
| 88 | Fixtures live next to each crate (`crates/*/fixtures/`) — correct convention | closed-by-existing-gate | Verified. |
| 89 | `docs/reference/crates.md` — verify covers all 9 | out-of-scope | CATALOG-v2 said "rewrite-needed"; outside P62 scope. Tracked under v0.12.1 docs polish carry-forward. |

## What's clean — no action

| # | Item | Disposition | Evidence/Citation |
|---|---|---|---|
| 90 | `.github/` — current | closed-by-existing-gate | 6 workflows; only one stale comment in `ci.yml:55` correctly historical. |
| 91 | `crates/reposix-cache/` — clean post-Phase 31 | closed-by-existing-gate | Build clean. |
| 92 | `crates/reposix-core/` — single source of truth | closed-by-existing-gate | Compile-fail tests prove invariants. |
| 93 | `crates/reposix-cli/tests/` (12 files) — one-per-subcommand | closed-by-existing-gate | 1:1 dispatcher mapping. |
| 94 | `crates/reposix-remote/tests/{stateless_connect, push_conflict, bulk_delete_cap, protocol}.rs` | closed-by-existing-gate | v0.9.0 test moat. |
| 95 | `docs/{concepts, how-it-works, tutorials, guides, reference, benchmarks}/` | closed-by-existing-gate | Diátaxis IA from Phase 43. |
| 96 | `runtime/`, `target/`, `site/`, `.pytest_cache/`, `.playwright-mcp/` | closed-by-existing-gate | All gitignored. |
| 97 | `.claude/skills/{reposix-agent-flow, reposix-banned-words, reposix-quality-review}/` | closed-by-existing-gate | Per OP-4 carve-out. |
| 98 | `examples/` 5 dirs | closed-by-existing-gate | Matches launch checklist. |
| 99 | `benchmarks/fixtures/` | closed-by-existing-gate | `check-fixtures.py` validates. |

---

## Items closed by P62 Wave 3 (commits `8842d48` + verifier extension)

All Wave 3 fix items closed. Final closure breakdown:

| Original Wave-3 item | Final disposition | Closure |
|---|---|---|
| (rec #3 / #50) `scripts/__pycache__/` | closed-by-deletion | `.pyc` files were workspace-only (never tracked); `.gitignore:30` already covers `__pycache__/`. |
| (rec #23) `.planning/v0.{9,10}.0-MILESTONE-AUDIT.md` | closed-by-relocation | `git mv` to `.planning/milestones/audits/` (commit `8842d48`). |
| (rec #31) `.planning/SESSION-END-STATE*` | closed-by-relocation | `git mv` to `.planning/archive/session-end-state/` + new `README.md` naming `quality/PROTOCOL.md` as supersession (commit `8842d48`). |
| (rec #56) `docs/development/roadmap.md` FUSE mentions | closed-by-existing-gate | All 3 mentions are historical release-notes context (allowed pattern, like CHANGELOG). No edit needed. |
| (rec #57) `docs/development/contributing.md` FUSE mention | closed-by-existing-gate | False positive in source audit — the substring matched "**re**fuse"; no actual `fuse`/`FUSE` reference present. |
| (rec #71) `PUBLIC-LAUNCH-CHECKLIST.md` tag command | out-of-scope | Defer to v0.11.0 tag-push owner gate (per STATE.md). |
| (rec #84) `research/scratch/` | out-of-scope | Workspace-only convention; `git ls-files` returns empty. Empty dirs aren't tracked. |
| (verifier extension) 3 P62 Wave 1 row branches | closed-by-existing-gate | `quality/gates/structure/freshness-invariants.py` extended with `verify_no_loose_top_level_planning_audits`, `verify_no_pre_pivot_doc_stubs`, `verify_repo_org_audit_artifact_present`. All 3 exit 0 on the post-Wave-3 repo state. |

After Wave 3, the 3 Wave 1 waivers stay armed until 2026-05-15 as a safety net, but are no longer load-bearing — the runner re-grades the rows to PASS on next cadence sweep (waiver dominance keeps them WAIVED-as-GREEN until expiry).
