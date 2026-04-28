# v0.12.0 P65 — Docs-alignment backfill PUNCH-LIST

**Run dir:** `quality/reports/doc-alignment/backfill-20260428T085523Z/`
**Backfill date:** 2026-04-28
**Catalog:** `quality/catalogs/doc-alignment.json`

## Top-line numbers

- claims_total: **388**
- claims_bound: **181**
- claims_missing_test: **166**
- claims_retire_proposed: **41**
- alignment_ratio: **0.466** (floor 0.50; floor_waiver until 2026-07-31)

## How to read this list

Each cluster maps to a user-facing surface. **MISSING_TEST** rows are claims with no test asserting them — these are the gap-closure work for v0.12.1 (P71+). **RETIRE_PROPOSED** rows are claims an extractor flagged as superseded by a documented architecture decision; the human (`reuben`) confirms each retirement explicitly via `reposix-quality doc-alignment confirm-retire --row-id X` from a TTY (env-guarded against agent contexts).

**Important caveat on cluster sizes.** A few clusters are over-extracted: `Reference: glossary` shows 24 RETIRE_PROPOSED rows because the extractor treated each glossary term as a claim and proposed retiring them all. Treat such clusters as a bulk-confirm review item (the human can `confirm-retire` all 24 in one sitting), not 24 individual investigation tickets.

## Clusters by user-facing surface

### ADR: JIRA issue mapping

- BOUND: 2
- MISSING_TEST: 3
- RETIRE_PROPOSED: 0

<details><summary>3 MISSING_TEST</summary>

- **docs/decisions/005-jira-issue-mapping/adf-description-plain-text** — ADF description extraction produces plain text only, no Markdown conversion <br/>source: `docs/decisions/005-jira-issue-mapping.md:64-77`
- **docs/decisions/005-jira-issue-mapping/version-synthesis** — Issue version is synthesized from fields.updated as Unix milliseconds (u64) <br/>source: `docs/decisions/005-jira-issue-mapping.md:48-61`
- **docs/decisions/005-jira-issue-mapping/attachments-comments-excluded** — Phase 28 excludes attachments and comments; they are deferred to Phase 29+ <br/>source: `docs/decisions/005-jira-issue-mapping.md:79-87`

</details>

### ADR: helper backend dispatch

- BOUND: 0
- MISSING_TEST: 4
- RETIRE_PROPOSED: 0

<details><summary>4 MISSING_TEST</summary>

- **docs/decisions/008-helper-backend-dispatch/cache-slug-naming** — Cache slug format is <backend_slug>-<sanitized-project>.git; GitHub owner/repo sanitized to owner-repo <br/>source: `docs/decisions/008-helper-backend-dispatch.md:55-77`
- **docs/decisions/008-helper-backend-dispatch/url-scheme-dispatch** — Helper performs URL-scheme dispatch via parse_remote_url to instantiate matching BackendConnector <br/>source: `docs/decisions/008-helper-backend-dispatch.md:26-48`
- **docs/decisions/008-helper-backend-dispatch/credential-resolution** — Missing-creds errors list every absent env var and point to docs/reference/testing-targets.md <br/>source: `docs/decisions/008-helper-backend-dispatch.md:79-94`
- **docs/decisions/008-helper-backend-dispatch/audit-signal** — op=helper_backend_instantiated row appended to audit_events_cache with backend_slug, project_for_cache, project_for_backend <br/>source: `docs/decisions/008-helper-backend-dispatch.md:96-103`

</details>

### ADR: nested mount layout (FUSE-era)

- BOUND: 0
- MISSING_TEST: 0
- RETIRE_PROPOSED: 1

<details><summary>1 RETIRE_PROPOSED</summary>

- **docs/decisions/003-nested-mount-layout/fuse-architecture-retired-v0.9.0** — ADR-003 entire content (pages/, tree/, .gitignore, symlinks, slugify_title) SUPERSEDED by v0.9.0 git-native pivot <br/>source: `docs/decisions/003-nested-mount-layout.md:1-7`

</details>

### Archived REQUIREMENTS.md (v0.10.0-phases)

- BOUND: 3
- MISSING_TEST: 11
- RETIRE_PROPOSED: 2

<details><summary>11 MISSING_TEST</summary>

- **planning-milestones-v0-10-0-phases-REQUIREMENTS-md/cold-reader-16page-audit** — 16-page cold-reader clarity audit with zero critical friction points <br/>source: `.planning/milestones/v0.10.0-phases/REQUIREMENTS.md:22-22`
- **planning-milestones-v0-10-0-phases-REQUIREMENTS-md/tutorial-replay-5min** — 5-minute first-run tutorial verified by quality/gates/docs-repro/tutorial-replay.sh (7-step quickstart from docs/tutorials/first-run.md) <br/>source: `.planning/milestones/v0.10.0-phases/REQUIREMENTS.md:18-18`
- **planning-milestones-v0-10-0-phases-REQUIREMENTS-md/banned-words-linter** — Banned-word linter + skill enforces P1/P2 framing rules in Markdown docs <br/>source: `.planning/milestones/v0.10.0-phases/REQUIREMENTS.md:21-21`
- **planning-milestones-v0-10-0-phases-REQUIREMENTS-md/mkdocs-nav-diataxis-restructure** — MkDocs nav restructured per Diátaxis (tutorials, how-tos, references, explanations) <br/>source: `.planning/milestones/v0.10.0-phases/REQUIREMENTS.md:19-19`
- **planning-milestones-v0-10-0-phases-REQUIREMENTS-md/docs08-theme-readme-rewrite** — mkdocs-material theme tuning + README hero rewrite <br/>source: `.planning/milestones/v0.10.0-phases/REQUIREMENTS.md:20-20`
- **planning-milestones-v0-10-0-phases-REQUIREMENTS-md/docs01-value-prop-10sec** — Reader can understand reposix value proposition within 10 seconds of landing on docs/index.md <br/>source: `.planning/milestones/v0.10.0-phases/REQUIREMENTS.md:13-13`
- **planning-milestones-v0-10-0-phases-REQUIREMENTS-md/docs11-readme-mkdocs-changelog** — README points at mkdocs site; CHANGELOG [v0.10.0] entry exists <br/>source: `.planning/milestones/v0.10.0-phases/REQUIREMENTS.md:23-23`
- **planning-milestones-v0-10-0-phases-REQUIREMENTS-md/docs05-simulator-relocated** — Simulator relocated from 'How it works' to Reference (docs/reference/simulator.md) <br/>source: `.planning/milestones/v0.10.0-phases/REQUIREMENTS.md:17-17`
- **planning-milestones-v0-10-0-phases-REQUIREMENTS-md/docs04-three-guides** — Three guides (write-your-own-connector, integrate-with-your-agent, troubleshooting) <br/>source: `.planning/milestones/v0.10.0-phases/REQUIREMENTS.md:16-16`
- **planning-milestones-v0-10-0-phases-REQUIREMENTS-md/docs02-three-page-howitworks** — Three-page 'How it works' section (filesystem-layer, git-layer, trust-model) <br/>source: `.planning/milestones/v0.10.0-phases/REQUIREMENTS.md:14-14`
- **planning-milestones-v0-10-0-phases-REQUIREMENTS-md/docs03-two-concept-pages** — Two home-adjacent concept pages (mental-model-in-60-seconds, reposix-vs-mcp-and-sdks) <br/>source: `.planning/milestones/v0.10.0-phases/REQUIREMENTS.md:15-15`

