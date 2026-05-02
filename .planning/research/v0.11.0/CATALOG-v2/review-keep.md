# REVIEW + KEEP

← [index](./index.md)

## REVIEW

| Path | Owner judgement needed because… |
|---|---|
| `/home/reuben/workspace/reposix/docs/blog/2026-04-25-reposix-launch.md` | Owner explicitly flagged: "owner may not want a blog directory at all." It's a 357-line launch post. NOT wired into mkdocs nav. Either (a) wire it in via Material blog plugin (full treatment), (b) move to `docs/release-history/v0.10.0-launch.md` (release-anchored), or (c) move to a personal blog repo and just link from CHANGELOG. |
| `/home/reuben/workspace/reposix/docs/demos/asciinema-script.md` | A 200-line shell-paste-ready screenplay for recording the launch screencast. It's a one-shot script masquerading as docs. Once the screencast is recorded and uploaded, this file's value drops to zero. **Decision:** keep until launch+1d, then delete or move to `.planning/archive/asciinema-launch.md`. |
| `/home/reuben/workspace/reposix/docs/screenshots/site-home.png` and `site-security.png` | Catalog-v1 says "keep" (refresh in Phase 44/45). I find no inbound link from any live page. If they're not promoted into README/index.md by v0.11.0, **delete**. |
| `/home/reuben/workspace/reposix/docs/social/twitter.md`, `linkedin.md` | Promo copy. Useful for launch only. After launch, **either move to `.planning/archive/social-2026-04-25/` or delete**. |
| `/home/reuben/workspace/reposix/scripts/take-screenshots.sh` | Stub that names a contract for screenshots that "deferred (cairo system libs unavailable)" per STATE.md. If never executed, **delete**; otherwise document the cairo prereq in `docs/development/` (or its successor). |
| `/home/reuben/workspace/reposix/.planning/MILESTONES.md`, `RETROSPECTIVE.md` | Both are living docs; no timestamping. Owner judgement: are these still tools the orchestrator + GSD agents read, or are they superseded by `STATE.md` + per-milestone audits? |
| `/home/reuben/workspace/reposix/scripts/v0.9.0-latency.sh` | One-shot latency-table regenerator for `docs/benchmarks/v0.9.0-latency.md`. As of v0.10.0, the table has `pending-secrets` cells; will it be re-run for v0.11.0 with real-backend numbers? If yes, **rename to `scripts/bench-latency.sh`** (parameterize over version). If no, **delete after the table is finalized**. |
| `/home/reuben/workspace/reposix/scripts/dev/list-confluence-spaces.sh` | Still functional, but is it ever run in practice? Could be replaced by a `reposix spaces` invocation. If `reposix spaces` covers it, **delete**. |
| `/home/reuben/workspace/reposix/.planning/v0.10.0-MILESTONE-AUDIT.md`, `v0.9.0-MILESTONE-AUDIT.md` | These are auditable artifacts but they sit at `.planning/` root; per pattern they should probably be under `.planning/milestones/v0.X.0-MILESTONE-AUDIT.md`. Owner-judgement on archival convention. |
| `/home/reuben/workspace/reposix/.planning/notes/phase-30-narrative-vignettes.md` | Status `ready-for-phase-30-planning` is stale; banned-word list inside is FUSE-era. CATALOG-v1 says "keep (annotate)" but the document is inert. **Either delete or rewrite to inherit from REQUIREMENTS.md DOCS-07.** |

---

## KEEP (summary, not exhaustive)

| Area | File count | Notes |
|---|---:|---|
| `crates/reposix-core/` (src + tests) | 20 | Clean public API; load-bearing compile-fail tests for SG-01/04/05. |
| `crates/reposix-cache/` (excl. cli_compat.rs) | 23 | Phase-31 baby; `gc.rs`, `sync_tag.rs`, `meta.rs` v0.11-vision-anchored. |
| `crates/reposix-cli/` (excl. FUSE residue + 4-way duplication) | 21 | Well-documented; `agent_flow_real.rs` is dark-factory keystone. |
| `crates/reposix-sim/` | 16 | Clean. |
| `crates/reposix-remote/` | 11 | `backend_dispatch.rs` closes the v0.9.0 carry-forward debt. |
| `crates/reposix-{confluence,github,jira}/` | 12 | Clean apart from cosmetic FUSE comments; ADF logic current. |
| `crates/reposix-swarm/` | 14 | `fuse_mode.rs` deletion already done (CATALOG-v1 outdated). |
| `docs/{index,concepts,how-it-works,tutorials,guides}/` | 11 | Post-v0.10.0 narrative; clean. |
| `docs/decisions/004..008/`, `docs/research/`, `docs/reference/{simulator,testing-targets,jira,confluence}.md`, `docs/benchmarks/v0.9.0-latency.md`, `docs/archive/` | 14 | Live or correctly historical. |
| `examples/` | 12 | Five working examples plus README; clean. |
| `benchmarks/` | 11 | Token-economy benchmark + fixture pairs; clean. |
| `.planning/milestones/` | 307 | Write-once log. |
| `.planning/research/` | 18 | Living research corpus. |
| `.github/` | 11 | `audit.yml`, `docs.yml`, `release.yml` clean (CATALOG-v1's release.yml broken-tarball claim is stale; verified line 77 lists `reposix reposix-sim git-remote-reposix` only — no reposix-fuse). |
| `.claude/` | 3 | Settings + 2 agent skills. Clean. |
| Root: Cargo.toml + 9 config files | 10 | Clean apart from Cargo.toml version-bump miss (top recommendation #1). |
| Root: README/CHANGELOG/CONTRIBUTING/SECURITY/COC/CLAUDE/mkdocs/pre-commit | 8 | Live; clean post-v0.10.0 (apart from `mkdocs.yml` ADR-008 nav-miss). |
| `scripts/` (banned-words-lint, bench_token_economy, check_*, dark-factory-test, green-gauntlet, install-hooks, hooks/, migrations/) | 12 | Live tooling. |
