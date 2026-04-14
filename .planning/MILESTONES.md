# Milestones — reposix

## v0.5.0 — Dynamic sitemap + IssueBackend decoupling (SHIPPED 2026-04-14)

**Goal:** Ship OP-2 partial (`_INDEX.md` in bucket dir) and decouple FUSE write path + git-remote helper from the sim-specific REST layer.

**Shipped:**
- Phase 14 (v0.4.1): Decouple sim REST shape from FUSE write path and git-remote helper — route through `IssueBackend` trait. Deleted `fetch.rs`, `write.rs`, `client.rs` (~830 LoC). 
- Phase 15 (v0.5.0): Dynamic `_INDEX.md` synthesized in FUSE bucket directory — YAML frontmatter + markdown table sitemap, read-only, lazily rendered.

**Tests:** 277/277 green. Clippy clean. CI green.

---

## v0.4.0 — Nested mount layout: pages/ + tree/ symlinks (SHIPPED 2026-04-14)

**Goal:** Ship OP-1 — convert flat `<id>.md` root to two-view layout: writable `pages/` bucket + synthesized read-only `tree/` symlink overlay for Confluence parentId hierarchy.

**Shipped:**
- Phase 13: Nested mount layout, ADR-003, slug deduplication, cycle-safe DFS, integration tests.

**Tests:** 272/272 green.

---

## v0.3.0 — Confluence Cloud read-only adapter (SHIPPED 2026-04-14)

**Goal:** Ship `reposix-confluence` crate implementing `IssueBackend` against Atlassian Confluence Cloud REST v2. CLI dispatch for list + mount.

**Shipped:**
- Phase 11: ConfluenceReadOnlyBackend, wiremock tests, contract tests, parity demo, ADR-002, connector guide.

**Tests:** 193/193 green.

---

## v0.2.0-alpha — GitHub read-only adapter + demo suite (SHIPPED 2026-04-13)

**Goal:** Ship Phase 8 (demo suite restructure + IssueBackend seam + GithubReadOnlyBackend + contract tests).

**Shipped:**
- Phase 8: IssueBackend trait, SimBackend impl, GithubReadOnlyBackend, contract test suite, Tier-1 demos.

---

## v0.1.0 — MVD: Simulator + FUSE + CLI + demo (SHIPPED 2026-04-13)

**Goal:** Simulator-first read-only FUSE mount that an LLM agent can use with `cat`, `grep`, and `git`.

**Shipped:**
- Phase 1: Core contracts + security guardrails
- Phase 2: Simulator + audit log
- Phase 3: Read-only FUSE mount + CLI
- Phase 4: Demo + recording + README
- Phase S: STRETCH — write path + swarm + FUSE-in-CI

**Tests:** 168/168 green. Demo recorded.