</details>

<details><summary>2 RETIRE_PROPOSED</summary>

- **planning-milestones-v0-10-0-phases-REQUIREMENTS-md/helper-sim-backend-tech-debt-closed** — Helper-hardcodes-SimBackend tech debt closed in v0.11.0 <br/>source: `.planning/milestones/v0.10.0-phases/REQUIREMENTS.md:28-28`
- **planning-milestones-v0-10-0-phases-REQUIREMENTS-md/playwright-screenshots-deferred** — Playwright screenshots deferred to v0.11.0 due to cairo system libs unavailable on dev host (POLISH-11) <br/>source: `.planning/milestones/v0.10.0-phases/REQUIREMENTS.md:27-27`

</details>

### Archived REQUIREMENTS.md (v0.11.0-phases)

- BOUND: 6
- MISSING_TEST: 15
- RETIRE_PROPOSED: 0

<details><summary>15 MISSING_TEST</summary>

- **planning-milestones-v0-11-0-phases-REQUIREMENTS-md/polish-13-cli-dedup** — 4-way CLI duplication consolidated, parse_remote_url unified, cli_compat.rs deleted, FUSE residue stripped from refresh.rs <br/>source: `.planning/milestones/v0.11.0-phases/REQUIREMENTS.md:95-95`
- **planning-milestones-v0-11-0-phases-REQUIREMENTS-md/polish-03-mermaid-render** — All mermaid diagrams render without console errors on the live site <br/>source: `.planning/milestones/v0.11.0-phases/REQUIREMENTS.md:85-85`
- **planning-milestones-v0-11-0-phases-REQUIREMENTS-md/polish2-03-bench-cron** — bench-latency-cron Authorization-header duplicate fixed by setting persist-credentials: false <br/>source: `.planning/milestones/v0.11.0-phases/REQUIREMENTS.md:28-28`
- **planning-milestones-v0-11-0-phases-REQUIREMENTS-md/polish-07-binstall** — cargo binstall reposix-cli works with Cargo metadata for binstall configured <br/>source: `.planning/milestones/v0.11.0-phases/REQUIREMENTS.md:89-89`
- **planning-milestones-v0-11-0-phases-REQUIREMENTS-md/polish-17-docs-validation** — CLAUDE.md adds: any docs-site work MUST be playwright-validated; scripts/check-docs-site.sh exists and is wired into pre-push <br/>source: `.planning/milestones/v0.11.0-phases/REQUIREMENTS.md:96-96`
- **planning-milestones-v0-11-0-phases-REQUIREMENTS-md/polish2-06-landing** — Connector capability matrix added to landing page <br/>source: `.planning/milestones/v0.11.0-phases/REQUIREMENTS.md:31-31`
- **planning-milestones-v0-11-0-phases-REQUIREMENTS-md/polish2-10-confluence-split** — crates/reposix-confluence/src/lib.rs (3 989 LOC) split into types.rs / translate.rs / client.rs modules <br/>source: `.planning/milestones/v0.11.0-phases/REQUIREMENTS.md:35-35`
- **planning-milestones-v0-11-0-phases-REQUIREMENTS-md/polish2-11-jira-split** — crates/reposix-jira/src/lib.rs (1 957 LOC) split into matching modules <br/>source: `.planning/milestones/v0.11.0-phases/REQUIREMENTS.md:36-36`
- **planning-milestones-v0-11-0-phases-REQUIREMENTS-md/polish-02-glossary** — docs/reference/glossary.md exists; every other page links to it on first jargon term (≥24 entries) <br/>source: `.planning/milestones/v0.11.0-phases/REQUIREMENTS.md:84-84`
- **planning-milestones-v0-11-0-phases-REQUIREMENTS-md/polish2-09-typed-errors** — Error::Other(String) rewritten to typed Error::NotFound / Error::NotSupported / Error::VersionMismatch variants in reposix-core <br/>source: `.planning/milestones/v0.11.0-phases/REQUIREMENTS.md:34-34`
- **planning-milestones-v0-11-0-phases-REQUIREMENTS-md/polish-08-latency** — Latency table populated for sim + github (real-backend cells) with record counts and 3-sample medians <br/>source: `.planning/milestones/v0.11.0-phases/REQUIREMENTS.md:90-90`
- **planning-milestones-v0-11-0-phases-REQUIREMENTS-md/polish2-02-aarch64** — linux-aarch64-musl binary added to release.yml dist matrix <br/>source: `.planning/milestones/v0.11.0-phases/REQUIREMENTS.md:27-27`
- **planning-milestones-v0-11-0-phases-REQUIREMENTS-md/polish-04-mkdocs-strict** — mkdocs build --strict is green; ADR-008 in nav; blog post in not_in_nav; pymdownx.emoji configured <br/>source: `.planning/milestones/v0.11.0-phases/REQUIREMENTS.md:86-86`
- **planning-milestones-v0-11-0-phases-REQUIREMENTS-md/polish-06-binaries** — Pre-built binaries published to GitHub Releases for linux musl x86/arm64, macOS x86/arm64, windows msvc on every git tag <br/>source: `.planning/milestones/v0.11.0-phases/REQUIREMENTS.md:88-88`
- **planning-milestones-v0-11-0-phases-REQUIREMENTS-md/polish-05-repro-tutorial** — Tutorial reproducible from fresh clone — bash scripts/repro-quickstart.sh runs the 7-step tutorial and asserts each step passes <br/>source: `.planning/milestones/v0.11.0-phases/REQUIREMENTS.md:87-87`

</details>

### Archived REQUIREMENTS.md (v0.8.0-phases)

- BOUND: 5
- MISSING_TEST: 14
- RETIRE_PROPOSED: 11

<details><summary>14 MISSING_TEST</summary>

