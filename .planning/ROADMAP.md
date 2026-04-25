# Roadmap: reposix

## Milestones

- ✅ **v0.1.0 MVD** — Phases 1-4, S (shipped 2026-04-13) · [archive](milestones/v0.8.0-ROADMAP.md)
- ✅ **v0.2.0-alpha** — Phase 8: GitHub read-only adapter (shipped 2026-04-13)
- ✅ **v0.3.0** — Phase 11: Confluence Cloud read-only adapter (shipped 2026-04-14)
- ✅ **v0.4.0** — Phase 13: Nested mount layout pages/+tree/ (shipped 2026-04-14)
- ✅ **v0.5.0** — Phases 14-15: IssueBackend decoupling + bucket _INDEX.md (shipped 2026-04-14)
- ✅ **v0.6.0** — Phases 16-20: Write path + full sitemap (shipped 2026-04-14)
- ✅ **v0.7.0** — Phases 21-26: Hardening + Confluence expansion + docs (shipped 2026-04-16)
- ✅ **v0.8.0 JIRA Cloud Integration** — Phases 27-29 (shipped 2026-04-16)
- ✅ **v0.9.0 Architecture Pivot — Git-Native Partial Clone** — Phases 31–36 (shipped 2026-04-24) · [archive](milestones/v0.9.0-ROADMAP.md)
- ✅ **v0.10.0 Docs & Narrative Shine** — Phases 40–45 (shipped 2026-04-25) · [archive](milestones/v0.10.0-ROADMAP.md)
- 📋 **v0.11.0 Polish & Reproducibility** — planning (Phases 50–55 scaffolded; jargon glosses + glossary, mermaid render hygiene, fresh-clone tutorial runner, dist release pipeline, real-backend latency table, vision-innovations surface — `.planning/research/v0.11.0-vision-and-innovations.md`)

## Phases

## v0.11.0 Polish & Reproducibility (PLANNING)

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

---

<details>
<summary>✅ v0.10.0 Docs & Narrative Shine (Phases 40–45) — SHIPPED 2026-04-25</summary>

## v0.10.0 Docs & Narrative Shine (PLANNING)

> **Status:** scoping complete; phases 40–45 scaffolded. The architecture pivot shipped in v0.9.0 (2026-04-24); v0.10.0 ports the deferred Phase 30 docs work onto the git-native design and adds tutorial / how-it-works / mental-model pages around the new flow. Source-of-truth design in `.planning/research/v0.10.0-post-pivot/milestone-plan.md`. Original narrative IA in `.planning/notes/phase-30-narrative-vignettes.md` (framing principles P1/P2 inherited; banned-word list revised for git-native). v0.9.0 archive in `.planning/milestones/v0.9.0-ROADMAP.md`.

**Thesis.** A cold visitor understands reposix in 10 seconds and runs the tutorial in 5 minutes. The architecture pivot becomes a story, not a code change.

**Carry-forward from v0.9.0 (tech debt):** Helper hardcodes `SimBackend` in the `stateless-connect` handler — documented in `.planning/v0.9.0-MILESTONE-AUDIT.md` §5. Resolution scheduled before v0.11.0 benchmark commits (track as a hotfix or v0.11.0 prereq).

### Phase 40: Hero + concepts — landing page lands the value prop in 10 seconds (v0.10.0)

**Goal:** Rewrite `docs/index.md` and the README hero so a cold reader states reposix's value prop within 10 seconds. Hero opens with a V1 before/after code block (Jira-close vignette from `.planning/notes/phase-30-narrative-vignettes.md`) and a three-up value-prop grid that cites *measured* numbers from `docs/benchmarks/v0.9.0-latency.md` (`8 ms` get-issue, `24 ms` `reposix init` cold, `9 ms` list-issues, `5 ms` capabilities probe). Add the two home-adjacent concept pages: `docs/concepts/mental-model-in-60-seconds.md` (clone = snapshot · frontmatter = schema · `git push` = sync verb) and `docs/concepts/reposix-vs-mcp-and-sdks.md` (positioning page grounding P1, with a numbers-table contrasting tokens-per-task, latency, and dependency footprint). README hero adjective-rewrite is split: this phase replaces the `docs/index.md` hero copy; Phase 45 finishes the README. Operating-principle hooks: **OP-1 close the feedback loop** — render every diagram via mcp-mermaid + playwright screenshot before merge; **numbers, not adjectives** — banned-word linter (Phase 43) rejects any adjective in the hero that doesn't dereference a number in `docs/benchmarks/v0.9.0-latency.md`; **P1 complement-not-replace** — the word "replace" stays out of the hero copy.

**Requirements:** DOCS-01, DOCS-03, DOCS-08 (partial — index.md hero only)

**Depends on:** (nothing — v0.9.0 shipped; latency artifact already committed)

**Success criteria:**
1. `mkdocs build --strict` green for `docs/index.md`, `docs/concepts/mental-model-in-60-seconds.md`, `docs/concepts/reposix-vs-mcp-and-sdks.md`.
2. Cold-reader test: a `doc-clarity-review` subagent given only `docs/index.md` (copied to `/tmp`, no repo context) states reposix's value proposition unprompted within 10s of reading.
3. Every adjective in the `docs/index.md` hero block dereferences a number sourced from `docs/benchmarks/v0.9.0-latency.md` (assertable: `scripts/banned-words-lint.sh` Phase 43 acceptance test pre-flight).
4. The word "replace" does NOT appear in the hero or three-up value props.
5. `docs/concepts/mental-model-in-60-seconds.md` is one-page (≤ 60-second read; `wc -w` ≤ 250).
6. `docs/concepts/reposix-vs-mcp-and-sdks.md` numbers-table cites `docs/benchmarks/v0.9.0-latency.md` by relative link and renders in MkDocs.
7. Playwright screenshots committed for landing page at desktop (1280px) and mobile (375px) widths.

**Context anchor:** `.planning/research/v0.10.0-post-pivot/milestone-plan.md` §2 v0.10.0 Phase 41 entry, `.planning/notes/phase-30-narrative-vignettes.md` (hero vignette V1 + framing principles P1/P2), `docs/benchmarks/v0.9.0-latency.md` (numbers source), `.planning/milestones/v0.9.0-ROADMAP.md` (architecture context).

