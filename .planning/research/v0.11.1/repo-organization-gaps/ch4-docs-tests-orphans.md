# docs/ and tests/ orphans

[← index](./index.md)

**docs/ orphans:**

- `docs/demo.md` (198 bytes, "(moved)" stub) + `docs/architecture.md` (357 bytes, stub) + `docs/security.md` (304 bytes) + `docs/why.md` (422 bytes) + `docs/connectors/guide.md` — all redirect-stub pages from the v0.10.0 nav restructure. **KEEP as long as external inbound links exist** (the docs site has been live since v0.4); revisit at v1.0.
- `docs/demo.transcript.txt` + `docs/demo.typescript` (top-level, ~7 KB) — pre-v0.4 single-demo recording. **DELETE**. Superseded by `docs/demos/recordings/` (which itself is being deleted in rec #1).
- `docs/github-readme-top.png` (144 KB) — pre-pivot screenshot of the README hero. **DELETE** — README.md no longer references it (it was last referenced from a v0.4-era top section).
- `docs/screenshots/gh-pages-home-v0.2.png` + `gh-pages-why-real-github.png` — v0.2-era screenshots, FUSE-shaped UI. **ARCHIVE** to `docs/screenshots/archive/v0.2/` or **DELETE**. `gh-pages-home.png`, `site-architecture.png`, `site-home.png`, `site-security.png` are current.
- `docs/decisions/001-github-state-mapping.md` + `002-confluence-page-mapping.md` + `003-nested-mount-layout.md` — referenced by CATALOG-v2 as "ADR-001/002/003 reference deleted FUSE layer; need scope-superseded notice". **KEEP, ANNOTATE** with the standard "Status: superseded by ADR-008" header. (Note: the file names ADR-004/006 already deleted in v0.11.0 per STATE.md; remaining ADRs are 001, 002, 003, 005, 007, 008.)
- `docs/development/roadmap.md` — CATALOG-v2 says "stops at v0.7". **REWRITE** to a 1-screen "we use GSD; see `.planning/ROADMAP.md`" stub or delete; the public mkdocs roadmap drifted from `.planning/ROADMAP.md`. Same with `docs/development/contributing.md` which mentions `reposix-fuse/` per CATALOG-v2 — repoint to top-level `CONTRIBUTING.md` (which is already current — see top-level audit).
- `docs/research/initial-report.md` + `agentic-engineering-reference.md` — both have explicit pre-v0.1 status banners. **KEEP**.
- `docs/social/assets/` (15 promo files) + `_build_*.py` builders (5 files) — large generated assets (PNG/GIF/SVG combined ~MB). **MOVE builders to `scripts/social/`** (currently in docs/), keep PNGs in docs. Or accept the convention.
- `docs/javascripts/mermaid-render.js`, `docs/stylesheets/extra.css`, `docs/.banned-words.toml` — **KEEP** all (load-bearing for the mkdocs build).

**tests/ orphans:**

No orphan fixtures found. Each fixture is referenced by at least one test file:

- `crates/reposix-cache/fixtures/cache_schema.sql` — read by `audit_is_append_only.rs`, `gc.rs`, etc.
- `crates/reposix-core/fixtures/audit.sql` — read by `audit_schema.rs`.
- `crates/reposix-sim/fixtures/seed.json` — sim startup.
- `benchmarks/fixtures/{github_issues,confluence_pages,mcp_jira_catalog}.json` + `.tokens.json` + `reposix_session.txt` — driven by `bench_token_economy.py`.

Test surface (40 integration test files across 9 crates) looks healthy. The `compile-fail/` tests in `reposix-core` (3 cases) and `reposix-cache` (2 cases) are tightly tied to the type-system invariants and properly named.

`crates/reposix-swarm/src/metrics.rs` lines 238 and 240 still mention `fuse` mode in *doc comments* (e.g. `/// Mode name (\`sim-direct\` or \`fuse\`).`). CATALOG-v2 caught this as `FuseWorkload` residue but a follow-up commit deleted the variant; the doc strings are now inaccurate but harmless. **REWRITE** the two comments.

`crates/reposix-cli/src/worktree_helpers.rs` is currently `??` (untracked) per `git status` — likely staged for the in-flight commit; not an orphan.
