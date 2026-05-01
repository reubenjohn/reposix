# reposix — PROJECT history

Companion to `.planning/PROJECT.md`. Holds the validated-requirements log and the per-milestone narrative for everything that has shipped. Live scope (current milestone, active requirements, decisions, constraints) lives in `PROJECT.md`. Detailed retrospective lessons live in `RETROSPECTIVE.md`. Per-milestone phase plans live in `.planning/milestones/v*-phases/`.

This file exists so PROJECT.md stays at "what an agent needs in working memory every session." History is one click away when needed.

## Validated requirements

Shipped and confirmed valuable. Phase references point at `.planning/milestones/v*-phases/` archives.

- ✓ **RENAME-01: `IssueBackend` → `BackendConnector` trait rename** — Phase 27
- ✓ **EXT-01: `Issue.extensions` field** — Phase 27
- ✓ **JIRA-01: `reposix-jira` crate — read-only `BackendConnector` impl** — Phase 28
- ✓ **JIRA-02: JQL pagination + status-category mapping + subtask hierarchy** — Phase 28
- ✓ **JIRA-03: JIRA-specific `extensions` in frontmatter** — Phase 28
- ✓ **JIRA-04: CLI dispatch** — Phase 28
- ✓ **JIRA-05: Tests + docs + ADRs** — Phase 28
- ✓ **JIRA-06 (stretch): JIRA write path** — Phase 29. `create_issue` (POST), `update_issue` (PUT), `delete_or_close` (Transitions API + DELETE fallback). 31 unit tests + 5-arm contract test. Audit log for all mutations.
- ✓ **ARCH-01: `crates/reposix-cache/` builds bare git repo from REST** — Phase 31
- ✓ **ARCH-02: cache returns `Tainted<Vec<u8>>` + audit row per blob materialization + append-only triggers** — Phase 31
- ✓ **ARCH-03: cache enforces `REPOSIX_ALLOWED_ORIGINS` via single `reposix_core::http::client()` factory** — Phase 31
- ✓ **ARCH-04: `git-remote-reposix` advertises `stateless-connect` (hybrid with `export`)** — Phase 32
- ✓ **ARCH-05: three protocol-v2 framing gotchas covered + refspec namespace `refs/heads/*:refs/reposix/*`** — Phase 32
- ✓ **ARCH-06: `BackendConnector::list_changed_since` on all 4 backends + sim `?since=` REST param** — Phase 33
- ✓ **ARCH-07: atomic delta-sync transaction (cache + last_fetched_at + audit row in one tx)** — Phase 33
- ✓ **ARCH-08: push-time conflict detection (`error refs/heads/main fetch first`, cache untouched on reject)** — Phase 34
- ✓ **ARCH-09: blob-limit guardrail with verbatim `git sparse-checkout` teaching string + audit row** — Phase 34
- ✓ **ARCH-10: frontmatter field allowlist on push (`id`/`created_at`/`version`/`updated_at` stripped)** — Phase 34
- ✓ **ARCH-11: `reposix init <backend>::<project> <path>` replaces `reposix mount`** — Phase 35
- ✓ **ARCH-12: dark-factory pure-git agent UX validated against sim** — Phase 35 (real-backend exercise `pending-secrets`)
- ✓ **ARCH-13: `crates/reposix-fuse/` + `fuser` dependency + `fuse-mount-tests` feature gate purged** — Phase 36
- ✓ **ARCH-14: project `CLAUDE.md` rewritten to steady-state git-native architecture (no FUSE residue)** — Phase 36
- ✓ **ARCH-15: `.claude/skills/reposix-agent-flow/SKILL.md` ships with dark-factory regression invocation** — Phase 36
- ✓ **ARCH-16: real-backend smoke harness wired (TokenWorld, `reubenjohn/reposix`, JIRA `TEST`)** — Phase 35 (test infra; `pending-secrets`)
- ✓ **ARCH-17: golden-path latency captured + `docs/benchmarks/v0.9.0-latency.md` artifact** — Phase 35 (capture) + Phase 36 (artifact); sim column populated
- ✓ **ARCH-18: `docs/reference/testing-targets.md` documents the three sanctioned targets** — Phase 36
- ✓ **ARCH-19: three CI integration-contract-*-v09 jobs defined** — Phase 36 (`pending-secrets` until secrets decrypt)
- ✓ **DOCS-01: `docs/index.md` hero — V1 vignette + 3 measured numbers; ≤ 250 above-fold words** — Phase 40
- ✓ **DOCS-02: How-it-works trio (`filesystem-layer`, `git-layer`, `trust-model`); each one mermaid diagram** — Phase 41
- ✓ **DOCS-03: Two concept pages (mental-model-in-60-seconds, reposix-vs-mcp-and-sdks)** — Phase 40
- ✓ **DOCS-04: Three guides (write-your-own-connector, integrate-with-your-agent, troubleshooting)** — Phase 42
- ✓ **DOCS-05: Simulator relocated to `docs/reference/simulator.md`** — Phase 42
- ✓ **DOCS-06: 5-minute first-run tutorial verified by `scripts/tutorial-runner.sh`** — Phase 42
- ✓ **DOCS-07: MkDocs nav restructured per Diátaxis** — Phase 43
- ✓ **DOCS-08: mkdocs-material theme tuning + README hero rewritten** — Phase 43 (linter wiring) + Phase 45 (README)
- ✓ **DOCS-09: Banned-words linter `scripts/banned-words-lint.sh` + `docs/.banned-words.toml` + pre-commit + CI + `reposix-banned-words` skill** — Phase 43
- ✓ **DOCS-10: 16-page cold-reader clarity audit; zero critical friction points** — Phase 44 (2 critical fixed, 1 escalated to Phase 45 and closed there)
- ✓ **DOCS-11: README points at mkdocs site; CHANGELOG `[v0.10.0]` finalized** — Phase 45 (playwright screenshots deferred to v0.11.0 — cairo system libs unavailable on dev host)