### Phase 41: How-it-works trio — three pages, three diagrams, git-native architecture as a story (v0.10.0)

**Goal:** Carve `docs/how-it-works/{filesystem-layer,git-layer,trust-model}.md` from the existing `docs/architecture.md` + `docs/security.md` + the v0.9.0 architecture-pivot summary. **Reframed for git-native** — `filesystem-layer.md` describes the cache + working tree + frontmatter (NOT FUSE / inode / daemon, all of which are Layer 4 jargon now); `git-layer.md` describes the promisor remote + push round-trip *as user-experience* (the words "stateless-connect", "fast-import", "protocol-v2" are Layer 4 only and stay in `docs/reference/git-remote.md`); `trust-model.md` covers taint typing, allowlist, append-only audit, and the blob-limit guardrail. Each page has exactly one mermaid diagram rendered via mcp-mermaid and screenshot-verified via playwright. Operating-principle hooks: **OP-1 close the feedback loop** — `mcp__mcp-mermaid__generate_mermaid_diagram` then `mcp__playwright__browser_take_screenshot` for every diagram, both committed; **OP-4 self-improving infrastructure** — old `docs/architecture.md` + `docs/security.md` either re-pointed (frontmatter redirect) or deleted in Phase 43 nav restructure, never left as zombie pages. P2 progressive disclosure must be honored — the new banned-above-Layer-3 list (`partial-clone`, `promisor`, `stateless-connect`, `fast-import`, `protocol-v2`) is permitted on these three pages but nowhere above them.

**Requirements:** DOCS-02

**Depends on:** Phase 40 (hero must land the value prop before architecture is leaked)

