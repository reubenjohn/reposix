# v0.11.0 ROADMAP — Polish & Reproducibility

> Extracted from `.planning/ROADMAP.md` on 2026-04-27 per **QG-08 + POLISH-STRUCT** (P57 Wave D).
> Content below is verbatim from the OLD top-level ROADMAP.md historical section for v0.11.0.
> Active-milestone ROADMAP work lives at `.planning/ROADMAP.md`; this file is historical.

---

> **Status:** scoping complete; phases 50–55 scaffolded. v0.10.0 surfaced a long tail (jargon density, broken mermaid renders, codebase duplicates flagged by `simplify`, missing reproducibility infra). v0.11.0 closes that tail and surfaces the vision-innovations API (`reposix doctor`, `reposix log --time-travel`, `reposix gc --orphans`, `reposix cost`, `reposix init --since`). Source-of-truth research: `.planning/research/v0.11.0-vision-and-innovations.md` plus the audit family (`v0.11.0-gsd-hygiene-report.md`, `v0.11.0-mkdocs-site-audit.md`, `v0.11.0-jargon-inventory.md`, `v0.11.0-latency-benchmark-plan.md`, `v0.11.0-release-binaries-plan.md`, `v0.11.0-cache-location-study.md`, `v0.11.0-CATALOG-v2.md`).

**Thesis.** v0.10.0 made the value prop legible. v0.11.0 makes the project reproducible (fresh clone → working tutorial → installable binary), polished (no jargon shocks, no broken diagrams, no zombie ADRs), and operationally honest (latency numbers for every backend, copy-pastable doctor output, time-travel + gc + cost surfaces).

### Phase 50: Hygiene & Cleanup Wave (v0.11.0)

**Goal:** Clean GSD planning state, bump workspace version, sweep archival files, and triage the open dependabot PR. Establish a clean baseline so Phases 51–55 land against a consistent ledger.

**Requirements:** POLISH-11 (archival sweep), POLISH-12 (workspace version bump — partial; final tag-time bump in milestone close)

**Depends on:** (nothing — entry-point phase)

**Success criteria:**
1. STATE/PROJECT/REQUIREMENTS/ROADMAP all consistent — frontmatter `milestone: v0.11.0`; v0.1.0 MVD ghosts removed from PROJECT.md `Active`; v0.10.0 DOCS-01..11 archived in REQUIREMENTS.md.
2. `mkdocs build --strict` green after the sweep.
3. Dependabot chore PR #15 (rustix 1.x / rand 0.9 / sha2 0.11) either merged or closed-with-rationale (no open undecided chore PRs).
4. Archival files deleted: `MORNING-WALKTHROUGH-2026-04-25.md`, `RELEASE-NOTES-v0.10.0.md`, `RELEASE-NOTES-v0.11.0-PREVIEW.md`, `docs/blog/2026-04-25-reposix-launch.md`, `docs/archive/MORNING-BRIEF.md`, `docs/archive/PROJECT-STATUS.md`.
5. `.planning/phases/30-docs-ia-...` archived to `.planning/milestones/v0.9.0-phases/30-docs-ia-deferred-superseded/`; `find .planning/phases/ -mindepth 1 -maxdepth 1 -type d` returns empty.
6. Workspace version `0.11.0-dev` lands; `cargo run -p reposix-cli -- --version` prints `reposix 0.11.0-dev`.

**Context anchor:** `.planning/research/v0.11.0-gsd-hygiene-report.md` (full P0/P1/P2 patch list — line-numbered fixes for STATE.md / PROJECT.md / REQUIREMENTS.md / ROADMAP.md), `.planning/research/v0.11.0-CATALOG-v2.md` (catalog of every `.planning/` artifact with keep/move/delete verdicts).

### Phase 51: Codebase Refactor Wave (v0.11.0)

**Goal:** Kill the four duplicates flagged by `simplify` during the v0.10.0 audit: 4-way CLI worktree-helper duplication, `parse_remote_url` clones across `reposix-core` and `reposix-remote/backend_dispatch`, the dead `cli_compat.rs` shim in `reposix-cache`, and FUSE residue in `crates/reposix-cli/src/refresh.rs`.

**Requirements:** POLISH-13, POLISH-14, POLISH-15, POLISH-16

**Depends on:** Phase 50 (clean planning ledger before refactor)

**Success criteria:**
1. Zero duplicate `parse_remote_url` definitions. Single source in `reposix-core`; `reposix-remote/backend_dispatch` calls into it.
2. One `worktree_helpers` module at `crates/reposix-cli/src/worktree_helpers.rs`; the four ad-hoc copies in `init.rs`, `tokens.rs`, `doctor.rs`, `gc.rs` (or wherever they live) call into it.
3. `crates/reposix-cache/src/cli_compat.rs` deleted; downstream consumers migrated to the canonical opener.
4. Zero FUSE field/fn references in non-test code. `git grep -i 'is_fuse_active\|mount_point' crates/reposix-cli/src/refresh.rs` returns empty.
5. `cargo clippy --workspace --all-targets -- -D warnings` green.
6. `cargo test --workspace` green; existing test count preserved or grown.

**Context anchor:** `.planning/research/v0.11.0-CATALOG-v2.md` (duplicate inventory + simplify findings), v0.10.0 Phase 45 simplify pass (recorded in audit).

### Phase 52: Docs Polish Wave (v0.11.0)