## Historical: v0.1.0 MVD Active list

Shipped 2026-04-13; retained for traceability. Items prefixed `✗` were reversed by later milestones.

**Functional core (shipped v0.1.0)**
- ✓ Simulator-first architecture
- ✓ Issues as Markdown + YAML frontmatter
- ✗ FUSE mount with full read+write — REVERSED in v0.9.0 architecture pivot (FUSE deleted entirely; replaced by partial-clone working tree)
- ✓ `git-remote-reposix` helper (now hybrid `stateless-connect` + `export` per v0.9.0)
- ✗ `reposix sim`, `reposix mount`, `reposix demo` CLI — `reposix mount` deleted in v0.9.0; replaced by `reposix init <backend>::<project> <path>`
- ✓ Audit log of every network-touching action
- ✓ Adversarial swarm harness
- ✓ Working CI on GitHub Actions (FUSE-mount-in-runner removed v0.9.0)
- ✓ Demo-ready by 2026-04-13 morning

**Security guardrails (shipped v0.1.0 / v0.4.1 / v0.5.0; updated v0.9.0)**
- ✓ Outbound HTTP allowlist (`reposix_core::http::client()` factory — clippy `disallowed_methods` enforces single construction site)
- ✓ Bulk-delete cap on push
- ✓ Server-controlled frontmatter fields immutable from clients (`id`, `created_at`, `version`, `updated_at` stripped before egress)
- ✓ Filename derivation never uses titles
- ✓ Tainted-content typing (`reposix_core::Tainted<T>` along the data path; explicit `sanitize` step)
- ✓ Audit log append-only (SQLite WAL + `BEFORE UPDATE/DELETE RAISE` trigger)
- ✗ "FUSE never blocks the kernel forever" — N/A post-v0.9.0 (FUSE deleted)
- ✓ Demo recording shows guardrails firing

## Previously Validated Milestone: v0.12.x — Quality Gates + Carry-forwards (SHIPPED 2026-04-30)