**Success criteria:**
1. `docs/how-it-works/filesystem-layer.md`, `docs/how-it-works/git-layer.md`, `docs/how-it-works/trust-model.md` all exist and render under `mkdocs build --strict`.
2. Each page has exactly one mermaid diagram (assertable: `grep -c '```mermaid' <file>` returns `1`).
3. Each diagram is rendered to PNG via mcp-mermaid; the PNG is committed under `docs/how-it-works/diagrams/` and visually reviewed for spaghetti edges, overlapping labels, unreadable node text per global OP-1.
4. Playwright screenshots of each rendered page (desktop + mobile) committed under `docs/screenshots/how-it-works/`.
5. The words `FUSE`, `fusermount`, `inode`, `daemon`, `mount`, `kernel`, `syscall` do NOT appear in any of the three pages (banned per P2 + git-native cleanup).
6. `filesystem-layer.md` describes the cache layer + working tree + frontmatter (the v0.9.0 architecture), not the deleted FUSE design.
7. `git-layer.md` describes push round-trip and conflict rebase as user experience; protocol-v2 jargon stays in Reference (`docs/reference/git-remote.md`).
8. Cross-link from each page back to `docs/concepts/mental-model-in-60-seconds.md` (Phase 40 anchor).

**Context anchor:** `.planning/research/v0.10.0-post-pivot/milestone-plan.md` §2 v0.10.0 Phase 42 entry, `.planning/research/v0.9-fuse-to-git-native/architecture-pivot-summary.md` §2–4 (cache + transport + sync), `docs/architecture.md` + `docs/security.md` (source content to carve), `.planning/notes/phase-30-narrative-vignettes.md` (P2 layer rules), `.planning/milestones/v0.9.0-ROADMAP.md` Phases 31–34.

### Phase 42: Tutorial + guides + simulator relocate to Reference (v0.10.0)

**Goal:** Ship the 5-minute first-run tutorial (`docs/tutorials/first-run.md`) that takes a fresh user from `cargo install reposix` (or release tarball) through `reposix init sim::demo /tmp/repo`, `cat`, edit, `git commit`, `git push`, and ends with a real edit applied. The tutorial is end-to-end runnable; `scripts/tutorial-runner.sh` verifies each step (the doc IS the test, per OP-1). Add the three guides: `docs/guides/write-your-own-connector.md` (BackendConnector trait walkthrough — moved/rewritten from existing `docs/connectors/guide.md`), `docs/guides/integrate-with-your-agent.md` (Claude Code / Cursor / SDK patterns — pointer page; full recipes ship in v0.12.0), `docs/guides/troubleshooting.md` (push rejections, audit-log queries, blob-limit recovery — sourced from v0.9.0 Phase 34 + 35 verification artifacts). Move simulator docs out of "How it works" into Reference (`docs/reference/simulator.md`, deduplicating against existing `crates/reposix-sim` docs and `docs/reference/http-api.md`). Operating-principle hooks: **OP-1 close the feedback loop** — `scripts/tutorial-runner.sh` runs in CI; **OP-6 ground truth obsession** — the tutorial commits a real edit and asserts the simulator audit log row.

**Requirements:** DOCS-04, DOCS-05, DOCS-06

**Depends on:** Phase 41 (how-it-works trio shapes the vocabulary the tutorial inherits)

**Success criteria:**
1. `docs/tutorials/first-run.md` exists and renders under `mkdocs build --strict`.
2. `scripts/tutorial-runner.sh` exists, is executable, and runs the tutorial end-to-end against `reposix-sim` in under 5 minutes wall clock on the dev host.
3. CI workflow gains a `tutorial-runner` job that invokes `scripts/tutorial-runner.sh` and fails if any step diverges from the tutorial copy.
4. `docs/guides/write-your-own-connector.md`, `docs/guides/integrate-with-your-agent.md`, `docs/guides/troubleshooting.md` all exist and render.
5. `docs/guides/integrate-with-your-agent.md` is explicitly a pointer page — it links to `docs/tutorials/first-run.md` for setup, lists Claude Code / Cursor / SDK as v0.12.0-coming-soon, and does NOT inline recipe code (that's v0.12.0 scope).
6. `docs/reference/simulator.md` exists; any prior simulator content under `docs/how-it-works/` is removed or redirected.
7. `docs/connectors/guide.md` either redirects to `docs/guides/write-your-own-connector.md` or is deleted (Phase 43 nav restructure finalizes this).
8. The tutorial ends with a successful `git push` and a one-line audit-log assertion the reader can run themselves.

**Context anchor:** `.planning/research/v0.10.0-post-pivot/milestone-plan.md` §2 v0.10.0 Phase 43 entry, `docs/reference/cli.md` (current CLI surface, post v0.9.0), `docs/connectors/guide.md` (existing content to migrate), `.planning/milestones/v0.9.0-ROADMAP.md` Phase 34 + 35 (push rejection + blob-limit recovery as troubleshooting source).

### Phase 43: Nav restructure (Diátaxis) + theme tuning + banned-words linter (v0.10.0)

**Goal:** Restructure `mkdocs.yml` per Diátaxis (Home / How it works / Tutorials / Guides / Reference / Decisions / Research). Tune mkdocs-material theme (palette, hero features, social cards). Ship the banned-words linter `scripts/banned-words-lint.sh` (the layer-banned-word list lives in a checked-in config, e.g. `docs/.banned-words.toml`) and wire it into pre-commit + CI. Per global OP-4: the linter replaces the ad-hoc Phase 40-style grep — it is committed, reviewable code, not a one-off bash pipeline. Add (or extend) a project Claude Code skill `reposix-banned-words` at `.claude/skills/reposix-banned-words/SKILL.md` so authoring agents can self-check before commit. Delete or redirect now-stale top-level docs (`docs/architecture.md` carved into Phase 41 trio; `docs/security.md` carved into Phase 41 trust-model; `docs/connectors/guide.md` redirected per Phase 42); `mkdocs.yml not_in_nav` cleaned. Operating-principle hooks: **OP-4 self-improving infrastructure** — banned-words linter is the institutional memory of P2 framing rules, not a checklist; **OP-1 close the feedback loop** — pre-commit hook + CI integration both verified green before this phase ships.

**Requirements:** DOCS-07, DOCS-08 (linter wiring half), DOCS-09

**Depends on:** Phase 40, Phase 41, Phase 42 (all new pages must exist before nav restructure)

**Success criteria:**
1. `mkdocs.yml` nav reads top-down: Home / How it works / Tutorials / Guides / Reference / Decisions / Research (Diátaxis-clean — no mixed types within a section).
2. `mkdocs build --strict` green; no orphan pages in `not_in_nav`.
3. mkdocs-material theme palette + hero features + social cards configured; social cards generated and committed.
4. `scripts/banned-words-lint.sh` exists, is executable, exits non-zero on any P2 violation, and reads its layer list from `docs/.banned-words.toml` (or equivalent checked-in config).
5. Linter is wired into `.pre-commit-config.yaml` (or equivalent local hook) AND `.github/workflows/<docs|ci>.yml`; both invocations verified green on a clean tree and red on a seeded violation.
6. `.claude/skills/reposix-banned-words/SKILL.md` exists with frontmatter `name: reposix-banned-words` and `description: <one-line>`. Skill body documents the layer rules and points at `docs/.banned-words.toml`.
7. `docs/architecture.md`, `docs/security.md`, `docs/connectors/guide.md` are either deleted or replaced with one-line redirect stubs pointing at the carved-out successor pages.
8. P2 banned terms (`FUSE`, `fusermount`, `kernel`, `syscall`, `partial-clone`, `promisor`, `stateless-connect`, `fast-import`, `protocol-v2`) do not appear above Layer 3 (How it works) anywhere in `docs/`.

**Context anchor:** `.planning/research/v0.10.0-post-pivot/milestone-plan.md` §2 v0.10.0 Phase 44 entry + §3.5 (numbers-not-adjectives linter) + §5 (`reposix-banned-words` skill), `.planning/notes/phase-30-narrative-vignettes.md` §"P2 progressive disclosure" (layer rules — banned-word list revised), `mkdocs.yml` (current nav), global OP-4 (committed-artifact rule for ad-hoc bash).

### Phase 44: doc-clarity-review release gate — zero critical friction points (v0.10.0)

**Goal:** Run the `doc-clarity-review` skill as a release gate over every user-facing page in `docs/` — each page reviewed in isolation in a fresh `/tmp` context with no repo grounding (the cold-reader scenario, OP-6 ground truth). Findings are logged per page; zero critical friction points must remain in any user-facing page before v0.10.0 ships. Operating-principle hooks: **OP-1 close the feedback loop** — clarity review is run, not assumed; **OP-2 aggressive subagent delegation** — the orchestrator dispatches one `doc-clarity-review` subagent per page and integrates findings, never reads pages itself; **OP-6 ground truth obsession** — findings live in committed `.planning/phases/44-.../CLARITY-FINDINGS.md`, not session memory.

**Requirements:** DOCS-10

**Depends on:** Phase 40, Phase 41, Phase 42, Phase 43 (all docs must be in their final shape before clarity review)

**Success criteria:**
1. Every user-facing page in `docs/` (Home, How it works trio, Tutorials, Guides, concept pages) has a `doc-clarity-review` finding row in `.planning/phases/44-.../CLARITY-FINDINGS.md`.
2. Each finding row has a status: `clean` (no critical friction), `fixed-this-phase` (fix landed), or `deferred-with-justification` (documented reason, owner-approved).
3. Zero `critical` friction points remain across any page.
4. Findings file committed; `mkdocs build --strict` green after any fix-this-phase landings.
5. Subagent fan-out happened in parallel (one subagent per page) — verified via the `.planning/phases/44-.../EXECUTION-LOG.md`.

**Context anchor:** `.planning/research/v0.10.0-post-pivot/milestone-plan.md` §3.6 (`doc-clarity-review` on every doc page as release gate), `doc-clarity-review` skill at `.claude/skills/doc-clarity-review/SKILL.md` (or wherever it lives — discoverable via skill directory), Phase 26 doc-clarity-review precedent (`.planning/milestones/v0.8.0-ROADMAP.md` Phase 26 entry).

### Phase 45: README rewrite + CHANGELOG + screenshots + final integration + tag (v0.10.0)

**Goal:** Final integration phase. Rewrite the README hero — every adjective replaced with a measured number sourced from `docs/benchmarks/v0.9.0-latency.md` or v0.9.0 audit/threat-model artifacts. Point the README at the mkdocs site as the source of truth for narrative; root-level docs become stubs (`README.md` is grounding-only). Finalize CHANGELOG `[v0.10.0]` block. Commit playwright screenshots for landing + how-it-works trio + tutorial pages at desktop (1280px) and mobile (375px). Generate social cards. Cross-link `docs/benchmarks/v0.9.0-latency.md` from `docs/index.md`, the vs-MCP page, and the tutorial. Verify `mkdocs build --strict` green, banned-words linter green, all CI green. Author `scripts/tag-v0.10.0.sh` mirroring `scripts/tag-v0.9.0.sh` (6 safety guards minimum: clean tree, on `main`, version match, CHANGELOG `[v0.10.0]` exists, tests green, signed tag). Run `gsd-audit-milestone` + `gsd-complete-milestone` cleanup. Operating-principle hooks: **OP-1 close the feedback loop** — `gh run view` shows green CI before tag; **OP-6 ground truth obsession** — every README claim grounds in a committed artifact, no marketing copy.

**Requirements:** DOCS-08 (README hero rewrite half), DOCS-11

**Depends on:** Phase 44 (clarity gate must pass before release artifacts finalize)

**Success criteria:**
1. README.md hero rewritten — `scripts/banned-words-lint.sh --readme` (or equivalent guard) confirms zero adjectives lacking a number-source.
2. README points at the mkdocs site (`https://reubenjohn.github.io/reposix` or wherever it deploys) as the narrative source of truth; root README contains grounding-only content (install, build, link out).
3. CHANGELOG `[v0.10.0]` block finalized — summarizes phases 40–45 and lists DOCS-01..11 as shipped.
4. Playwright screenshots committed under `docs/screenshots/`: landing page (desktop + mobile), each how-it-works page (desktop + mobile), tutorial walkthrough (desktop). At least 8 PNGs committed.
5. Social cards generated and committed under `docs/social/assets/` (or theme-default location); MkDocs social plugin configured + green.
6. `docs/index.md`, `docs/concepts/reposix-vs-mcp-and-sdks.md`, and `docs/tutorials/first-run.md` all relative-link to `docs/benchmarks/v0.9.0-latency.md`.
7. `mkdocs build --strict` green; banned-words linter green; `gh run view` on the release commit shows all CI jobs green.
8. `scripts/tag-v0.10.0.sh` exists with ≥ 6 safety guards (clone Phase 36's `scripts/tag-v0.9.0.sh`); `bash scripts/tag-v0.10.0.sh` is the user-gate handoff (orchestrator does NOT push the tag).
9. `gsd-audit-milestone` run and `.planning/v0.10.0-MILESTONE-AUDIT.md` written (mirrors `.planning/v0.9.0-MILESTONE-AUDIT.md`).

**Context anchor:** `.planning/research/v0.10.0-post-pivot/milestone-plan.md` §2 v0.10.0 Phase 45 entry + §3.5 (numbers-not-adjectives), `.planning/milestones/v0.9.0-ROADMAP.md` Phase 36 (release-cycle precedent: tag-script + audit + complete-milestone), `docs/benchmarks/v0.9.0-latency.md` (numbers source), `scripts/tag-v0.9.0.sh` (template for v0.10.0 tag script).

</details>

---

<details>
<summary>✅ v0.9.0 Architecture Pivot (Phases 31–36) — SHIPPED 2026-04-24</summary>

## v0.9.0 Architecture Pivot — Git-Native Partial Clone

**Motivation:** The FUSE-based design is fundamentally slow (every `cat`/`ls` triggers a live REST API call) and doesn't scale (10k Confluence pages = 10k API calls on directory listing). FUSE also has operational pain: fusermount3, /dev/fuse permissions, WSL2 quirks, pkg-config/libfuse-dev build dependencies. Research confirmed that git's built-in partial clone + the existing `git-remote-reposix` helper can replace FUSE entirely, giving agents a standard git workflow with zero custom CLI awareness required.

**Research:** See `.planning/research/v0.9-fuse-to-git-native/architecture-pivot-summary.md` (canonical design document), `partial-clone-remote-helper-findings.md` (transport layer POC), `push-path-stateless-connect-findings.md` (write path POC), `sync-conflict-design.md` (sync model). POC code in `poc/` subdir (`git-remote-poc.py`, `run-poc.sh`, `run-poc-push.sh`, trace logs).

**Key design decisions:**
- DELETE `crates/reposix-fuse` entirely; drop `fuser` dependency
- ADD `stateless-connect` capability to `git-remote-reposix` for partial-clone reads
- KEEP `export` capability for push (hybrid confirmed working in POC)
- ADD `reposix-cache` crate: backing bare-repo cache built from REST responses
- Agent UX is pure git: `git clone`, `cat`, `git push` — zero reposix CLI awareness
- Push-time conflict detection: helper checks backend state at push time, rejects with standard git error
- Blob limit guardrail: helper refuses to serve >N blobs, error message teaches agent to use sparse-checkout
- Tree sync always full (cheap metadata); blob materialization is the only limited/lazy operation
- Delta sync via `since` queries (all backends support this natively)

**Phases (31–36):**
1. Phase 31 — `reposix-cache` crate (bare-repo cache from REST responses, audit + tainted + allowlist)
2. Phase 32 — `stateless-connect` capability in `git-remote-reposix` (read path; protocol-v2 tunnel)
3. Phase 33 — Delta sync (`list_changed_since` on `BackendConnector` + cache integration)
4. Phase 34 — Push path (conflict detection + blob limit + frontmatter allowlist)
5. Phase 35 — CLI pivot (`reposix init`) + dark-factory agent UX validation
6. Phase 36 — FUSE deletion + CLAUDE.md update + `reposix-agent-flow` skill + release

### Phase 31: `reposix-cache` crate — backing bare-repo cache from REST responses (v0.9.0)

**Goal:** Land the foundation crate that materializes REST API responses into a real on-disk bare git repo. The cache is the substrate every later phase builds on. Operating-principle hooks for this phase: **audit log non-optional** (one row per blob materialization); **tainted-by-default** (cache returns `Tainted<Vec<u8>>` — the type system encodes the trust boundary); **egress allowlist** (no new HTTP client construction outside `reposix_core::http::client()`); **simulator-first** (every test in this crate runs against `SimBackend`). Per project CLAUDE.md "Subagent delegation rules": use `gsd-phase-researcher` for any "how do I build a bare git repo from raw blobs in Rust" question — non-trivial, easy to over-research in the orchestrator.

**Requirements:** ARCH-01, ARCH-02, ARCH-03

**Depends on:** (nothing — foundation phase)

**Success criteria:**
1. `cargo build -p reposix-cache` and `cargo clippy -p reposix-cache --all-targets -- -D warnings` clean.
2. Given a `SimBackend` seeded with N issues, `reposix_cache::Cache::build_from(backend)` produces a valid bare git repo on disk containing N blobs (lazy — only materialized on demand) and a tree object that lists every issue path.
3. Audit table contains exactly one `op="materialize"` row per blob materialization (test seeds N issues, materializes M blobs, asserts `count(*) == M`).
4. Cache returns blob bytes wrapped in `reposix_core::Tainted<Vec<u8>>`; a compile-fail test asserts that calling `egress::send(blob)` without `sanitize` is a type error.
5. Egress allowlist test: pointing the cache at a backend whose origin is not in `REPOSIX_ALLOWED_ORIGINS` returns an error and writes an audit row with `op="egress_denied"`.
6. SQLite audit table is append-only — `BEFORE UPDATE/DELETE RAISE` trigger asserted by integration test.

**Context anchor:** `.planning/research/v0.9-fuse-to-git-native/architecture-pivot-summary.md` §2 (How it works), §5 (Add — `reposix-cache` crate), §6 (What stays the same — `BackendConnector` trait reused), and §7 open question 2 (atomicity of REST write + bare-repo cache update — implementation note for this phase).

**Plans:** 3 plans across 3 waves

- [ ] 31-01-PLAN.md — Wave 1: reposix-cache crate scaffold + gix 0.82 smoke + Cache::build_from with lazy tree (ARCH-01)
- [ ] 31-02-PLAN.md — Wave 2: cache_schema.sql + audit/db/meta modules + Cache::read_blob (Tainted + egress-denial audit) + lift cache_db.rs from reposix-cli (ARCH-02, ARCH-03)
- [ ] 31-03-PLAN.md — Wave 3: trybuild compile-fail fixtures — Tainted→Untainted + Untainted::new pub(crate) locks (ARCH-02)

### Phase 32: `stateless-connect` capability in `git-remote-reposix` (read path) (v0.9.0)

**Goal:** Port the Python POC's `stateless-connect` handler to Rust inside `crates/reposix-remote/`. Tunnel protocol-v2 traffic to the Phase 31 cache so `git clone --filter=blob:none reposix::sim/proj-1 /tmp/clone` works end-to-end with lazy blob loading. The existing `export` capability for push must keep working in the same binary (hybrid). Operating-principle hooks: **subagent delegation per project CLAUDE.md** — use `gsd-phase-researcher` for the protocol-v2 stateless-connect Rust port (non-trivial; three protocol gotchas from POC must be encoded correctly or git misframes the next request); **ground truth obsession** — verify against a real `git clone` run, not against unit-test mocks; **close the feedback loop** — capture a fresh trace log analogous to POC `poc-helper-trace.log` and commit it under `.planning/research/v0.9-fuse-to-git-native/rust-port-trace.log`.

**Requirements:** ARCH-04, ARCH-05

**Depends on:** Phase 31

**Success criteria:**
1. `git clone --filter=blob:none reposix::sim/proj-1 /tmp/clone` succeeds with all blobs missing (assertable via `git rev-list --objects --missing=print --all`).
2. Lazy blob fetch on `git cat-file -p <oid>` hits the backend exactly once per OID (idempotent — second `cat-file` is local-only; assertable via audit-row count).
3. `git checkout origin/main` after `git sparse-checkout set issues/PROJ-24*` batches blob fetches into a single `command=fetch` RPC (assertable: helper records exactly one `command=fetch` audit row with multiple `want` lines, not N rows with one `want` each).
4. Refspec namespace is `refs/heads/*:refs/reposix/*` (regression test that `refs/heads/*:refs/heads/*` would cause empty-delta bug per POC).
5. The same helper binary still services `git push` via `export` (hybrid POC parity). Existing v0.8.0 push tests pass unchanged.
6. Three protocol gotchas (initial advert no `0002`; subsequent responses DO need `0002`; binary stdin throughout) are covered by named tests.

**Context anchor:** architecture-pivot-summary §3 (Confirmed Technical Findings — `stateless-connect`, transport routing, three protocol gotchas, refspec namespace). POC artifacts: `.planning/research/v0.9-fuse-to-git-native/poc/git-remote-poc.py`, `poc-helper-trace.log`, `run-poc.sh`.

### Phase 33: Delta sync — `list_changed_since` on `BackendConnector` + cache integration (v0.9.0)

**Goal:** Add incremental backend queries so `git fetch` after a backend mutation transfers only the changed issue's tree+blob, not the whole project. Wire `last_fetched_at` (already present in `crates/reposix-cli/src/cache_db.rs`) into the new `reposix-cache` crate, and update it atomically with each delta sync. Operating-principle hooks: **simulator-first** (sim respects `since` query param; all delta-sync tests use sim); **audit log non-optional** (one audit row per delta-sync invocation); **ground truth obsession** (test asserts that after a single backend mutation, exactly one issue's blob OID changes — not all of them).

**Requirements:** ARCH-06, ARCH-07

**Depends on:** Phase 31, Phase 32

**Success criteria:**
1. `BackendConnector::list_changed_since(timestamp) -> Vec<IssueId>` defined on the trait and implemented for `SimBackend`, `GithubBackend`, `ConfluenceBackend`, `JiraBackend`. Each backend uses its native incremental query (`?since=`, JQL `updated >=`, CQL `lastModified >`).
2. `SimBackend` REST surface respects a `since` query parameter (if absent, returns all — backwards compatible).
3. After `agent_a` mutates issue `proj-1/42` on the simulator and `agent_b` runs `git fetch origin`, `git diff --name-only origin/main` returns exactly `issues/42.md`. Other blob OIDs are unchanged.
4. Tree sync is unconditional (not gated by `REPOSIX_BLOB_LIMIT`); the limit only applies to blob materialization.
5. Cache update + `last_fetched_at` write happen in one SQLite transaction (kill-9 chaos test asserts no divergent state — borrows the Phase 21 HARD-03 chaos pattern).
6. One audit row per delta-sync invocation: `(ts, backend, project, since_ts, items_returned, op="delta_sync")`.

**Context anchor:** architecture-pivot-summary §4 (Sync and Conflict Model — delta sync via `since` queries, fetch flow, agent-sees-changes-via-pure-git). Existing `cache_db.rs` `refresh_meta` row is the storage location for `last_fetched_at`.

### Phase 34: Push path — conflict detection + blob limit guardrail (v0.9.0)

**Goal:** Make the `export` handler conflict-aware and the `stateless-connect` handler scope-bounded. Push-time conflict detection rejects stale-base pushes with a canned `fetch first` git status so agents experience the standard "git pull --rebase, retry" cycle without learning anything new. Blob-limit guardrail caps `command=fetch` size so a runaway `git grep` cannot melt API quotas — and the stderr message names `git sparse-checkout` so an unprompted agent self-corrects (dark-factory pattern). Operating-principle hooks: **tainted-by-default** (frontmatter sanitize step is the explicit `Tainted -> Untainted` conversion); **audit log non-optional** (every push attempt — accept and reject — gets an audit row); **ROI awareness** (blob-limit error message is the cheapest possible regression net for "agent does naive `git grep`").

**Requirements:** ARCH-08, ARCH-09, ARCH-10

**Depends on:** Phase 32

**Success criteria:**
1. Stale-base push: agent pushes a commit whose base differs from the current backend version. Helper emits `error refs/heads/main fetch first` (canned status, git renders the standard "perhaps a `git pull` would help" hint) and a detailed diagnostic via stderr through `diag()`. Reject path drains the incoming stream and never touches the bare cache (no partial state — assertable: `git fsck` clean after reject).
2. Successful push: REST writes apply, bare-repo cache updates, helper emits `ok refs/heads/main`. REST + cache update is atomic (kill-9 between REST and cache leaves state consistent — same chaos pattern as Phase 33).
3. Frontmatter field allowlist: an issue body with `version: 999999` in frontmatter does not change the server version; `id`, `created_at`, `updated_at` are likewise stripped. Asserted by named test.
4. Blob limit: a `command=fetch` request with > `REPOSIX_BLOB_LIMIT` `want` lines (default 200) is refused. Helper's stderr message is verbatim: `error: refusing to fetch <N> blobs (limit: <M>). Narrow your scope with \`git sparse-checkout set <pathspec>\` and retry.`
5. `REPOSIX_BLOB_LIMIT` env var is read at helper startup; integration test asserts that setting it to `5` causes a 6-want fetch to fail and a 5-want fetch to succeed.
6. Audit row for every push attempt, accept and reject: `(ts, backend, project, ref, files_touched, decision, reason)`.

**Context anchor:** architecture-pivot-summary §3 ("Helper can count want lines and refuse", "Push rejection format", "Conflict detection happens inside `handle_export`"), §4 ("Blob limit as teaching mechanism"), §7 open question 2 (REST + cache atomicity). POC artifacts: `.planning/research/v0.9-fuse-to-git-native/poc/git-remote-poc.py` (push reject path), `poc-push-trace.log`.

### Phase 35: CLI pivot — `reposix init` replacing `reposix mount` + agent UX validation (v0.9.0)

**Goal:** Replace the `reposix mount` command with `reposix init <backend>::<project> <path>` (which `git init`s, configures `extensions.partialClone`, sets the remote URL, and runs `git fetch --filter=blob:none origin`). Then run the dark-factory acceptance test: a fresh subprocess agent with no reposix CLI awareness completes a clone -> grep -> edit -> commit -> push -> conflict -> pull --rebase -> push cycle against the simulator without invoking any `reposix` subcommand other than `init`. The dark-factory regression must run against BOTH the simulator AND at least one real backend: Confluence "TokenWorld" space, GitHub `reubenjohn/reposix` issues, or JIRA project `TEST` (credentials permitting). Latency for each step of the golden path (clone, first-blob, sparse-batched checkout, edit, push, conflict, pull-rebase, push-again) is captured and asserted against soft thresholds. Operating-principle hooks: **agent UX = pure git** (zero in-context learning required); **close the feedback loop** (acceptance test runs in CI and on local dev via the Phase 36 skill); **ground truth obsession** (the agent's transcript is captured as a test fixture so regressions are visible in `git diff`); **real backends are first-class test targets** (per project CLAUDE.md OP-6 — simulator-only coverage does NOT satisfy transport/perf acceptance).

**Requirements:** ARCH-11, ARCH-12, ARCH-16, ARCH-17 (capture)

**Depends on:** Phase 31, Phase 32, Phase 33, Phase 34

**Success criteria:**
1. `reposix init sim::proj-1 /tmp/repo` produces a directory containing a valid partial-clone working tree (`git rev-parse --is-inside-work-tree` returns true; `git config remote.origin.url` returns `reposix::sim/proj-1`; `git config extensions.partialClone` is set; `.git/objects` has tree objects but no blob objects until `git checkout` runs).
2. `reposix mount` is removed from the CLI; running it prints a helpful migration message pointing at `reposix init`.
3. CHANGELOG `[v0.9.0]` section documents the breaking CLI change with a migration note (`reposix mount /path` -> `reposix init <backend>::<project> /path`).
4. README.md updated to use `reposix init` everywhere.
5. **Dark-factory regression test (the headline acceptance test):** a subprocess Claude (or scripted shell agent acting as one) given ONLY a `reposix init` command + a goal ("find issues mentioning 'database' and add a TODO comment to each") completes the task using pure git/POSIX tools. The transcript exercises:
   - `cat`, `grep -r`, edit, `git add`, `git commit`, `git push` — happy path.
   - Conflict path: a second writer mutates one of the agent's target issues mid-flight; agent sees `! [remote rejected]`, runs `git pull --rebase`, retries `git push`, succeeds.
   - Blob-limit path: a naive `git grep` triggers the Phase 34 blob-limit error; agent reads the error message, runs `git sparse-checkout set issues/PROJ-24*`, retries, succeeds.
6. The transcript above is committed as a test fixture so any regression that breaks the dark-factory flow shows up in `git diff`.
7. Real-backend integration run passes against ≥1 of {Confluence TokenWorld, GitHub `reubenjohn/reposix`, JIRA `TEST`} when credentials present. Falls back to `#[ignore]` skip when absent, with a clear WARN that the v0.9.0 claim is unverified for that backend.
8. Latency captured for each golden-path step (clone, first-blob, sparse-batched checkout, edit, push, conflict, pull-rebase, push-again); written to `docs/benchmarks/v0.9.0-latency.md`. Soft thresholds asserted (sim cold clone < 500ms, real backend < 3s); regressions flagged but not CI-blocking.
9. `docs/reference/testing-targets.md` created documenting the three canonical targets (TokenWorld, `reubenjohn/reposix`, JIRA `TEST`) with env-var setup and the explicit "go crazy, it's safe" permission statement from the owner.

**Context anchor:** architecture-pivot-summary §4 ("Agent UX: pure git, zero in-context learning", "Blob limit as teaching mechanism"), §5 (Change — CLI flow). The acceptance test is the operationalization of architecture-pivot-summary §4's "agent learns from any tool error" claim. Project CLAUDE.md OP-6 (real backends as first-class test targets) defines the canonical TokenWorld / `reubenjohn/reposix` / JIRA `TEST` targets exercised here.

### Phase 36: FUSE deletion + CLAUDE.md update + `reposix-agent-flow` skill + final integration tests + release (v0.9.0)

**Goal:** Demolish FUSE entirely and ship v0.9.0. Per OP-4 self-improving infrastructure: **this phase updates project CLAUDE.md and adds the `reposix-agent-flow` skill — agent grounding must ship in lockstep with code**. There can be no window where CLAUDE.md describes deleted code, and no window where the project lacks the dark-factory regression skill that the v0.9.0 architecture is supposed to enable. Operating-principle hooks: **self-improving infrastructure (OP-4)** — CLAUDE.md + skill ship together with FUSE deletion; **close the feedback loop (OP-1)** — `gh run view` on the release tag must show green CI without the `apt install fuse3` step; **reversibility enables boldness (OP-5)** — execute via `gsd-pr-branch` or worktree so a botched FUSE deletion can be reverted in one move.

**Requirements:** ARCH-13, ARCH-14, ARCH-15, ARCH-17 (artifact), ARCH-18, ARCH-19

**Depends on:** Phase 35

**Success criteria:**
1. `crates/reposix-fuse/` is deleted (zero references in `cargo metadata --format-version 1` output).
2. `fuser` is removed from every `Cargo.toml` in the workspace (assertable: `grep -r '\bfuser\b' Cargo.toml crates/*/Cargo.toml` returns empty).
3. `cargo check --workspace && cargo clippy --workspace --all-targets -- -D warnings` clean.
4. CI workflow updated: drops `cargo test --features fuse-mount-tests`, drops `apt install fuse3`, drops `/dev/fuse` requirement. `gh run view` on the resulting commit shows green.
5. Project `CLAUDE.md` fully rewritten for git-native architecture per ARCH-14: no v0.9.0-in-progress banner — replaced with steady-state "Architecture (git-native partial clone)" section; FUSE references purged from elevator pitch, Operating Principles, Workspace layout, Tech stack, Commands, Threat model. `git grep -i 'fuser\|fusermount\|fuse-mount-tests\|reposix mount' CLAUDE.md` returns empty.
6. Skill `reposix-agent-flow` created at `.claude/skills/reposix-agent-flow/SKILL.md` with frontmatter `name: reposix-agent-flow` and `description: <one-line description referencing the dark-factory regression test>`. Skill body documents the test pattern and references architecture-pivot-summary §4. Skill is invoked from CI (release-gate job) and from local dev (`/reposix-agent-flow`).
7. `scripts/tag-v0.9.0.sh` created mirroring `scripts/tag-v0.8.0.sh` (6 safety guards minimum: clean tree, on `main`, version match in `Cargo.toml`, CHANGELOG `[v0.9.0]` exists, tests green, signed tag).
8. CHANGELOG `[v0.9.0]` section is finalized with all six phases summarized + breaking-change migration note (`reposix mount` -> `reposix init`).
9. Phase 35's dark-factory regression test (now invoked via the new skill) passes against the post-deletion codebase.
10. CI jobs `integration-contract-{confluence,github,jira}-v09` green on main (or `pending-secrets` when creds unavailable). Each job runs the ARCH-16 smoke suite and uploads latency rows as a run artifact.
11. Benchmark artifact `docs/benchmarks/v0.9.0-latency.md` includes a sim column AND at least one real-backend column (TokenWorld / `reubenjohn/reposix` / JIRA `TEST`). Soft thresholds documented; regressions flagged inline.
12. CLAUDE.md "Commands you'll actually use" section gains a "Testing against real backends" block naming TokenWorld / `reubenjohn/reposix` / JIRA `TEST` with env-var setup. CLAUDE.md OP-6 cross-references `docs/reference/testing-targets.md`.

**Context anchor:** architecture-pivot-summary §5 (Delete — `crates/reposix-fuse`, `fuser` dependency), §9 (Milestone Impact). Project `CLAUDE.md` "Subagent delegation rules" section. User global `CLAUDE.md` OP-4 "Self-improving infrastructure".

</details>

---

<details>
<summary>📋 Legacy Phase 30 — superseded by Phases 40–45 above (retained for traceability)</summary>

> **Why retained:** Phase 30 was originally scoped against the FUSE design. Now that v0.9.0 has shipped the git-native architecture, Phase 30's intent is delivered by the actively-tracked Phases 40–45 above. The plans below (`30-01-PLAN.md`..`30-09-PLAN.md`) are NOT executed; they are kept for traceability so anyone reading `git log` can trace v0.10.0's lineage back to the original narrative work.

### Phase 30: Docs IA and narrative overhaul — landing page aha moment and progressive-disclosure architecture reveal (v0.10.0) [SUPERSEDED]

**Goal:** Rewrite the landing page and restructure the MkDocs nav so reposix's value proposition lands hard within 10 seconds of a cold reader arriving, with technical architecture progressively revealed in a "How it works" section rather than leaked above the fold. Expand the site from a correct reference tree into a substrate story that explains *why*, *how*, and *how to extend*.

**Requirements:** DOCS-01, DOCS-02, DOCS-03, DOCS-04, DOCS-05, DOCS-06, DOCS-07, DOCS-08, DOCS-09 (now owned by Phases 40–45)

**Plans (NOT executed — superseded by Phases 40–45):**

- [ ] 30-01-PLAN.md — Wave 0: Vale + tooling install + CI integration + pre-commit hook + structure/screenshot/mermaid scripts (DOCS-09)
- [ ] 30-02-PLAN.md — Wave 0: Page skeletons (14 new pages + 2 .gitkeep) so Wave 1 nav doesn't dangle (DOCS-02, DOCS-03, DOCS-04, DOCS-05, DOCS-06)
- [ ] 30-03-PLAN.md — Wave 1: Hero rewrite of docs/index.md + mental-model + vs-mcp-sdks filled (DOCS-01, DOCS-03)
- [ ] 30-04-PLAN.md — Wave 1: mkdocs.yml nav restructure + theme tuning + social plugin (DOCS-07, DOCS-08)
- [ ] 30-05-PLAN.md — Wave 2: How-it-works carver (filesystem + git + trust-model) from architecture.md + security.md (DOCS-02)
- [ ] 30-06-PLAN.md — Wave 1: Tutorial author + end-to-end runner against simulator (DOCS-06)
- [ ] 30-07-PLAN.md — Wave 2: Guides (write-your-own-connector move + integrate-with-your-agent + troubleshooting) + reference/simulator fill (DOCS-04, DOCS-05)
- [ ] 30-08-PLAN.md — Wave 3: Grep-audit + delete architecture.md/security.md/demo.md/connectors/ + update README + clean mkdocs.yml not_in_nav (DOCS-07)
- [ ] 30-09-PLAN.md — Wave 4: Verification (mkdocs/vale/tutorial/structure) + 14 playwright screenshots + doc-clarity-review cold-reader + CHANGELOG v0.9.0 + SUMMARY (DOCS-01..09)

</details>

<details>
<summary>✅ v0.1.0–v0.7.0 (Phases 1–26) — SHIPPED 2026-04-13 through 2026-04-16</summary>

All details archived in `.planning/milestones/v0.8.0-ROADMAP.md`.

Summary:
- Phases 1–4, S: MVD simulator + FUSE + CLI + demo + write path + swarm
- Phase 8: GitHub read-only adapter + IssueBackend trait + contract tests
- Phase 11: Confluence Cloud read-only adapter + wiremock + ADR-002
- Phase 13: Nested mount layout (pages/ + tree/ symlinks)
- Phase 14: IssueBackend decoupling (FUSE write path + git-remote)
- Phase 15: Dynamic _INDEX.md in bucket dir (OP-2 partial)
- Phase 16: Confluence write path (create/update/delete + ADF↔Markdown)
- Phase 17: Swarm confluence-direct mode
- Phase 18: OP-2 remainder (tree-recursive + mount-root _INDEX.md)
- Phase 19: OP-1 remainder (labels/ symlink overlay)
- Phase 20: OP-3 (reposix refresh subcommand + git-diff cache)
- Phase 21: OP-7 hardening (contention swarm, truncation probe, chaos audit, macFUSE CI)
- Phase 22: OP-8 honest-tokenizer benchmarks
- Phase 23: OP-9a Confluence comments (pages/<id>.comments/)
- Phase 24: OP-9b Confluence whiteboards/attachments/folders
- Phase 25: OP-11 docs reorg (research/ migration)
- Phase 26: Docs clarity overhaul (doc-clarity-review, version sync)

</details>

<details>
<summary>✅ v0.8.0 JIRA Cloud Integration (Phases 27–29) — SHIPPED 2026-04-16</summary>

### Phase 27: Foundation — `IssueBackend` → `BackendConnector` rename + `Issue.extensions` field (v0.8.0)

**Goal:** Hard rename `IssueBackend` → `BackendConnector` across all crates + ADR-004. Add `Issue.extensions: BTreeMap<String, serde_yaml::Value>` for typed backend metadata.
**Plans:** 3/3 plans complete

- [x] 27-01-PLAN.md — IssueBackend → BackendConnector rename (SHIPPED)
- [x] 27-02-PLAN.md — BackendConnector rename propagation across workspace (SHIPPED)
- [x] 27-03-PLAN.md — Issue.extensions field + ADR-004 + v0.8.0 + CHANGELOG (SHIPPED)

### Phase 28: JIRA Cloud read-only adapter (`reposix-jira`) (v0.8.0)

**Goal:** First-class JIRA Cloud backend. JQL pagination, status-category mapping, subtask hierarchy, JIRA-specific extensions, CLI dispatch, contract tests, ADR-005, docs/reference/jira.md.
**Plans:** 3/3 plans complete

- [x] 28-01-PLAN.md — JiraBackend core adapter + 17 tests (SHIPPED)
- [x] 28-02-PLAN.md — CLI integration + contract tests (SHIPPED)
- [x] 28-03-PLAN.md — ADR-005 + jira.md + CHANGELOG v0.8.0 + tag script (SHIPPED)

### Phase 29: JIRA write path — `create_issue`, `update_issue`, `delete_or_close` via Transitions API (stretch) (v0.8.0)

**Goal:** Complete the JIRA write path. `create_issue` → POST, `update_issue` → PUT, `delete_or_close` → Transitions API with DELETE fallback. Audit log for all mutations.
**Requirements:** JIRA-06
**Plans:** 3/3 plans complete

- [x] 29-01-PLAN.md — ADF helpers + create_issue (SHIPPED)
- [x] 29-02-PLAN.md — update_issue + audit rows (SHIPPED)
- [x] 29-03-PLAN.md — delete_or_close transitions + contract test (SHIPPED)

</details>

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