**Goal:** Ship the docs polish pass. Inline-gloss every jargon term at first occurrence per page; add `docs/reference/glossary.md` with ≥24 entries; fix every mermaid render bug surfaced by the live-site audit; delete ADR-004 + ADR-006 (superseded — Issue→Record + IssueBackend→BackendConnector); add a v0.9.0-pivot disclaimer to `docs/research/agentic-engineering-reference.md`; rewrite `docs/how-it-works/` and `docs/guides/integrate-with-your-agent.md` for the new vocabulary.

**Requirements:** POLISH-01, POLISH-02, POLISH-03, POLISH-04, POLISH-11

**Depends on:** Phase 50 (clean ledger before docs sweep)

**Success criteria:**
1. Live site has zero `Syntax error in text` console errors (asserted via playwright `browser_console_messages` on every page).
2. Glossary covers ≥24 terms; every other doc page links to `docs/reference/glossary.md` on first jargon term occurrence per page.
3. ADR-004 + ADR-006 deleted from `docs/decisions/`; remaining ADR cross-refs purged.
4. `docs/research/agentic-engineering-reference.md` carries a top-banner disclaimer naming the v0.9.0 pivot and the deletion of FUSE.
5. `mkdocs build --strict` green; `pymdownx.emoji` extension configured; ADR-008 in nav; blog post in `not_in_nav` (or deleted per Phase 50).
6. Mermaid F1+F2+F3 fixes from `.planning/research/v0.11.0-mkdocs-site-audit.md` all landed.

**Context anchor:** `.planning/research/v0.11.0-jargon-inventory.md` (term-by-term gloss inventory across every doc page), `.planning/research/v0.11.0-mkdocs-site-audit.md` (live-site audit findings F1/F2/F3 + nav fixes).

### Phase 53: Reproducibility Wave (v0.11.0)

**Goal:** Make the project reproducible end-to-end: `bash scripts/repro-quickstart.sh` runs the 7-step tutorial against a fresh `/tmp/clone`, dist publishes pre-built binaries on every git tag, `cargo binstall reposix-cli` works, CLAUDE.md gains a playwright-validation rule for any docs-site change, `scripts/check-docs-site.sh` is wired into pre-push.

**Requirements:** POLISH-05, POLISH-06, POLISH-07, POLISH-17

**Depends on:** Phase 50 (clean ledger), Phase 51 (no duplicate symbols leaking into the binary), Phase 52 (tutorial copy must reflect post-polish vocabulary)

**Success criteria:**
1. `bash scripts/repro-quickstart.sh` passes from a fresh `/tmp/clone` — runs the 7-step `docs/tutorials/first-run.md` tutorial step-by-step, asserts each step succeeds.
2. Tag push triggers the dist release pipeline; binaries appear on GitHub Releases for `linux-musl-x86_64`, `linux-musl-aarch64`, `macos-x86_64`, `macos-aarch64`, `windows-msvc-x86_64`.
3. `cargo binstall reposix-cli` works against a published tag; integration test asserts version matches.
4. CLAUDE.md updated with: any docs-site work MUST be playwright-validated.
5. `scripts/check-docs-site.sh` exists, is executable, and is wired into the pre-push hook (not just CI). Hook fails on broken links / missing pages / mermaid errors / mkdocs --strict failures.

**Context anchor:** `.planning/research/v0.11.0-release-binaries-plan.md` (dist setup, target matrix, signing strategy), `docs/tutorials/first-run.md` (current 7-step tutorial — the contract `repro-quickstart.sh` enforces).

### Phase 54: Real-backend Latency Wave (v0.11.0)

**Goal:** Populate the latency table for sim + github + confluence + jira with record counts and 3-sample medians. Add a weekly cron that PR-creates table updates so the artifact stays honest.

**Requirements:** POLISH-08

**Depends on:** Phase 50 (clean ledger), Phase 51 (refactors must land before bench — bench-time symbol drift is the worst kind)

**Success criteria:**
1. `docs/benchmarks/v0.9.0-latency.md` (or v0.11.0-latency.md, per plan) has all 4 backend columns populated with measured numbers, footnotes naming N records, 3-sample medians.
2. Weekly cron (GitHub Actions schedule) runs the bench harness against sim + (when secrets present) github / confluence / jira; PR-creates an updated table on drift.
3. Bench harness is committed under `crates/reposix-bench/` or `scripts/bench/`; reproducible by any contributor.

**Context anchor:** `.planning/research/v0.11.0-latency-benchmark-plan.md` (full benchmark plan — golden path, sample sizes, statistical handling, secret-gated CI matrix).

### Phase 55: Vision Innovations Surface Wave (v0.11.0)

**Goal:** Surface the vision-innovations API: complete `reposix doctor` (full v3a checklist with copy-pastable fix strings), `reposix cost` (cumulative blob bytes + audit-row count + per-backend egress estimate), `reposix log --time-travel` (audit-log query with timestamp filter), `reposix init --since=<RFC3339>` (delta-clone from a point in time), `reposix gc --orphans` (cache cleanup of unreferenced blobs).

**Requirements:** POLISH-09, POLISH-10

**Depends on:** Phase 50 (clean ledger), Phase 51 (no duplicates blocking the new code paths)

**Success criteria:**
1. All five subcommands have integration tests against `reposix-sim`.
2. `reposix doctor` runs the full v3a checklist; every failure mode emits a copy-pastable fix string (no narrative-only output).
3. `--help` examples for each new subcommand land in the tutorial or troubleshooting guide.
4. CHANGELOG `[v0.11.0]` documents the new surfaces.

**Context anchor:** `.planning/research/v0.11.0-vision-and-innovations.md` (full spec for doctor checklist, cost estimator semantics, time-travel UX, gc orphan policy, init --since semantics).