v0.12.0 (Phases 56–65, SHIPPED 2026-04-28) replaced ad-hoc quality scripts with the dimension-tagged Quality Gates framework: `quality/` directory layout, unified catalog schema (`id`, `dimension`, `cadence`, `kind`, `verifier`, `artifact`, `status`, `freshness_ttl`, `waiver`, `blast_radius`, `owner_hint`), 9 dimensions homed (code, docs-alignment, docs-build, docs-repro, release, structure, agent-ux, perf, security), 6 cadences, 5 kinds. `end-state.py` migrated to a thin shim. Subjective gates skill (`reposix-quality-review`) ships with 30-day TTL freshness enforcement. Repo-org-gaps from v0.11.1 closed via structure-dim catalog rows. Catalog-first phase rule + verifier-subagent dispatch became the autonomous-mode protocol.

v0.12.1 (Phases 72–77, SHIPPED 2026-04-30) drained docs-alignment carry-forward: P72 lint-config invariants (8 verifiers, 9 MISSING_TEST rows closed); P73 connector contract gaps (4 wiremock-based contract tests for GitHub/Confluence auth headers + JIRA list_records rendering boundary); P74 narrative + UX cluster (4 propose-retires + 5 hash-shape binds + linkedin prose fix); P75 bind-verb hash-overwrite fix (`Source::Single` source_hash refresh, `Source::Multi` preservation; 3 walker regression tests); P76 surprises absorption (3 LOW intake entries drained); P77 good-to-haves polish (1 XS heading rename + verifier regex narrow). Owner-TTY close-out 2026-04-30: SSH config drift fixed; 27 RETIRE_PROPOSED rows confirmed via `--i-am-human`; pre-commit fmt hook installed; v0.12.0 tag pushed; 5 backlog items filed (999.2–999.6); milestone-close verdict ratified by unbiased subagent (verdict at `quality/reports/verdicts/milestone-v0.12.1/VERDICT.md`). OP-9 milestone-close ritual added; RETROSPECTIVE.md v0.12.1 section distilled.

**Carry-forward NOT closed by v0.12.x (rolling into v0.13.0):** `MULTI-SOURCE-WATCH-01` (walker watches only first source on Multi rows; path-(b) deferred). 5 backlog items filed (999.2–999.6) — push cadence resolved at v0.13.0 kickoff (RESOLVED 2026-04-30 → per-phase). RETROSPECTIVE.md backfill for v0.9.0 → v0.12.0 carries to v0.14.0.

## Previously Validated Milestone: v0.12.0 — Quality Gates (SHIPPED 2026-04-28)

**Goal:** Replace ad-hoc quality scripts with a coherent, dimension-tagged Quality Gates system that prevents the silent regressions the v0.11.x cycle missed (the curl installer URL became `Not Found` after release-plz cut over to per-crate tags; the existing `end-state.py` framework caught crates.io drift but not GitHub-release-asset drift). Ship the framework so any future change to docs, code, releases, or external integrations runs against a verifier that simulates a real user — not the agent's self-report.

**Mental model.** Each gate answers three orthogonal questions: **dimension** (code / docs-build / docs-repro / release / structure / perf / security / agent-ux), **cadence** (pre-push / pre-pr / weekly / pre-release / post-release / on-demand), **kind** (mechanical / container / asset-exists / subagent-graded / manual). Catalogs are the data; verifiers are the code; reports are the artifacts; runners compose gates by tag. Adding a new gate is one catalog row + one verifier in the right dimension dir — no bespoke script, no new pre-push wiring.

**Target features:** restore broken curl/PowerShell installer URLs + brew formula auto-update; `quality/` directory layout (`gates/<dim>/`, `catalogs/`, `reports/verifications/`, `runners/`) with `quality/PROTOCOL.md` runtime contract + `quality/SURPRISES.md` pivot journal; unified catalog schema generalizing `SESSION-END-STATE.json` + `docs_reproducible_catalog.json`; migrate structural rows out of `scripts/end-state.py` (becomes thin shim); release-dimension gates (installer URL HEAD, brew formula cross-check, crates.io max_version, post-release container rehearsal); docs-repro dimension (snippet extraction, container rehearsal harness); subjective gates skill (`reposix-quality-review`) with 30-day TTL; close out `.planning/research/v0.11.1/repo-organization-gaps.md`; autonomous-execution protocol (catalog-first phases, verifier-subagent grading, waivers with TTL).

