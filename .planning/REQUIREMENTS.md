# Requirements — Active milestone: v0.11.1 Polish & Reproducibility (second pass)

**Active milestone:** v0.11.1 Polish & Reproducibility (second pass) (planning_started 2026-04-26).
**Previous validated milestones:**
- v0.11.0 Polish & Reproducibility (SHIPPED 2026-04-25 implementation + autonomous §7 sweep 2026-04-26 PM, see "v0.11.0 Requirements" section below). Pre-tag owner gates remain (crates.io email verification + JIRA secrets); these carry forward into v0.11.1 as POLISH2-01 and POLISH2-04 alongside this milestone's typed-error / file-split / machine-readability work.
- v0.10.0 Docs & Narrative Shine (SHIPPED 2026-04-25, see "Archived (v0.10.0)" section below).
- v0.9.0 Architecture Pivot — Git-Native Partial Clone (SHIPPED 2026-04-24, see "v0.9.0 Requirements (Validated)" section below).

---

## v0.11.1 Requirements — Polish & Reproducibility (second pass)

**Milestone goal:** Close the carry-forward set surfaced by the autonomous §7 sweep on 2026-04-26 (HANDOVER.md §7-G). Two threads run in parallel: (a) **finish v0.11.0 ship-gates** (crates.io publish after owner email verification, linux-aarch64-musl in dist matrix, bench-latency-cron Authorization-header fix, JIRA latency cells once secrets land), and (b) **work the v0.12.0-deferred P1 list early** so the v0.11.x line gives harness authors and the security-lead persona enough machine-readability + typed-error hygiene to recommend reposix. Persona audits + code-quality gaps + repo-org gaps drive every requirement; nothing here is speculative.

**Source of truth:** `HANDOVER.md` §3 (release follow-ups) + §4 (friction matrix, 23 rows) + §7-G (next-coordinator carry-forward list); `.planning/research/v0.11.1-code-quality-gaps.md` P1 list (rows 1–9) + selected P2 (1, 7); `.planning/research/v0.11.1-persona-{mcp-user,harness-author,security-lead,skeptical-oss-maintainer,coding-agent}.md` (5 persona audits); `.planning/research/v0.11.1-repo-organization-gaps.md` recs #5 + orphan list. CATALOG-v3 JSON tracker at `.planning/CATALOG-v3.json` (rendered MD at `.planning/CATALOG-v3.md`) tracks per-file completion status.

**Operating-principle hooks (non-negotiable, per project CLAUDE.md):**

- **Self-improving infrastructure (OP-4).** `reposix doctor` becomes the canonical capability-matrix surface (POLISH2-08), removing the need for ad-hoc `gh secret list` + `cargo tree -p reposix-jira` reasoning when an agent debugs "why doesn't JIRA work for me." Audit-log schema unification (POLISH2-22) closes the dual-table gap that currently makes `doctor` blind to half the audit history.
- **Close the feedback loop (OP-1).** JIRA latency cells (POLISH2-04) + bench-cron Authorization fix (POLISH2-03) keep the latency table honest end-to-end; without the cron run, the table goes stale and `docs/benchmarks/v0.9.0-latency.md` rots.
- **Numbers, not adjectives.** The capability matrix (POLISH2-06) and `--json` output (POLISH2-18) give harness authors machine-checkable data; the v1.0 stability ADR (POLISH2-17) gives them a written commitment instead of "trust the changelog."
- **Ground truth obsession (OP-6).** Code-quality P1 work (typed errors, file splits, dep prune, `pub→pub(crate)`, sim typed-error) closes the stringly-typed protocol in `sim.rs:566-572` that the red-team analysis flagged as an attacker-controlled `Error::Other(String)` round-trip.

### Active