- **planning-milestones-v0-8-0-phases-REQUIREMENTS-md/docs-02** — AgenticEngineeringReference.md moved to docs/research/agentic-engineering-reference.md with redirect note <br/>source: `.planning/milestones/v0.8.0-phases/REQUIREMENTS.md:99-99`
- **planning-milestones-v0-8-0-phases-REQUIREMENTS-md/docs-03** — All cross-refs in CLAUDE.md, README.md, and planning docs updated to new paths <br/>source: `.planning/milestones/v0.8.0-phases/REQUIREMENTS.md:100-100`
- **planning-milestones-v0-8-0-phases-REQUIREMENTS-md/bench-01** — bench_token_economy.py uses client.messages.count_tokens() instead of len(text)/4; results cached in benchmarks/fixtures/*.tokens.json <br/>source: `.planning/milestones/v0.8.0-phases/REQUIREMENTS.md:81-81`
- **planning-milestones-v0-8-0-phases-REQUIREMENTS-md/conf-01** — cat mount/pages/<id>.comments/<comment-id>.md returns comment body in Markdown frontmatter format <br/>source: `.planning/milestones/v0.8.0-phases/REQUIREMENTS.md:87-87`
- **planning-milestones-v0-8-0-phases-REQUIREMENTS-md/bench-03** — Cold-mount time-to-first-ls matrix: 4 backends x [10, 100, 500] issues <br/>source: `.planning/milestones/v0.8.0-phases/REQUIREMENTS.md:83-83`
- **planning-milestones-v0-8-0-phases-REQUIREMENTS-md/conf-03** — Comments are read-only (no write path in this phase) <br/>source: `.planning/milestones/v0.8.0-phases/REQUIREMENTS.md:89-89`
- **planning-milestones-v0-8-0-phases-REQUIREMENTS-md/bench-04** — docs/why.md honest-framing section updated with real tokenization numbers <br/>source: `.planning/milestones/v0.8.0-phases/REQUIREMENTS.md:84-84`
- **planning-milestones-v0-8-0-phases-REQUIREMENTS-md/conf-06** — Folders (/folders endpoint) exposed as a separate tree alongside page hierarchy <br/>source: `.planning/milestones/v0.8.0-phases/REQUIREMENTS.md:95-95`
- **planning-milestones-v0-8-0-phases-REQUIREMENTS-md/docs-01** — InitialReport.md moved to docs/research/initial-report.md with redirect note at old path <br/>source: `.planning/milestones/v0.8.0-phases/REQUIREMENTS.md:98-98`
- **planning-milestones-v0-8-0-phases-REQUIREMENTS-md/conf-05** — ls mount/pages/<id>.attachments/ lists page attachments; binary passthrough <br/>source: `.planning/milestones/v0.8.0-phases/REQUIREMENTS.md:94-94`
- **planning-milestones-v0-8-0-phases-REQUIREMENTS-md/conf-02** — ls mount/pages/<id>.comments/ lists all inline + footer comments for that page <br/>source: `.planning/milestones/v0.8.0-phases/REQUIREMENTS.md:88-88`
- **planning-milestones-v0-8-0-phases-REQUIREMENTS-md/conf-04** — ls mount/whiteboards/ lists Confluence whiteboards; each exposed as <id>.json (raw) <br/>source: `.planning/milestones/v0.8.0-phases/REQUIREMENTS.md:93-93`
- **planning-milestones-v0-8-0-phases-REQUIREMENTS-md/bench-02** — Per-backend comparison table (sim, github, confluence) for token reduction vs raw JSON API <br/>source: `.planning/milestones/v0.8.0-phases/REQUIREMENTS.md:82-82`
- **planning-milestones-v0-8-0-phases-REQUIREMENTS-md/spaces-01** — reposix spaces --backend confluence subcommand lists all readable Confluence spaces in a table (KEY / NAME / URL) <br/>source: `.planning/milestones/v0.8.0-phases/REQUIREMENTS.md:90-90`

</details>

<details><summary>11 RETIRE_PROPOSED</summary>

- **planning-milestones-v0-8-0-phases-REQUIREMENTS-md/write-01** — Agent can create a new Confluence page by writing a new .md file in the FUSE mount (ConfluenceBackend::create_issue) <br/>source: `.planning/milestones/v0.8.0-phases/REQUIREMENTS.md:19-19`
- **planning-milestones-v0-8-0-phases-REQUIREMENTS-md/write-03** — Agent can delete/close a Confluence page by unlinking its .md file (ConfluenceBackend::delete_or_close) <br/>source: `.planning/milestones/v0.8.0-phases/REQUIREMENTS.md:21-21`
- **planning-milestones-v0-8-0-phases-REQUIREMENTS-md/write-02** — Agent can update a Confluence page by editing its .md file in the FUSE mount (ConfluenceBackend::update_issue) <br/>source: `.planning/milestones/v0.8.0-phases/REQUIREMENTS.md:20-20`
- **planning-milestones-v0-8-0-phases-REQUIREMENTS-md/index-02** — cat mount/_INDEX.md returns a whole-mount overview listing all backends, buckets, and top-level entry counts <br/>source: `.planning/milestones/v0.8.0-phases/REQUIREMENTS.md:30-30`
- **planning-milestones-v0-8-0-phases-REQUIREMENTS-md/index-01** — cat mount/tree/<subdir>/_INDEX.md returns a recursive markdown sitemap of that subtree, computed via cycle-safe DFS from TreeSnapshot <br/>source: `.planning/milestones/v0.8.0-phases/REQUIREMENTS.md:29-29`
- **planning-milestones-v0-8-0-phases-REQUIREMENTS-md/cache-02** — git diff HEAD~1 in the mount shows what changed at the backend since the last refresh (mount-as-time-machine) <br/>source: `.planning/milestones/v0.8.0-phases/REQUIREMENTS.md:38-38`
- **planning-milestones-v0-8-0-phases-REQUIREMENTS-md/nav-01** — ls mount/labels/<label>/ lists all issues/pages carrying that label as read-only symlinks pointing to the canonical file in the bucket <br/>source: `.planning/milestones/v0.8.0-phases/REQUIREMENTS.md:33-33`
- **planning-milestones-v0-8-0-phases-REQUIREMENTS-md/nav-02** — ls mount/spaces/<key>/ lists all pages in that Confluence space (multi-space mount support) <br/>source: `.planning/milestones/v0.8.0-phases/REQUIREMENTS.md:34-34`
- **planning-milestones-v0-8-0-phases-REQUIREMENTS-md/hard-04** — macFUSE parity: CI matrix entry for macOS with macFUSE, fusermount3 -> umount -f conditional swap <br/>source: `.planning/milestones/v0.8.0-phases/REQUIREMENTS.md:78-78`
- **planning-milestones-v0-8-0-phases-REQUIREMENTS-md/swarm-01** — reposix-swarm --mode confluence-direct exercises ConfluenceBackend directly (no FUSE overhead), mirroring SimDirectWorkload pattern <br/>source: `.planning/milestones/v0.8.0-phases/REQUIREMENTS.md:25-25`
- **planning-milestones-v0-8-0-phases-REQUIREMENTS-md/swarm-02** — Swarm run against confluence-direct produces summary metrics + audit-log rows, matching the sim-direct output format <br/>source: `.planning/milestones/v0.8.0-phases/REQUIREMENTS.md:26-26`

</details>

### Archived REQUIREMENTS.md (v0.9.0-phases)

- BOUND: 18
- MISSING_TEST: 1
- RETIRE_PROPOSED: 0

<details><summary>1 MISSING_TEST</summary>

- **planning-milestones-v0-9-0-phases-REQUIREMENTS-md/arch-19-ci-integration-contract** — CI integration-contract job for each real backend. Three CI jobs (integration-contract-confluence-v09, integration-contract-github-v09, integration-contract-jira-v09) run the ARCH-16 smoke suite when the corresponding secret block is present. <br/>source: `.planning/milestones/v0.9.0-phases/REQUIREMENTS.md:67-67`

</details>

### Benchmarks: headline numbers

- BOUND: 0
- MISSING_TEST: 20
- RETIRE_PROPOSED: 0

<details><summary>20 MISSING_TEST</summary>

- **docs/benchmarks/token-economy/confluence-reduction-76-percent** — Confluence raw-API fixture: 76.4% token reduction vs MCP baseline <br/>source: `docs/benchmarks/token-economy.md:23-27`
- **docs/benchmarks/latency/github-get-under-300ms** — Get one record: 291 ms for GitHub backend <br/>source: `docs/benchmarks/latency.md:34-45`
- **docs/benchmarks/latency/sim-get-under-10ms** — Get one record: 8 ms for sim backend <br/>source: `docs/benchmarks/latency.md:34-45`
- **docs/benchmarks/token-economy/github-reduction-85-percent** — GitHub raw-API fixture: 85.5% token reduction vs MCP baseline <br/>source: `docs/benchmarks/token-economy.md:23-27`
- **docs/benchmarks/latency/capabilities-probe-under-10ms** — Helper capabilities probe: ~5-6 ms (local-only, backend-independent) <br/>source: `docs/benchmarks/latency.md:34-45`
- **docs/benchmarks/token-economy/jira-real-adapter-not-implemented** — JIRA real adapter: N/A (adapter not yet implemented) <br/>source: `docs/benchmarks/token-economy.md:23-28`
- **docs/benchmarks/latency/jira-list-under-400ms** — List records: 332 ms (N=0) for JIRA backend <br/>source: `docs/benchmarks/latency.md:34-45`
- **docs/benchmarks/latency/github-list-under-500ms** — List records: 451 ms (N=18) for GitHub backend <br/>source: `docs/benchmarks/latency.md:34-45`
- **docs/benchmarks/latency/sim-list-under-15ms** — List records: 9 ms (N=6) for sim backend <br/>source: `docs/benchmarks/latency.md:34-45`
- **docs/benchmarks/token-economy/mcp-mediated-baseline-4883-tokens** — MCP-mediated baseline: 16,274 characters, 4,883 tokens for read-3-edit-1-push task <br/>source: `docs/benchmarks/token-economy.md:13-19`
- **docs/benchmarks/latency/sim-patch-under-20ms** — PATCH record (no-op): 11 ms for sim backend <br/>source: `docs/benchmarks/latency.md:34-45`
- **docs/benchmarks/latency/github-patch-under-900ms** — PATCH record (no-op): 828 ms for GitHub backend <br/>source: `docs/benchmarks/latency.md:34-45`
- **docs/benchmarks/token-economy/reposix-baseline-531-tokens** — reposix (shell session): 1,372 characters, 531 tokens for same task <br/>source: `docs/benchmarks/token-economy.md:13-19`
- **docs/benchmarks/latency/jira-init-under-30ms** — reposix init cold: 25 ms for JIRA backend <br/>source: `docs/benchmarks/latency.md:34-45`
- **docs/benchmarks/latency/github-init-under-30ms** — reposix init cold: 26 ms for GitHub backend <br/>source: `docs/benchmarks/latency.md:34-45`
- **docs/benchmarks/latency/sim-init-cold-under-30ms** — reposix init cold: 27 ms for sim backend <br/>source: `docs/benchmarks/latency.md:34-45`
- **docs/benchmarks/latency/confluence-init-under-600ms** — reposix init cold: 508 ms for Confluence backend <br/>source: `docs/benchmarks/latency.md:34-45`
- **docs/benchmarks/token-economy/reduction-89-percent** — reposix uses 89.1% fewer tokens than MCP baseline for same task (9.2x more context for MCP) <br/>source: `docs/benchmarks/token-economy.md:17-19`
- **docs/benchmarks/latency/soft-threshold-real-backend-under-3s** — Soft threshold: real-backend step < 3s (regression-flagged, not CI-blocking) <br/>source: `docs/benchmarks/latency.md:60-65`
- **docs/benchmarks/latency/soft-threshold-sim-init-under-500ms** — Soft threshold: sim cold init < 500ms (regression-flagged, not CI-blocking) <br/>source: `docs/benchmarks/latency.md:60-65`

</details>

### Concepts (mental model + positioning)

- BOUND: 0
- MISSING_TEST: 14
- RETIRE_PROPOSED: 0

<details><summary>14 MISSING_TEST</summary>

- **use-case-20-percent-rest-mcp** — 20% of agent operations should use REST/MCP for complex JQL, bulk imports, admin, reporting <br/>source: `docs/concepts/reposix-vs-mcp-and-sdks.md:49-54`
- **mcp-fixture-synthesized-not-live** — 4,883 tokens baseline is against synthesized MCP fixture modeled on Atlassian Forge, not live MCP server <br/>source: `docs/concepts/reposix-vs-mcp-and-sdks.md:27-32`
- **use-case-80-percent-routine-ops** — 80% of agent operations: status changes, comments, field edits, label adds, link creation absorbed by reposix <br/>source: `docs/concepts/reposix-vs-mcp-and-sdks.md:42-44`
- **docs/why/cached-read-8ms** — Cached read is 8ms <br/>source: `docs/concepts/mental-model-in-60-seconds.md:21-21`
- **frontmatter-yaml-schema** — Each issue is one Markdown file with YAML frontmatter; custom fields are additional YAML keys <br/>source: `docs/concepts/mental-model-in-60-seconds.md:23-41`
- **git-push-conflict-detection** — git push IS the sync verb; helper parses commits, checks backend version, applies writes or rejects with standard git fetch-first error <br/>source: `docs/concepts/mental-model-in-60-seconds.md:43-45`
- **token-baseline-mcp-4883** — MCP (synthesized fixture) = 4,883 tokens vs reposix 531 tokens = 89.1% reduction <br/>source: `docs/concepts/reposix-vs-mcp-and-sdks.md:25-25`
- **mcp-schema-discovery-100k-tokens** — MCP requires ~100k schema discovery tokens per server <br/>source: `docs/concepts/reposix-vs-mcp-and-sdks.md:13-13`
- **audit-trail-sqlite-git-log** — reposix audit = append-only SQLite + git log (both committed-or-fixture artifacts) <br/>source: `docs/concepts/reposix-vs-mcp-and-sdks.md:19-19`
- **latency-cached-read-8ms** — reposix cached read = 8 ms (sim) <br/>source: `docs/concepts/reposix-vs-mcp-and-sdks.md:14-14`
- **bootstrap-timing-24ms-vs-27ms** — reposix init bootstrap takes 24 ms against simulator <br/>source: `docs/concepts/mental-model-in-60-seconds.md:21-21`
- **docs/why/cold-init-24ms-sim** — reposix init cold takes 24ms against simulator <br/>source: `docs/concepts/mental-model-in-60-seconds.md:21-21`
- **token-baseline-reposix-531** — reposix loop = 531 tokens measured against shell session transcript on simulator <br/>source: `docs/concepts/reposix-vs-mcp-and-sdks.md:34-38`
- **conflict-semantics-native-git** — reposix uses native git merge conflicts; conflict recovery is standard git pull --rebase && git push <br/>source: `docs/concepts/reposix-vs-mcp-and-sdks.md:16-16`

</details>

### Developer workflow + invariants

- BOUND: 0
- MISSING_TEST: 17
- RETIRE_PROPOSED: 0

<details><summary>17 MISSING_TEST</summary>

- **docs-development-contributing-md/forbid-unsafe-per-crate** — Invariant row 1: #![forbid(unsafe_code)] at every crate root (except POSIX syscalls gated through rustix) <br/>source: `docs/development/contributing.md:59-59`
- **docs-development-contributing-md/reqwest-client-banned** — Invariant row 2: reqwest::Client is banned outside crates/reposix-core/src/http.rs via clippy disallowed-methods <br/>source: `docs/development/contributing.md:60-60`
- **docs-development-contributing-md/errors-doc-section-required** — Invariant row 3: Every public Result-returning function has an # Errors doc section (clippy missing-errors-doc is on) <br/>source: `docs/development/contributing.md:61-61`
- **docs-development-contributing-md/serde-no-handroll-json** — Invariant row 4: No hand-rolled JSON/YAML serialization, always use serde and serde_yaml for frontmatter <br/>source: `docs/development/contributing.md:62-62`
- **docs-development-contributing-md/clippy-pedantic-targeted-allows** — Invariant row 5: Clippy is pedantic with targeted allows only; blanket #[allow(clippy::pedantic)] is code-review failure <br/>source: `docs/development/contributing.md:63-63`
- **docs-development-contributing-md/rust-stable-no-nightly** — Invariant row 6: Workspace compiles on Rust stable; no nightly features; rust-toolchain.toml pins stable <br/>source: `docs/development/contributing.md:64-64`
- **docs-development-contributing-md/http-allowlist-enforcement** — Invariant row 7: Every outbound HTTP request passes the allowlist via sealed HttpClient newtype; per-request URL recheck catches redirects <br/>source: `docs/development/contributing.md:65-65`
- **docs-development-contributing-md/sanitize-server-fields-on-egress** — Invariant row 8: Server-controlled frontmatter fields are stripped on egress via sanitize(Tainted<Issue>, ServerMetadata) -> Untainted<Issue> <br/>source: `docs/development/contributing.md:66-66`
- **docs-development-contributing-md/cargo-check-workspace-available** — Quickstart: cargo check --workspace is available and used for fast type-check during development <br/>source: `docs/development/contributing.md:15-15`
- **docs-development-contributing-md/cargo-test-133-tests** — Quickstart: cargo test --workspace runs 133 tests (one #[ignore]'d timeout test runs via --ignored) <br/>source: `docs/development/contributing.md:20-20`
- **docs-development-contributing-md/demo-script-exists** — Quickstart: demo.sh script exists and runs end-to-end demo <br/>source: `docs/development/contributing.md:28-30`
- **docs-development-contributing-md/git-version-2-34-required** — Reporting bugs: git >= 2.34 required for partial-clone + stateless-connect <br/>source: `docs/development/contributing.md:130-130`
- **docs-development-roadmap-md/v0-1-0-shipped** — v0.1.0 MVD shipped 2026-04-13 with Simulator, IssueBackend trait, FUSE read-only mount, git-remote-reposix, eight security guardrails <br/>source: `docs/development/roadmap.md:9-9`
- **docs-development-roadmap-md/v0-10-0-shipped** — v0.10.0 shipped 2026-04-25: docs and narrative shine — landing page, tutorial set, mermaid diagrams, value-prop framing <br/>source: `docs/development/roadmap.md:18-18`
- **docs-development-roadmap-md/v0-11-0-active-milestone** — v0.11.0 is active milestone (PLANNING): Polish & Reproducibility — jargon glosses, mermaid render hygiene, fresh-clone tutorial runner, dist release pipeline, latency table, vision-innovations surface <br/>source: `docs/development/roadmap.md:22-22`
- **docs-development-roadmap-md/v0-2-0-alpha-shipped** — v0.2.0-alpha shipped 2026-04-13 with GitHub Issues read-only adapter <br/>source: `docs/development/roadmap.md:10-10`
- **docs-development-roadmap-md/v0-9-0-pivot-shipped** — v0.9.0 shipped 2026-04-24: Architecture pivot to git-native partial clone; FUSE mount retired; git-remote-reposix now advertises stateless-connect + export against on-disk cache; agent UX is upstream git <br/>source: `docs/development/roadmap.md:17-17`

</details>

### Other

- BOUND: 1
- MISSING_TEST: 10
- RETIRE_PROPOSED: 0

<details><summary>10 MISSING_TEST</summary>

- **docs/connectors/guide/trait-method-count-eight** — BackendConnector trait has 8 main methods (name, supports, list_records, list_changed_since, get_record, create_record, update_record, delete_or_close) <br/>source: `crates/reposix-core/src/backend.rs:216-331`
- **docs/connectors/guide/backendconnector-create-record-method** — BackendConnector trait has async fn create_record(&self, project, issue: Untainted<Record>) -> Result<Record> <br/>source: `crates/reposix-core/src/backend.rs:284-284`
- **docs/connectors/guide/backendconnector-delete-or-close-method** — BackendConnector trait has async fn delete_or_close(&self, project, id, reason: DeleteReason) -> Result<()> <br/>source: `crates/reposix-core/src/backend.rs:315-320`
- **docs/connectors/guide/backendconnector-get-record-method** — BackendConnector trait has async fn get_record(&self, project, id) -> Result<Record> <br/>source: `crates/reposix-core/src/backend.rs:272-272`
- **docs/connectors/guide/backendconnector-list-changed-since-method** — BackendConnector trait has async fn list_changed_since(&self, project: &str, since: DateTime<Utc>) -> Result<Vec<RecordId>> <br/>source: `crates/reposix-core/src/backend.rs:253-257`
- **docs/connectors/guide/backendconnector-list-records-method** — BackendConnector trait has async fn list_records(&self, project: &str) -> Result<Vec<Record>> <br/>source: `crates/reposix-core/src/backend.rs:235-235`
- **docs/connectors/guide/backendconnector-update-record-method** — BackendConnector trait has async fn update_record(&self, project, id, patch: Untainted<Record>, expected_version: Option<u64>) -> Result<Record> <br/>source: `crates/reposix-core/src/backend.rs:299-305`
- **docs/connectors/guide/backendconnector-name-method** — BackendConnector trait has fn name(&self) -> &'static str <br/>source: `crates/reposix-core/src/backend.rs:219-219`
- **docs/connectors/guide/backendconnector-root-collection-name-method** — BackendConnector trait has fn root_collection_name(&self) -> &'static str with default "issues" <br/>source: `crates/reposix-core/src/backend.rs:328-330`
- **docs/connectors/guide/backendconnector-supports-method** — BackendConnector trait has fn supports(&self, feature: BackendFeature) -> bool <br/>source: `crates/reposix-core/src/backend.rs:227-227`

</details>

### README headline + install + dark-factory loop

- BOUND: 9
- MISSING_TEST: 8
- RETIRE_PROPOSED: 0

<details><summary>8 MISSING_TEST</summary>

- **README-md/rust-1-82-requirement** — Build from source requires Rust stable 1.82+ <br/>source: `README.md:79-79`
- **README-md/clippy-clean** — cargo clippy --workspace --all-targets with -D warnings is clean <br/>source: `README.md:109-109`
- **README-md/tests-green** — cargo test --workspace passes <br/>source: `README.md:109-109`
- **README-md/forbid-unsafe-code** — Every crate declares #![forbid(unsafe_code)] <br/>source: `README.md:109-109`
- **README-md/token-89-percent** — Input context token reduction is 89.1% vs MCP-tool-catalog baseline <br/>source: `README.md:25-25`
- **README-md/latency-8ms** — Read one issue from local cache takes 8ms (measured on simulator) <br/>source: `README.md:23-23`
- **README-md/init-24ms** — reposix init cold bootstrap against simulator takes 24ms <br/>source: `README.md:24-24`
- **README-md/git-2-34-requirement** — Runtime requires git >= 2.34 for extensions.partialClone and stateless-connect <br/>source: `README.md:79-79`

</details>

### Reference: confluence

- BOUND: 0
- MISSING_TEST: 3
- RETIRE_PROPOSED: 0

<details><summary>3 MISSING_TEST</summary>

- **confluence.md/v0.4_write_path_claim** — 'v0.4 will add the write path' — claim is outdated, write path ships in phases 22/24 not v0.4 <br/>source: `docs/reference/confluence.md:152-154`
- **confluence.md/fuse_mount_symlink_tree** — FUSE mount layout (v0.4+) produces pages/, tree/ (symlink hierarchy), and .gitignore <br/>source: `docs/reference/confluence.md:110-128`
- **confluence.md/fuse_daemon_role** — IssueBackend trait consumed by FUSE daemon and reposix list CLI <br/>source: `docs/reference/confluence.md:6-8`

</details>

### Reference: exit-codes

- BOUND: 4
- MISSING_TEST: 3
- RETIRE_PROPOSED: 0

<details><summary>3 MISSING_TEST</summary>

- **exit-codes/cli-exit-0-success** — All reposix subcommands exit with 0 on success <br/>source: `docs/reference/exit-codes.md:24-25`
- **exit-codes/cli-exit-1-error** — All reposix subcommands exit with 1 on any handled error <br/>source: `docs/reference/exit-codes.md:8-8`
- **exit-codes/doctor-error-findings-exit-1** — reposix doctor exits 1 if at least one ERROR-severity finding <br/>source: `docs/reference/exit-codes.md:27-27`

</details>

### Reference: glossary

- BOUND: 0
- MISSING_TEST: 0
- RETIRE_PROPOSED: 24

<details><summary>24 RETIRE_PROPOSED</summary>

- **glossary/audit-log-definition** — audit log is append-only audit_events_cache table with BEFORE UPDATE/DELETE triggers <br/>source: `docs/reference/glossary.md:152-152`
- **glossary/backendconnector-definition** — BackendConnector is trait that every adapter implements with list_records, create_record, update_record, delete_or_close methods <br/>source: `docs/reference/glossary.md:184-184`
- **glossary/bare-repo-definition** — bare repo is git repository without working tree <br/>source: `docs/reference/glossary.md:84-84`
- **glossary/capability-advertisement-definition** — capability advertisement is first thing git remote helper writes to stdout <br/>source: `docs/reference/glossary.md:100-100`
- **glossary/dark-factory-definition** — dark-factory pattern: ship code with no human review via simulators, swarms, self-teaching errors <br/>source: `docs/reference/glossary.md:216-216`
- **glossary/egress-allowlist-definition** — egress allowlist is single choke-point via REPOSIX_ALLOWED_ORIGINS, all HTTP through reposix_core::http::client() <br/>source: `docs/reference/glossary.md:200-200`
- **glossary/extensions-partialClone-definition** — extensions.partialClone tells git this remote is a promisor for lazy object delivery <br/>source: `docs/reference/glossary.md:92-92`
- **glossary/fast-export-definition** — fast-export emits commits and tree changes as stream <br/>source: `docs/reference/glossary.md:60-60`
- **glossary/fast-import-definition** — fast-import streams commits, trees, blobs as single text stream <br/>source: `docs/reference/glossary.md:52-52`
- **glossary/frontmatter-definition** — frontmatter is YAML block at top of Markdown file, delimited by --- lines <br/>source: `docs/reference/glossary.md:172-172`
- **glossary/git-remote-helper-definition** — git remote helper is out-of-process binary git invokes for non-native URL schemes <br/>source: `docs/reference/glossary.md:116-116`
- **glossary/gix-definition** — gix is pure-Rust git implementation, pinned =0.82 because pre-1.0 <br/>source: `docs/reference/glossary.md:136-136`
- **glossary/lethal-trifecta-definition** — lethal trifecta is: private data + untrusted input + exfiltration channel <br/>source: `docs/reference/glossary.md:208-208`
- **glossary/partial-clone-definition** — partial clone fetches tree but skips blob contents until read, using --filter=blob:none <br/>source: `docs/reference/glossary.md:20-24`
- **glossary/pkt-line-definition** — pkt-line is protocol-v2 framing: 4-byte hex length prefix plus payload <br/>source: `docs/reference/glossary.md:108-108`
- **glossary/promisor-remote-definition** — promisor remote promises to deliver missing objects on request <br/>source: `docs/reference/glossary.md:36-36`
- **glossary/protocol-v2-definition** — protocol-v2 is current git wire protocol (default since git 2.26) <br/>source: `docs/reference/glossary.md:68-68`
- **glossary/push-round-trip-definition** — push round-trip sequences from git push to backend write to confirmation <br/>source: `docs/reference/glossary.md:124-124`
- **glossary/refspec-definition** — refspec is <src>:<dst> mapping telling git which refs go where <br/>source: `docs/reference/glossary.md:76-76`
- **glossary/sparse-checkout-definition** — sparse-checkout materializes only a subset of paths in working tree <br/>source: `docs/reference/glossary.md:28-28`
- **glossary/sqlite-wal-definition** — SQLite WAL writes go to separate file, merged on checkpoint <br/>source: `docs/reference/glossary.md:144-144`
- **glossary/stateless-connect-definition** — stateless-connect is capability allowing git to tunnel protocol-v2 traffic to helper <br/>source: `docs/reference/glossary.md:44-44`
- **glossary/tainted-definition** — Tainted<T> is newtype wrapper around bytes from remote, requiring sanitize() before egress sink <br/>source: `docs/reference/glossary.md:192-192`
- **glossary/yaml-definition** — YAML is human-friendly serialization format for nested key-value pairs <br/>source: `docs/reference/glossary.md:164-164`

</details>

### Reference: jira

- BOUND: 1
- MISSING_TEST: 0
- RETIRE_PROPOSED: 1

<details><summary>1 RETIRE_PROPOSED</summary>

- **docs/reference/jira.md/read-only-phase-28** — In Phase 28, create_record, update_record, and delete_or_close return not supported <br/>source: `docs/reference/jira.md:96-99`

</details>

### Social posts (marketing copy)

- BOUND: 0
- MISSING_TEST: 2
- RETIRE_PROPOSED: 0

<details><summary>2 MISSING_TEST</summary>

- **docs/social/twitter/token-reduction-92pct** — reposix achieves ~92% fewer tokens than MCP-mediated baseline in token-economy benchmark <br/>source: `docs/social/twitter.md:18-18`
- **docs/social/linkedin/token-reduction-92pct** — reposix used ~92% fewer tokens than MCP-mediated baseline in benchmark (simulated 35-tool Jira) <br/>source: `docs/social/linkedin.md:21-21`

</details>

### Tutorials (first-run flow)

- BOUND: 1
- MISSING_TEST: 6
- RETIRE_PROPOSED: 0

<details><summary>6 MISSING_TEST</summary>

- **docs/tutorials/first-run/git-diff-edit** — After editing (append comment + sed status change), git diff shows both hunks <br/>source: `docs/tutorials/first-run.md:111-125`
- **docs/tutorials/first-run/cat-issue-structure** — cat issues/0001.md returns frontmatter (id, title, status, assignee, labels, version) + body markdown <br/>source: `docs/tutorials/first-run.md:96-107`
- **docs/tutorials/first-run/checkout-origin** — git checkout -B main refs/reposix/origin/main succeeds after reposix init <br/>source: `docs/tutorials/first-run.md:76-88`
- **docs/tutorials/first-run/git-push-roundtrip** — git push succeeds with output To reposix::..., [new branch] main -> main <br/>source: `docs/tutorials/first-run.md:142-154`
- **docs/tutorials/first-run/ls-issues** — ls issues/ in initialized working tree lists exactly [0001.md 0002.md 0003.md 0004.md 0005.md] <br/>source: `docs/tutorials/first-run.md:90-94`
- **docs/tutorials/first-run/audit-query** — sqlite3 ~/.cache/reposix/sim-demo.git/cache.db SELECT ts, op, decision FROM audit_events_cache WHERE op LIKE 'helper_push_%' returns rows <br/>source: `docs/tutorials/first-run.md:156-166`

</details>

### User guides (integration, troubleshooting, write-connector)

- BOUND: 13
- MISSING_TEST: 15
- RETIRE_PROPOSED: 0

<details><summary>15 MISSING_TEST</summary>

- **docs/connectors/guide/auth-header-exact-test** — Auth header is byte-exact (Bearer prefix, Basic + base64, etc.) <br/>source: `docs/guides/write-your-own-connector.md:158-158`
- **docs/connectors/guide/clippy-disallowed-methods-lint** — clippy disallowed-methods lint rejects direct reqwest::Client::new() and Client::builder() <br/>source: `docs/guides/write-your-own-connector.md:149-149`
- **docs/connectors/guide/http-client-factory-required** — Connectors must use reposix_core::http::client() factory for all HTTP operations (no direct reqwest::Client) <br/>source: `docs/guides/write-your-own-connector.md:142-146`
- **docs/connectors/guide/wiremock-five-tests-required** — Each connector must publish ≥5 wiremock tests covering: happy path, 404, auth header, pagination, rate-limit <br/>source: `docs/guides/write-your-own-connector.md:154-162`
- **docs/connectors/guide/egress-allowlist-honors-reposix-allowed-origins** — Egress allowlist honors REPOSIX_ALLOWED_ORIGINS env var; client factory refuses non-allowlisted origins <br/>source: `docs/guides/write-your-own-connector.md:145-145`
- **docs/connectors/guide/contract-test-required** — Every connector publishes a contract test in tests/contract.rs running fixed invariant set against both SimBackend and connector via wiremock <br/>source: `docs/guides/write-your-own-connector.md:164-164`
- **docs/connectors/guide/get-record-404-test** — get_record 404 returns Error::Other whose message starts with 'not found: ' <br/>source: `docs/guides/write-your-own-connector.md:157-157`
- **docs/connectors/guide/list-records-happy-path-test** — list_records test returns ≥1 issue on happy path, asserts length and first row <br/>source: `docs/guides/write-your-own-connector.md:156-156`
- **docs/connectors/guide/pagination-cursor-test** — Pagination cursor is followed correctly; 2-page test asserts second request URL matches first response <br/>source: `docs/guides/write-your-own-connector.md:159-159`
- **docs/connectors/guide/rate-limit-429-test** — Rate-limit gate arms on 429 / Retry-After header <br/>source: `docs/guides/write-your-own-connector.md:160-160`
- **docs/connectors/guide/read-only-backends-err-not-supported** — Read-only backends return Error::Other('not supported: ...') from write methods and false from supports() queries <br/>source: `docs/guides/write-your-own-connector.md:35-35`
- **docs/connectors/guide/error-contract-not-found-prefix** — Read-path not-found errors return Error::Other with message starting with 'not found: ' <br/>source: `docs/guides/write-your-own-connector.md:29-29`
- **docs/connectors/guide/real-backend-smoke-fixture** — Real-backend smoke fixture lands behind #[ignore] with env-var setup docs (GITHUB_TOKEN, ATLASSIAN_API_KEY, etc.) <br/>source: `docs/guides/write-your-own-connector.md:172-173`
- **docs/guides/troubleshooting/blob-limit-env-override** — REPOSIX_BLOB_LIMIT env var can be set to override the default blob limit (200) for a single operation <br/>source: `docs/guides/troubleshooting.md:71-75`
- **docs/connectors/guide/error-contract-not-supported-prefix** — Write-path not-supported errors return Error::Other with message starting with 'not supported: ' <br/>source: `docs/guides/write-your-own-connector.md:35-35`

</details>

### docs/architecture.md

- BOUND: 0
- MISSING_TEST: 0
- RETIRE_PROPOSED: 1

<details><summary>1 RETIRE_PROPOSED</summary>

- **docs/architecture/redirect** — Architecture page redirects to How it works (filesystem-layer, git-layer, trust-model) <br/>source: `docs/architecture.md:7-12`

</details>

### docs/demo.md

- BOUND: 0
- MISSING_TEST: 0
- RETIRE_PROPOSED: 1

<details><summary>1 RETIRE_PROPOSED</summary>

- **docs/demo/redirect** — Demo page redirects to tutorials/first-run.md <br/>source: `docs/demo.md:7-9`

</details>

### docs/index.md

- BOUND: 8
- MISSING_TEST: 20
- RETIRE_PROPOSED: 0

<details><summary>20 MISSING_TEST</summary>

- **docs/index/latency-24ms-cold-init** — 24 ms cold init against simulator <br/>source: `docs/index.md:18-18`
- **docs/index/5-line-install** — 5-line install path exists <br/>source: `docs/index.md:19-19`
- **docs/index/latency-8ms-read** — 8 ms cached read against simulator <br/>source: `docs/index.md:18-18`
- **docs/index/token-reduction-89-percent** — 89.1% fewer tokens vs MCP for same 3-issue read+edit+push workflow <br/>source: `docs/index.md:17-17`
- **docs/index/tested-three-backends** — Architecture tested end-to-end against Confluence TokenWorld, GitHub reubenjohn/reposix, JIRA TEST project <br/>source: `docs/index.md:86-91`
- **docs/index/audit-trail-git-log** — Audit trail is git log (no SDK vendoring required) <br/>source: `docs/index.md:78-78`
- **docs/index/backend-capabilities-struct** — BackendCapabilities struct exists in crates/reposix-core/src/backend.rs <br/>source: `docs/index.md:113-113`
- **docs/index/bootstrap-latency-24ms** — Bootstrap takes ≤ 24 ms against the simulator <br/>source: `docs/index.md:129-129`
- **docs/index/install-brew** — brew install reubenjohn/reposix/reposix works <br/>source: `docs/index.md:59-59`
- **docs/index/install-cargo-binstall** — cargo binstall reposix-cli reposix-remote works <br/>source: `docs/index.md:64-64`
- **docs/index/ci-badge** — CI workflow badge resolves and indicates passing status <br/>source: `docs/index.md:7-7`
- **latency-hero-24ms-mismatch** — Homepage hero: reposix cold init = 24 ms <br/>source: `docs/index.md:18-18`
- **docs/index/mcp-loop-4883-tokens** — MCP loop uses ~4,883 tokens for same 3-issue read+edit+push <br/>source: `docs/index.md:35-39`
- **docs/index/install-powershell-irm** — PowerShell irm installer downloads from latest release <br/>source: `docs/index.md:54-54`
- **docs/index/quality-score-badge** — Quality score badge from custom endpoint resolves <br/>source: `docs/index.md:9-9`
- **docs/index/quality-weekly-badge** — Quality weekly workflow badge resolves <br/>source: `docs/index.md:8-8`
- **docs/index/real-git-working-tree** — reposix creates a real git working tree with cat, grep, edit, git commit, git push <br/>source: `docs/index.md:13-13`
- **docs/index/reposix-loop-531-tokens** — reposix loop uses ~531 tokens for 3-issue read+edit+push (from token-economy benchmark) <br/>source: `docs/index.md:31-34`
- **docs/index/soft-threshold-24ms** — Sim cold init is 24 ms (soft threshold 500 ms) <br/>source: `docs/index.md:93-93`
- **docs/why/token-economy-89-1-percent** — Token reduction is 89.1% vs MCP <br/>source: `docs/index.md:17-17`

</details>

## v0.12.1 carry-forward

v0.12.1 milestone phases P71+ close clusters above. Suggested phase scoping (1 cluster per phase or one phase per related-cluster group):

- **P71** — Confluence backend parity (close `Reference: confluence` MISSING_TEST + `Archived REQUIREMENTS.md (v0.8.0-phases)` Confluence rows). The smoking-gun page-tree-symlink regression originates here.
- **P72** — JIRA shape (close `ADR: JIRA issue mapping` + `Reference: jira` MISSING_TEST).
- **P73** — Benchmark numbers (close `Benchmarks: headline numbers` MISSING_TEST). Includes drift fix: README/index/social copy headline numbers vs measured benchmarks.
- **P74** — Connector authoring guide (close `Connector authoring guide` MISSING_TEST).
- **P75** — Tutorial integration coverage (close `Tutorials (first-run flow)` MISSING_TEST steps 4-8).
- **P76** — Glossary retirement bulk confirm (`Reference: glossary` 24 RETIRE_PROPOSED rows).
- **P77** — Developer workflow + invariants (close `Developer workflow + invariants` MISSING_TEST).
- **P78** — Concepts (close mental-model + reposix-vs-mcp MISSING_TEST).
- **P79** — Guides MISSING_TEST (REPOSIX_BLOB_LIMIT env override + others).
- **P80** — FUSE-era ADR retirement (`ADR: nested mount layout` etc.; cite v0.9.0 architecture pivot).

Numbering is suggestive; the human re-prioritizes when v0.12.1 opens.

## Floor-waiver expiry

`summary.floor_waiver.until` and `quality/catalogs/freshness-invariants.json` row `docs-alignment/walk` waiver both expire **2026-07-31**. Before that date, v0.12.1 must close enough clusters to lift `alignment_ratio` above 0.50 OR the human ratchets the floor + waiver explicitly.

---

*Generated by `scripts/gen_punch_list.py` from the populated docs-alignment catalog at the end of P65.*