**Phases (56–63):** see `.planning/ROADMAP.md`. P56 Restore release artifacts → P57 Quality Gates skeleton + structure dim → P58 Release dim → P59 Docs-repro dim → P60 Docs-build migration → P61 Subjective gates skill → P62 Repo-org-gaps cleanup → P63 Retire old + document.

**Source-of-truth handover bundle:** `.planning/research/v0.12.0/{vision-and-mental-model,naming-and-architecture,roadmap-and-rationale,autonomous-execution-protocol,install-regression-diagnosis,decisions-log,open-questions-and-deferrals}.md` + `.planning/docs_reproducible_catalog.json` (DRAFT seed for P59).

**Carry-forward from v0.11.x:** repo-organization-gaps cleanup → P62; curl-installer regression → P56+P58+P59; `Error::Other` 156→144 partial migration deferred from POLISH2-09 to v0.12.1.

## Previously Validated Milestone: v0.11.x — Polish & Reproducibility (SHIPPED 2026-04-27)

v0.11.0 (Phases 50–55) shipped 2026-04-25; v0.11.1 + v0.11.2 polish passes shipped 2026-04-26 / 2026-04-27 via release-plz. All eight crates published to crates.io at v0.11.2; mkdocs site live at `https://reubenjohn.github.io/reposix/`; pre-push hook runs `scripts/end-state.py verify` over 20 freshness + crates.io + mermaid claims (last verdict GREEN at session close). The cycle introduced the §0.8 SESSION-END-STATE framework that v0.12.0 generalizes into the Quality Gates system. **Carry-forward NOT closed by v0.11.x:** installer URLs broken on every release after v0.11.0 because `release.yml` tag glob `v*` does not match release-plz's per-crate `reposix-cli-v*` pattern — diagnosed in `.planning/research/v0.12.0/install-regression-diagnosis.md`; fixed by P56.

## Previously Validated Milestone: v0.10.0 — Docs & Narrative Shine (SHIPPED 2026-04-25)

**Goal:** Make the v0.9.0 architecture pivot legible. A cold visitor understands reposix's value proposition within 10 seconds of landing on the docs site, runs the 5-minute first-run tutorial against the simulator, and ends with a real edit committed and pushed via `reposix init` + standard git. The architecture pivot becomes a *story* (cache layer / git layer / trust model — three pages, each with one mermaid diagram rendered via mcp-mermaid and a playwright screenshot), not a code change.

**Target features (Phases 40–45 — see `.planning/ROADMAP.md` for the full breakdown, `.planning/research/v0.10.0-post-pivot/milestone-plan.md` for the design authority):**

- **Phase 40** — Hero rewrite of `docs/index.md` + two home-adjacent concept pages (`mental-model-in-60-seconds`, `reposix-vs-mcp-and-sdks`). Hero numbers sourced from `docs/benchmarks/v0.9.0-latency.md` (`8 ms` get-issue, `24 ms` `reposix init` cold, `9 ms` list, `5 ms` capabilities probe).
- **Phase 41** — How-it-works trio (`filesystem-layer`, `git-layer`, `trust-model`); each page one mermaid diagram + playwright screenshot, **reframed for git-native** (no FUSE / inode / daemon vocabulary above Layer 4).
- **Phase 42** — 5-minute first-run tutorial verified by `scripts/tutorial-runner.sh` (the doc IS the test). Three guides: write-your-own-connector, integrate-with-your-agent (pointer page; v0.12.0 ships full recipes), troubleshooting. Simulator relocated to Reference.
- **Phase 43** — MkDocs nav restructured per Diátaxis. mkdocs-material theme tuning. `scripts/banned-words-lint.sh` + `docs/.banned-words.toml` + `.claude/skills/reposix-banned-words/SKILL.md` ship together (institutional memory of P2 framing rules per OP-4, replacing ad-hoc grep).
- **Phase 44** — `doc-clarity-review` skill run as a release gate over every user-facing page (cold-reader scenario in isolation, OP-6); zero critical friction points required to ship.
- **Phase 45** — README hero rewrite (every adjective replaced with a measured number from `docs/benchmarks/v0.9.0-latency.md` or v0.9.0 audit/threat-model), CHANGELOG `[v0.10.0]`, social cards, playwright screenshots committed for landing + how-it-works + tutorial. `scripts/tag-v0.10.0.sh` mirrors v0.9.0 tag script (≥6 safety guards).