- [ ] **POLISH2-01**: crates.io email verification + first publish of all 9 reposix-* crates. Owner action: verify email at https://crates.io/settings/profile then `gh workflow run release-plz.yml`. Source: HANDOVER.md §3a.
- [shipped] **POLISH2-02**: linux-aarch64-musl binary added to `release.yml` dist matrix. Source: HANDOVER.md §3b. shipped via `feat(release): aarch64-unknown-linux-musl via cargo-zigbuild (POLISH2-02, closes §3b)` — added matrix entry with `zig: true`, conditional `cargo-zigbuild` install step, and branched build step in `.github/workflows/release.yml`.
- [shipped] **POLISH2-03**: `bench-latency-cron` Authorization-header duplicate fixed by setting `persist-credentials: false` on the `actions/checkout` step in `.github/workflows/bench-latency-cron.yml`. Source: bench-cron run 24965024010 failure. Shipped via b5fbcc6.
- [ ] **POLISH2-04**: JIRA real-backend latency cells populated. Owner-action prereq: provision `JIRA_EMAIL` + `JIRA_API_TOKEN` + `REPOSIX_JIRA_INSTANCE` secrets, then re-dispatch `bench-latency-v09`. Source: HANDOVER.md §4 row 5 partial.
- [shipped] **POLISH2-05**: Methodology callout added to `docs/concepts/reposix-vs-mcp-and-sdks.md` naming the synthesized MCP fixture (per HANDOVER §4 row 1). Source: friction row 1 partial. Shipped via c1f3614.
- [shipped] **POLISH2-06**: Connector capability matrix added to landing page (per friction row 7 follow-up + persona-coding-agent F-1). Source: friction row 7 follow-up. Shipped via b5fbcc6.
- [shipped] **POLISH2-07**: Comments-shape callout added to `docs/tutorials/first-run.md` ("sim round-trips; jira/github/confluence connector-specific"). Source: persona-coding-agent F-2. Shipped via c1f3614.
- [shipped] **POLISH2-08**: `reposix doctor` prints a configured-backend capability-matrix row. Source: persona-coding-agent fix #3. Shipped via `feat(doctor): print backend capability matrix row (POLISH2-08, persona-coding-agent fix #3)` — `BackendCapabilities` / `CommentSupport` / `VersioningModel` in `reposix-core`, `pub const CAPABILITIES` in each backend crate, `check_backend_capabilities` Info finding in `crates/reposix-cli/src/doctor.rs`.
- [partial-shipped] **POLISH2-09**: Code-quality P1 — `Error::Other(String)` rewritten to typed `Error::NotFound` / `Error::NotSupported` / `Error::VersionMismatch` variants in `reposix-core`; closes the stringly-typed protocol in `crates/reposix-core/src/backend/sim.rs:566-572`. Source: code-quality P1-1 + P1-5. shipped: 3 typed variants + sim.rs migrated. 150+ Error::Other sites in backend adapters still pending v0.12.0.
- [shipped] **POLISH2-10**: Code-quality P1 — `crates/reposix-confluence/src/lib.rs` (3 989 LOC) split into matching modules (mirror of POLISH2-11). Source: code-quality P1-2. Shipped via `refactor(confluence): split lib.rs into types/translate/client modules (POLISH2-10, code-quality P1-2)` — new `types.rs` (417 LOC), `translate.rs` (358 LOC), `client.rs` (2 850 LOC), trimmed `lib.rs` (447 LOC, holds only `BackendConnector` impl + module declarations + `pub use` re-exports).
- [shipped] **POLISH2-11**: Code-quality P1 — `crates/reposix-jira/src/lib.rs` (1 957 LOC) split into matching modules (mirror of POLISH2-10). Source: code-quality P1-2. Shipped via `refactor(jira): split lib.rs into types/translate/client modules (POLISH2-11, code-quality P1-2)` — new `types.rs` (159 LOC), `translate.rs` (355 LOC), `client.rs` (1001 LOC), trimmed `lib.rs` (517 LOC, holds only `BackendConnector` impl + module declarations + `pub use` re-exports).
- [shipped] **POLISH2-12**: Code-quality P1 — drop 3 unused Cargo deps from `reposix-remote` (`serde`, `serde_yaml`, `clap`). Source: code-quality P1-6. Shipped via 48fcd4c. Note: `serde_json` KEPT — used in 3 integration tests; audit was off by one.
- [shipped] **POLISH2-13**: Code-quality P1 — demote `pub` → `pub(crate)` on 49 `reposix-remote` symbols (binary-only crate; `pub` is a no-op + future-confusing). Source: code-quality P1-7. Shipped via dba89c5.
- [shipped] **POLISH2-14**: Code-quality P1 — typed `SimError` introduced in `reposix-sim`; dropped `anyhow` from the library API. Source: code-quality P1-4. Shipped via fc459d0+649da8c.
- [shipped] **POLISH2-15**: `docs/development/roadmap.md` rewritten as 1-screen stub pointing at `.planning/ROADMAP.md` (the GSD source of truth). Source: friction row 21 NEW + repo-org docs/ orphans. Shipped via c088cd1.
- [shipped] **POLISH2-16**: Internal ADR-002 + ADR-003 nav cleanup (ADR-003 marked superseded earlier in 220bfbd; ADR-002 verified current — `Accepted (layout section superseded by ADR-003)`). Source: friction row 15 partial. Shipped via 48859ae.
- [shipped] **POLISH2-17**: ADR-009 v1.0 stability commitment authored (137 lines locking URL shape + CLI surface + exit codes + helper protocol + frontmatter allowlist + BackendConnector trait). Source: friction row 13 + persona-harness-author. Shipped via 48859ae.
- [shipped] **POLISH2-18**: `docs/reference/exit-codes.md` authored (186 lines documenting 0/1 for CLI subcommands and 0/1/2 for git-remote-reposix). Source: friction row 14 + persona-harness-author. Shipped via 47aabf0. `--json`/`--format=json` flag deferred to v0.12.0.
- [ ] **POLISH2-19**: `.claude/skills/reposix-banned-words/SKILL.md` path refs at L9+L64 updated from `.planning/notes/` → `.planning/archive/notes/`. Owner-approval gated. Source: §7-F2 deferred item 4.
- [shipped] **POLISH2-20**: Upstream issue filed at https://github.com/squidfunk/mkdocs-material/issues/8584 ("Bug: superfences `<pre class="mermaid">` content is stripped when minify_html is enabled"). Audit doc footer updated. Source: HANDOVER §7-G last bullet. Shipped via 0375ffc.
- [ ] **POLISH2-21**: `.planning/` tree condensed per repo-org rec #5: 8 `v0.X.0-phases` dirs collapsed into 8 `ARCHIVE.md` files (273 → ~16 files net). Source: repo-org rec #5 (P2).
- [ ] **POLISH2-22**: Two parallel audit-log schemas unified per code-quality CC-3 (or document the dual-schema design intentionally). Source: code-quality P0-4 / friction row 12.