**Framing principles** (P1 complement-not-replace, P2 progressive-disclosure, numbers-not-adjectives, self-improving infra) are codified in `.planning/notes/phase-30-narrative-vignettes.md` and the `reposix-banned-words` skill — those remain canonical. **Source of truth:** `.planning/research/v0.10.0-post-pivot/milestone-plan.md` (design authority, drafted 2026-04-24). **Carry-forward from v0.9.0:** none (helper backend URL dispatch landed 2026-04-25 commit `cd1b0b6`; v0.9.0 audit verdict flipped `tech_debt` → `passed`).

## Previously Validated Milestone: v0.9.0 — Architecture Pivot — Git-Native Partial Clone (SHIPPED 2026-04-24)

**Goal:** Replace the FUSE virtual filesystem with git's built-in partial clone mechanism. The `git-remote-reposix` helper becomes a promisor remote tunnelling protocol-v2 traffic to a local bare-repo cache built from REST responses. Agents interact with the project using only standard git commands — no reposix-specific CLI awareness. FUSE is deleted entirely; `crates/reposix-fuse/` is removed and the `fuser` dependency is purged. The pivot is operationalized across six phases (31–36) covering cache foundation, read transport, delta sync, push path, CLI pivot, and the demolition+release cycle.

**Target features:**
- DELETE `crates/reposix-fuse/` entirely; drop `fuser` dependency; remove `fuse-mount-tests` feature gate; CI no longer needs `apt install fuse3` or `/dev/fuse`.
- ADD `crates/reposix-cache/` — a new crate that materializes REST API responses into a local bare git repo. Lazy blobs, full tree, audit row per blob materialization, `Tainted<Vec<u8>>` return type, egress allowlist enforced.
- ADD `stateless-connect` capability to `git-remote-reposix` (alongside existing `export`) — protocol-v2 tunnel from git into the cache. Refspec namespace `refs/heads/*:refs/reposix/*`. Three protocol gotchas from POC encoded correctly in Rust.
- ADD `BackendConnector::list_changed_since(timestamp)` trait method + per-backend implementations (sim `?since=`, GitHub `?since=`, JIRA JQL `updated >=`, Confluence CQL `lastModified >`). Delta sync transfers only changed issues.
- ADD push-time conflict detection inside the `export` handler — emits canned `error refs/heads/main fetch first` so agents experience standard `git pull --rebase, retry` UX. Reject path drains the stream and never touches the bare cache.
- ADD blob-limit guardrail — helper counts `want` lines per `command=fetch`, refuses if > `REPOSIX_BLOB_LIMIT` (default 200), with a stderr error message that names `git sparse-checkout` so unprompted agents self-correct (dark-factory pattern).
- ADD frontmatter field allowlist on the push path — server-controlled fields (`id`, `created_at`, `version`, `updated_at`) stripped before REST.
- CHANGE CLI: `reposix mount <path>` -> `reposix init <backend>::<project> <path>` (runs `git init`, configures `extensions.partialClone`, sets remote URL, runs `git fetch --filter=blob:none origin`). Breaking change documented in CHANGELOG.
- ADD project Claude Code skill `reposix-agent-flow` at `.claude/skills/reposix-agent-flow/SKILL.md` — encodes the dark-factory autonomous-agent regression test (subprocess agent given only `reposix init` + a goal completes the work via pure git/POSIX). Invoked from CI and local dev.
- UPDATE project `CLAUDE.md` in lockstep with FUSE deletion (Phase 36) — no window where grounding describes deleted code; FUSE references purged from elevator pitch, Operating Principles, Workspace layout, Tech stack, Commands, Threat model.

**Framing principles** (simulator-first, tainted-by-default, audit-log non-optional, no hidden state, working-tree = real git repo, self-improving infra) are codified as OPs in the project `CLAUDE.md` and remain in force post-v0.9.0. **Source of truth:** `.planning/research/v0.9-fuse-to-git-native/architecture-pivot-summary.md` (canonical design doc, 440 lines, ratified 2026-04-24). Supporting POC artifacts in `.planning/research/v0.9-fuse-to-git-native/poc/`.