### Out of Scope

- Full glossary polish + persona-driven page rewrites — separate doc-quality milestone, not v0.11.1.
- Supply chain hardening (cosign, SBOM, SLSA, Apple notarization, Authenticode) — friction row 19; deferred to v0.12.0+ per security-lead priority guidance.
- `research/threat-model-and-critique.md` rewrite for v0.9.0 architecture — friction row 20; the FUSE-era file at `.planning/research/v0.1-fuse-era/threat-model-and-critique.md` stays as-is for v0.11.1; the rewrite ships in v0.12.0.

### Traceability

| REQ-ID | Phase | Status |
|--------|-------|--------|
| POLISH2-01 | TBD (Phase 56+) | planning (owner-action gate) |
| POLISH2-02 | inline (no GSD phase) | shipped |
| POLISH2-03 | inline (no GSD phase) | shipped |
| POLISH2-04 | TBD | planning (owner-action gate) |
| POLISH2-05 | inline (no GSD phase) | shipped |
| POLISH2-06 | inline (no GSD phase) | shipped |
| POLISH2-07 | inline (no GSD phase) | shipped |
| POLISH2-08 | inline (no GSD phase) | shipped |
| POLISH2-09 | inline (no GSD phase) | partial-shipped (typed variants + sim.rs migrated; 150+ Error::Other in backends pending v0.12.0) |
| POLISH2-10 | inline (no GSD phase) | shipped |
| POLISH2-11 | inline (no GSD phase) | shipped |
| POLISH2-12 | inline (no GSD phase) | shipped |
| POLISH2-13 | inline (no GSD phase) | shipped |
| POLISH2-14 | inline (no GSD phase) | shipped |
| POLISH2-15 | inline (no GSD phase) | shipped |
| POLISH2-16 | inline (no GSD phase) | shipped |
| POLISH2-17 | inline (no GSD phase) | shipped |
| POLISH2-18 | inline (no GSD phase) | shipped (exit-codes.md only; --json deferred v0.12.0) |
| POLISH2-19 | TBD | planning (owner-approval gate) |
| POLISH2-20 | inline (no GSD phase) | shipped (issue #8584) |
| POLISH2-21 | TBD | planning |
| POLISH2-22 | TBD | planning |

---

## v0.11.0 Requirements — Polish & Reproducibility

**Milestone goal:** Close the long tail that v0.10.0 surfaced. Polish the docs site (jargon glosses + glossary + mermaid render hygiene + ADR cleanup), kill four codebase duplicates flagged by `simplify` (worktree helpers, `parse_remote_url`, `cli_compat.rs`, FUSE residue in `refresh.rs`), and ship reproducibility infrastructure: a fresh-clone tutorial runner, pre-built binaries via `dist`, `cargo binstall` metadata, `reposix doctor` / `reposix log --time-travel` / `reposix gc --orphans` / `reposix cost` surfaces, and a real-backend latency table for sim + GitHub + Confluence + JIRA.

**Source of truth:** `.planning/research/v0.11.0-vision-and-innovations.md` (vision spec) plus the v0.11.0 audit family: `v0.11.0-gsd-hygiene-report.md`, `v0.11.0-mkdocs-site-audit.md`, `v0.11.0-jargon-inventory.md`, `v0.11.0-latency-benchmark-plan.md`, `v0.11.0-release-binaries-plan.md`, `v0.11.0-cache-location-study.md`, `v0.11.0-CATALOG-v2.md`. Framing principles inherited from `.planning/notes/phase-30-narrative-vignettes.md` (P1 complement-not-replace; P2 progressive disclosure — banned-above-Layer-3 list extended with v0.11.0 jargon glosses owned by the new `docs/reference/glossary.md`).

**Operating-principle hooks (non-negotiable, per project CLAUDE.md):**

- **Self-improving infrastructure (OP-4).** `scripts/repro-quickstart.sh`, `scripts/check-docs-site.sh`, and the dist release pipeline are committed-and-CI-wired, not session memory. Per project CLAUDE.md: ad-hoc bash that asserts cross-file invariants is a missing-tool signal.
- **Close the feedback loop (OP-1).** Mermaid diagrams render via mcp-mermaid AND playwright-screenshot the live site for every page touched. `mkdocs build --strict` is green by definition; pre-push hook runs `check-docs-site.sh` so a broken site never reaches `main`.
- **Numbers, not adjectives.** Latency table populated for sim + github + confluence + jira with record counts and 3-sample medians; doctor output has copy-pastable fix strings, not narrative.
- **Ground truth obsession (OP-6).** Tutorial reproducibility is asserted by a script that runs against a fresh `/tmp/clone`, not by reading a doc.

### Active

- [ ] **POLISH-01**: All jargon terms have inline gloss + external link at first occurrence per page (drives Phase 52). Inventory in `.planning/research/v0.11.0-jargon-inventory.md`.
- [ ] **POLISH-02**: `docs/reference/glossary.md` exists; every other page links to it on first jargon term (≥24 entries, drives Phase 52).
- [ ] **POLISH-03**: All mermaid diagrams render without console errors on the live site. Drives Phase 52, F1+F2+F3 from `.planning/research/v0.11.0-mkdocs-site-audit.md`.
- [ ] **POLISH-04**: `mkdocs build --strict` is green; ADR-008 in nav; blog post in `not_in_nav`; `pymdownx.emoji` configured.
- [ ] **POLISH-05**: Tutorial reproducible from fresh clone — `bash scripts/repro-quickstart.sh` runs the 7-step tutorial and asserts each step passes (drives Phase 53).
- [ ] **POLISH-06**: Pre-built binaries published to GitHub Releases for linux musl x86/arm64, macOS x86/arm64, windows msvc on every git tag (drives Phase 53).
- [ ] **POLISH-07**: `cargo binstall reposix-cli` works (drives Phase 53). Cargo metadata for binstall configured.
- [ ] **POLISH-08**: Latency table populated for sim + github + confluence + jira with record counts and 3-sample medians (drives Phase 54). Plan in `.planning/research/v0.11.0-latency-benchmark-plan.md`.
- [ ] **POLISH-09**: `reposix doctor` runs the full v3a checklist with copy-pastable fix strings (drives Phase 55). Spec in `.planning/research/v0.11.0-vision-and-innovations.md`.
- [ ] **POLISH-10**: `reposix log --time-travel`, `reposix init --since=<RFC3339>`, `reposix cost`, `reposix gc --orphans` all surfaced (drives Phase 55).
- [ ] **POLISH-11**: ADR-004 + ADR-006 deleted; `agentic-engineering-reference.md` carries a disclaimer; archival doc sweep complete (`MORNING-WALKTHROUGH-*.md`, root `RELEASE-NOTES-*.md`, blog launch post, `docs/archive/MORNING-BRIEF.md`).
- [ ] **POLISH-12**: `Cargo.toml` workspace version is `0.11.0-dev` until tag time, then bumped to `0.11.0` for ship.
- [ ] **POLISH-13**: 4-way CLI worktree-helper duplication consolidated into `crates/reposix-cli/src/worktree_helpers.rs` (drives Phase 51).
- [ ] **POLISH-14**: `parse_remote_url` exists once in `reposix-core`; `reposix-remote/backend_dispatch` calls into it (drives Phase 51).
- [ ] **POLISH-15**: `crates/reposix-cache/src/cli_compat.rs` deleted; downstream consumers migrated to canonical opener (drives Phase 51).
- [ ] **POLISH-16**: `crates/reposix-cli/src/refresh.rs` has zero FUSE residue (`is_fuse_active`, `mount_point` removed) (drives Phase 51).
- [ ] **POLISH-17**: CLAUDE.md adds: any docs-site work MUST be playwright-validated; `scripts/check-docs-site.sh` exists and is wired into pre-push (drives Phase 53).

### Out of Scope

- New backend connectors. v0.11.0 stays on the existing 4 (sim, github, confluence, jira).
- `cargo install reposix-cli` from crates.io publishing — kept as a Phase 53 stretch goal but not gated on it; `cargo binstall` is the ship requirement.
- Full v0.12.0 agent-SDK recipes (Claude Code / Cursor / Aider). v0.11.0 keeps `docs/guides/integrate-with-your-agent.md` as a pointer page; recipes ship in v0.12.0.

### Traceability

| REQ-ID | Phase | Status |
|--------|-------|--------|
| POLISH-01 | 52 | planning |
| POLISH-02 | 52 | planning |
| POLISH-03 | 52 | planning |
| POLISH-04 | 52 | planning |
| POLISH-05 | 53 | planning |
| POLISH-06 | 53 | planning |
| POLISH-07 | 53 | planning |
| POLISH-08 | 54 | planning |
| POLISH-09 | 55 | planning |
| POLISH-10 | 55 | planning |
| POLISH-11 | 50, 52 | in-progress (Phase 50 wave landed) |
| POLISH-12 | 50, then tag-time | in-progress (Phase 50 bump landed; final bump at tag time) |
| POLISH-13 | 51 | planning |
| POLISH-14 | 51 | planning |
| POLISH-15 | 51 | planning |
| POLISH-16 | 51 | planning |
| POLISH-17 | 53 | planning |

---

## Archived (v0.10.0) — Docs & Narrative Shine (SHIPPED 2026-04-25)

**Milestone goal:** Make the reposix value proposition land in 10 seconds for a cold reader, with progressive disclosure of architecture and a tested 5-minute first-run tutorial. Sales-ready docs with hard numbers, agent-SDK guidance, and a banned-word linter that enforces P1/P2 framing rules.

**Source of truth:** `.planning/research/v0.10.0-post-pivot/milestone-plan.md`. Framing principles inherited from `.planning/notes/phase-30-narrative-vignettes.md`.

### Validated

- ✓ **DOCS-01**: Reader can understand reposix's value proposition within 10 seconds of landing on `docs/index.md` — Phase 40
- ✓ **DOCS-02**: Three-page "How it works" section (`filesystem-layer`, `git-layer`, `trust-model`) — Phase 41
- ✓ **DOCS-03**: Two home-adjacent concept pages (`mental-model-in-60-seconds`, `reposix-vs-mcp-and-sdks`) — Phase 40
- ✓ **DOCS-04**: Three guides (`write-your-own-connector`, `integrate-with-your-agent`, `troubleshooting`) — Phase 42
- ✓ **DOCS-05**: Simulator relocated from "How it works" to Reference (`docs/reference/simulator.md`) — Phase 42
- ✓ **DOCS-06**: 5-minute first-run tutorial verified by `scripts/tutorial-runner.sh` — Phase 42
- ✓ **DOCS-07**: MkDocs nav restructured per Diátaxis — Phase 43
- ✓ **DOCS-08**: mkdocs-material theme tuning + README hero rewrite — Phase 43 (linter wiring) + Phase 45 (README)
- ✓ **DOCS-09**: Banned-word linter + skill — Phase 43
- ✓ **DOCS-10**: 16-page cold-reader clarity audit; zero critical friction points — Phase 44
- ✓ **DOCS-11**: README points at mkdocs site; CHANGELOG `[v0.10.0]` — Phase 45 (playwright screenshots deferred to v0.11.0 — cairo system libs unavailable on dev host; tracked under POLISH-11)

---

## v0.9.0 Requirements (Validated) — Architecture Pivot — Git-Native Partial Clone

**Milestone goal:** Replace the FUSE virtual filesystem with git's built-in partial clone mechanism. The `git-remote-reposix` helper becomes a promisor remote tunnelling protocol-v2 traffic to a local bare-repo cache built from REST responses. Agents interact with the project using only standard git commands (`clone`, `fetch`, `cat`, `grep`, `commit`, `push`) — zero reposix-specific CLI awareness required. FUSE is deleted entirely; `crates/reposix-fuse/` is removed and the `fuser` dependency is purged.

**Source of truth:** `.planning/research/v0.9-fuse-to-git-native/architecture-pivot-summary.md` (canonical design doc, 440 lines, ratified 2026-04-24). Supporting POC artifacts in `.planning/research/v0.9-fuse-to-git-native/poc/`.

**Operating-principle hooks (non-negotiable, per project CLAUDE.md):**

- **Simulator-first.** All ARCH requirements ship and test against `reposix-sim` by default. Real backends (GitHub, Confluence, JIRA) are exercised only behind `REPOSIX_ALLOWED_ORIGINS` + explicit credentials.
- **Tainted-by-default.** Every byte materialized into the bare-repo cache or returned to git originated from a remote (real or simulator). Tainted content must be wrapped in `reposix_core::Tainted<T>` along the data path; sanitization is explicit.
- **Audit log non-optional.** Every blob materialization, every `command=fetch`, every `export` push (accept and reject) writes a row to the SQLite audit table.
- **Egress allowlist.** All HTTP construction goes through the existing `reposix_core::http::client()` factory; no new direct `reqwest::Client::new()` call sites. `REPOSIX_ALLOWED_ORIGINS` is enforced before any outbound request.
- **Working tree = real git repo.** The mount point is no longer synthetic — it is a true git working tree backed by `.git/objects` (partial clone, blobs lazy).
- **Self-improving infrastructure (OP-4).** Project CLAUDE.md and Claude Code skills MUST ship in lockstep with the code that invalidates them. Phase 36 explicitly bundles agent-grounding updates with FUSE deletion.

---

## v1 Requirements

### Cache & data path

- [ ] **ARCH-01**: New crate `crates/reposix-cache/` constructs a real on-disk bare git repo from REST responses via the existing `BackendConnector` trait. The cache produces a fully populated tree (filenames, directory structure, blob OIDs) but stores blobs lazily — a blob is only materialized when the helper requests it on behalf of git. The cache is the substrate that `git-remote-reposix` proxies protocol-v2 traffic to. Source: architecture-pivot-summary §2 (How it works), §5 (Add).

- [ ] **ARCH-02**: `reposix-cache` writes one audit row per blob materialization (per OP "audit non-optional"). Cache returns blob bytes wrapped in `reposix_core::Tainted<Vec<u8>>` (per OP "tainted-by-default") so downstream code cannot accidentally route tainted bytes into side-effecting actions on other systems without an explicit `sanitize` step. Audit schema: `(ts, backend, project, issue_id, blob_oid, byte_len, op="materialize")`. The cache table mirrors today's SQLite WAL append-only policy — no UPDATE or DELETE on audit rows.

- [ ] **ARCH-03**: `reposix-cache` enforces `REPOSIX_ALLOWED_ORIGINS` egress allowlist by reusing the single `reposix_core::http::client()` factory. No new `reqwest::Client` construction site is added; clippy `disallowed_methods` already catches direct calls. Tests assert that an attempt to materialize a blob from a backend whose origin is not in the allowlist returns `EPERM`-equivalent and is audited.

### Transport (read path)

- [ ] **ARCH-04**: `git-remote-reposix` advertises the `stateless-connect` capability and tunnels protocol-v2 fetch traffic (handshake, ls-refs, fetch with filter) to the cache's bare repo. The existing `export` capability for push remains alongside `stateless-connect` (hybrid helper, confirmed working in POC `poc/git-remote-poc.py`). Both capabilities live in the same binary; git dispatches based on direction. Source: architecture-pivot-summary §3 (Transport routing).

- [ ] **ARCH-05**: The Rust `stateless-connect` implementation correctly handles all three protocol gotchas surfaced during POC iteration (architecture-pivot-summary §3 "Three protocol gotchas"):
  1. **Initial advertisement does NOT terminate with `0002`** — only `0000` (flush). Rust port must send the bytes from `upload-pack --advertise-refs --stateless-rpc` verbatim, no trailing response-end.
  2. **Subsequent RPC responses DO need trailing `0002`** — after each response pack, write the bytes from `upload-pack --stateless-rpc` followed by the response-end marker.
  3. **Stdin reads in binary mode throughout** — the helper reads the protocol stream via a `BufReader<Stdin>` consistently; mixing text and binary reads corrupts the framing.

  Refspec namespace MUST be `refs/heads/*:refs/reposix/*` (NOT `refs/heads/*:refs/heads/*` — the empty-delta bug from POC). The current `crates/reposix-remote` already uses the correct namespace and must not regress.

### Sync (delta path)

- [ ] **ARCH-06**: `BackendConnector::list_changed_since(timestamp) -> Vec<IssueId>` is added as a trait method. All backends (`SimBackend`, `GithubBackend`, `ConfluenceBackend`, `JiraBackend`) implement it using their native incremental query mechanism: GitHub `?since=<ISO8601>`, Jira `JQL: updated >= "<datetime>"`, Confluence `CQL: lastModified > "<datetime>"`. The simulator implements it by filtering its in-memory issue set against the `since` query parameter exposed in its REST surface. Source: architecture-pivot-summary §4 (Delta sync).

- [ ] **ARCH-07**: Delta sync flow: on `git fetch`, the helper reads `last_fetched_at` from the cache DB, calls `list_changed_since(last_fetched_at)` on the backend, materializes the changed items into the bare-repo cache, advertises the updated tree to git via protocol v2, then atomically writes the new `last_fetched_at` (per architecture-pivot-summary §4 "Fetch flow"). Tree sync is unconditional and not gated by the blob limit (tree metadata is small — full sync is cheap). Cache update and `last_fetched_at` write happen in one SQLite transaction so a crash mid-sync cannot leave divergent state. One audit row per delta-sync invocation: `(ts, backend, project, since_ts, items_returned, op="delta_sync")`.

### Push path

- [ ] **ARCH-08**: Push-time conflict detection lives inside the `export` handler. Flow (architecture-pivot-summary §3 "Conflict detection happens inside `handle_export`"):
  1. Parse the fast-import stream in memory (or via state machine).
  2. For each changed file, fetch the current backend version via REST.
  3. If the backend version differs from the agent's commit base: emit `error refs/heads/main fetch first` (canned status — git renders the standard "perhaps a `git pull` would help" hint) AND a detailed diagnostic via stderr through the existing `diag()` channel.
  4. Reject path drains the incoming stream and never touches the bare cache (no partial state).
  5. On success: apply REST writes, update bare-repo cache, emit `ok refs/heads/main`.

  Audit row for every push attempt (accept and reject): `(ts, backend, project, ref, files_touched, decision, reason)`.

- [ ] **ARCH-09**: Blob limit guardrail. The helper counts `want <oid>` lines per `command=fetch` request and refuses if the count exceeds `REPOSIX_BLOB_LIMIT` (default 200, env-configurable). Refusal writes a stderr error message that *names* the remediation: `"error: refusing to fetch <N> blobs (limit: <M>). Narrow your scope with `git sparse-checkout set <pathspec>` and retry."`. The error message is deliberately self-teaching (dark-factory pattern from architecture-pivot-summary §4 "Blob limit as teaching mechanism" — an agent unfamiliar with reposix observes the error, runs `git sparse-checkout`, and recovers without human prompt engineering).

- [ ] **ARCH-10**: Frontmatter field allowlist on the push path. Server-controlled fields (`id`, `created_at`, `version`, `updated_at`) are stripped from inbound writes BEFORE the REST call, mirroring the policy currently enforced on the FUSE write path. An attacker-authored issue body with `version: 999999` MUST NOT update the server version. The `Tainted<T>` -> `Untainted<T>` conversion is the explicit `sanitize` step where this stripping happens.

### CLI & agent UX

- [ ] **ARCH-11**: `reposix init <backend>::<project> <path>` replaces `reposix mount`. The new command:
  1. Runs `git init <path>`.
  2. Configures `extensions.partialClone=origin`.
  3. Sets `remote.origin.url=reposix::<backend>/<project>`.
  4. Runs `git fetch --filter=blob:none origin`.
  5. Optionally runs `git checkout origin/main` (or leaves the working tree empty for sparse-first agents).

  `reposix mount` is removed in the same release. CHANGELOG `[v0.9.0]` documents the breaking CLI change with a migration note. Source: architecture-pivot-summary §5 (Change).

- [ ] **ARCH-12**: End-to-end agent UX validation. A fresh subprocess agent (no reposix CLI awareness, no in-context system-prompt instructions about reposix) is given ONLY a `reposix init` command and a goal (e.g., "find issues mentioning 'database' and add a TODO comment to each"). The agent succeeds using pure git/POSIX tools: `cd /tmp/repo && cat issues/<id>.md && grep -r TODO . && <edit> && git add . && git commit && git push`. The validation specifically exercises the conflict-rebase cycle (a second writer modifies a backend issue between the agent's `clone` and `push`; agent observes `! [remote rejected] main -> main (fetch first)`, runs `git pull --rebase`, retries `git push`, succeeds). This is the dark-factory acceptance test — see architecture-pivot-summary §4 "Agent UX: pure git, zero in-context learning".

### Demolition & grounding

- [ ] **ARCH-13**: `crates/reposix-fuse/` is deleted entirely. The `fuser` dependency is removed from the workspace `Cargo.toml`. All FUSE-only test feature gates (`fuse-mount-tests`) are removed. CI no longer runs `apt install fuse3` or mounts `/dev/fuse`. After this requirement lands, `cargo metadata --format-version 1 | grep fuser` returns nothing, and `cargo check --workspace && cargo clippy --workspace --all-targets -- -D warnings` is green without any FUSE-related package on the host. Source: architecture-pivot-summary §5 (Delete).

- [ ] **ARCH-14**: Project `CLAUDE.md` is updated as part of the same phase that lands FUSE deletion (NOT a follow-up phase). Specifically:
  - All FUSE references are purged from the elevator pitch, Operating Principles, Workspace layout, Tech stack, "Commands you'll actually use", and the Threat-model table.
  - Any "Architecture transition" or "v0.9.0 in progress" banner is replaced with a steady-state **Architecture (git-native partial clone)** section describing the cache + helper + agent-UX flow.
  - The Tech stack row for `fuser` is removed. The replacement row mentions `git >= 2.34` as a runtime requirement (per architecture-pivot-summary §7 risks).
  - "Commands you'll actually use" no longer mentions `reposix mount` or `cargo test --features fuse-mount-tests`. New commands shown: `reposix init`, `git clone reposix::sim/proj-1 /tmp/repo`.
  - Threat-model table updated: the helper (not the FUSE daemon) is the egress surface. Allowlist now applies to the helper + cache, not the FUSE daemon.

  Per OP-4 self-improving infrastructure: agent grounding must match shipped reality — there can be no window where CLAUDE.md describes deleted code.

- [ ] **ARCH-15**: A project Claude Code skill `reposix-agent-flow` is created at `/home/reuben/workspace/reposix/.claude/skills/reposix-agent-flow/SKILL.md` (Claude Code skill convention — directory with `SKILL.md` and YAML frontmatter `name:` + `description:`). The skill encodes the dark-factory autonomous-agent regression test: spawn a fresh subprocess Claude (or scripted shell agent acting as one) inside an empty directory, hand it ONLY a `reposix init` command and a natural-language goal, and verify the agent completes the task using pure git/POSIX tools — including the conflict-rebase cycle and the blob-limit-induced sparse-checkout recovery from ARCH-09. The skill is invoked from CI (release gate) and from local dev (`/reposix-agent-flow`). The skill MUST mention that it is the StrongDM/dark-factory regression harness for the v0.9.0 architecture and reference architecture-pivot-summary §4 "Agent UX". Per OP-4: ships in the same phase as ARCH-13 and ARCH-14.

### Real-backend validation, performance, and canonical test targets

- [ ] **ARCH-16**: Real-backend smoke tests against the project's canonical test targets. The helper + cache must be validated end-to-end against real APIs, not just the simulator. Targets: Confluence "TokenWorld" space, reposix's own GitHub issues (`reubenjohn/reposix`), JIRA project `TEST` (overridable via `JIRA_TEST_PROJECT` or `REPOSIX_JIRA_PROJECT`). Gated by `REPOSIX_ALLOWED_ORIGINS` + credentials; skipped in CI when creds absent (`#[ignore]` or `skip_if_no_env!`). Must exercise: partial clone, lazy blob fetch, delta sync after upstream mutation, push round-trip, conflict rejection. Simulator-only coverage does NOT satisfy this requirement — at least one real backend must be exercised under the same suite for v0.9.0 to ship.

- [ ] **ARCH-17**: Latency & performance envelope captured for each backend. A committed benchmark script runs the reposix golden path (`clone` → first blob read → batched checkout of 10 blobs → edit → push) against sim + each real backend, records wall-clock latency per step, and writes a Markdown artifact under `docs/benchmarks/v0.9.0-latency.md`. Targets are encoded as soft thresholds (e.g. sim cold clone < 500ms, real backend < 3s). Regressions are flagged but not CI-blocking. This is a **sales asset** — the artifact is the reposix-vs-MCP-vs-REST comparison table that ships in narrative docs (v0.10.0).

- [ ] **ARCH-18**: Canonical test-target documentation. `docs/reference/testing-targets.md` enumerates TokenWorld (Confluence), `reubenjohn/reposix` issues (GitHub), and JIRA project `TEST` — with env-var setup, rate-limit notes, cleanup procedure ("do not leave junk issues"), and the explicit "go crazy, it's safe" permission statement from the owner. CLAUDE.md cross-references this file so any agent looking for sanctioned test targets is one hop away from the truth.

- [ ] **ARCH-19**: CI integration-contract job for each real backend. Three CI jobs (`integration-contract-confluence-v09`, `integration-contract-github-v09`, `integration-contract-jira-v09`) run the ARCH-16 smoke suite when the corresponding secret block is present. Each job writes a run artifact (latency rows from ARCH-17). Pattern mirrors the existing `integration-contract-confluence` job from Phase 11 — `pending-secrets` status when creds unavailable, green when present.

---

## Future Requirements

*(Deferred; emerge from open questions in architecture-pivot-summary §7.)*

- Cache eviction policy for `reposix-cache` (LRU / TTL / per-project disk quota / manual `reposix gc`).
- `import` capability deprecation (kept one release cycle past v0.9.0, removed in v0.10.0 or v0.11.0).
- Stream-parsing performance for `export` — production state-machine parser with commit-or-rollback barrier at fast-import `done` terminator.
- Threat-model update for push-through-export flow — `research/threat-model-and-critique.md` revision once the helper-as-egress-surface model is stable.
- Non-issue file handling on push (e.g., changes to `.planning/` paths) — reject vs. silently commit-to-cache decision.

---

## Out of Scope

- **Bringing FUSE back.** The pivot is one-way; `crates/reposix-fuse/` is removed, not deprecated.
- **Custom CLI for read operations.** Agents use `git clone`, `git fetch`, `cat`, `grep` — no `reposix list`, no `reposix get`. The whole point is zero in-context learning.
- **Full real-backend exercise in autonomous mode.** Real backends require explicit credentials and a non-default `REPOSIX_ALLOWED_ORIGINS`. Autonomous execution validates against the simulator only.
- **Cache eviction implementation in v0.9.0.** Decision deferred per architecture-pivot-summary §7 open question 1; the cache grows monotonically until then.
- **`docs/` rewrite.** Carried forward to v0.10.0 (DOCS-01..09).

---

## Traceability (v0.9.0 — historical)

| REQ-ID | Phase | Status |
|--------|-------|--------|
| ARCH-01 | 31 | shipped |
| ARCH-02 | 31 | shipped |
| ARCH-03 | 31 | shipped |
| ARCH-04 | 32 | shipped |
| ARCH-05 | 32 | shipped |
| ARCH-06 | 33 | shipped |
| ARCH-07 | 33 | shipped |
| ARCH-08 | 34 | shipped |
| ARCH-09 | 34 | shipped |
| ARCH-10 | 34 | shipped |
| ARCH-11 | 35 | shipped |
| ARCH-12 | 35 | shipped |
| ARCH-13 | 36 | shipped |
| ARCH-14 | 36 | shipped |
| ARCH-15 | 36 | shipped |
| ARCH-16 | 35 | shipped |
| ARCH-17 | 35 (capture) + 36 (artifact) | shipped |
| ARCH-18 | 36 | shipped |
| ARCH-19 | 36 | shipped (pending-secrets until CI secrets decrypt) |

*(v0.9.0 ARCH-01..19 all shipped 2026-04-24. v0.10.0 DOCS-01..11 all shipped 2026-04-25 — see "Archived (v0.10.0)" above. v0.11.0 active list is POLISH-01..17 above.)*
