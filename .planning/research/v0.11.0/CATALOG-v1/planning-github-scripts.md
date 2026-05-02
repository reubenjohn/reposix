← [back to index](./index.md)

# Bucket-by-bucket review — `.planning/`, `.github/`, `.claude/`, `scripts/`, `benchmarks/`, root files

## `.planning/` review

#### Sources of truth (active)

| File | Disposition | Notes |
|---|---|---|
| `STATE.md` | keep | The cursor. |
| `PROJECT.md` | keep | Active milestone v0.10.0 + validated v0.9.0 sections. |
| `REQUIREMENTS.md` | keep | DOCS-01..11 active; v0.9.0 ARCH-* validated. |
| `MILESTONES.md` | keep | v0.9.0 entry. |
| `ROADMAP.md` | keep | Phases 40–45 detailed. |
| `RETROSPECTIVE.md` | keep | Living doc; historical FUSE-era milestones expected. |
| `v0.9.0-MILESTONE-AUDIT.md` | keep | The "tech_debt" verdict + carry-forward items. |
| `config.json` | keep | GSD configuration. |

#### Phase records (active phase dirs in `.planning/phases/`)
- `30-docs-ia-and-narrative-overhaul-...` — Phase 30 was deferred from v0.9.0 to v0.10.0 (renumbered to Phases 40–45). The phase 30 dir contains `09 PLAN.md` files + `OVERVIEW`, `RESEARCH`, `VALIDATION`, `PATTERNS`. **Disposition:** investigate — either fold this into the milestone archive (move to `.planning/milestones/v0.9.0-phases/30-...` like the other v0.9.0 phases), or delete after extracting any still-applicable content. STATE.md notes: "Legacy Phase 30 entry retained in ROADMAP.md as `<details>` traceability block but not executed." Strong recommendation: **move to `.planning/milestones/v0.9.0-phases/`** to match the rest of the historical convention.
- `40, 41, 42, 43` — active v0.10.0 phases. Each has CONTEXT.md + VERIFICATION.md so far.

#### `.planning/notes/`
| File | Disposition | Notes |
|---|---|---|
| `phase-30-narrative-vignettes.md` | keep (annotate) | Status `ready-for-phase-30-planning` is stale; banned-word list inside is for the FUSE era. REQUIREMENTS.md already has the revised git-native banned-word list. Add a header note: "Banned-word list superseded by REQUIREMENTS.md DOCS-07; vignette V1 still applicable." |
| `gsd-feedback.md` | keep | meta-tool feedback log |

#### `.planning/research/`
| Dir / File | Disposition | Notes |
|---|---|---|
| `v0.1-fuse-era/` (4 files) | keep | Path explicitly marks era; all FUSE refs valid. |
| `v0.9-fuse-to-git-native/` (9 files + `poc/` 4 files) | keep | The pivot design corpus; load-bearing for the architectural argument. POC artefacts are reproducible by `run-poc*.sh`. |
| `v0.10.0-post-pivot/milestone-plan.md` | keep | Active milestone source-of-truth. |

#### `.planning/milestones/`
**Disposition (default):** keep. 307 archived phase records from v0.1.0 → v0.9.0. Treat as a write-once log; do not edit. Subdirs are conventional (e.g. `v0.7.0-phases/22-bench-token-economy.../...`). Two top-level files in this dir are slightly off-pattern:
- `v0.8.0-REQUIREMENTS.md`, `v0.8.0-ROADMAP.md`, `v0.9.0-ROADMAP.md` — fine; per-milestone snapshots taken at archive time.

Per-milestone dir summary:
| Milestone | Files | Notes |
|---|---:|---|
| `v0.1.0-phases/` | 38 | original sim+FUSE+helper work |
| `v0.2.0-alpha-phases/` | 4 | github read-only adapter |
| `v0.3.0-phases/` | 19 | confluence read |
| `v0.4.0-phases/` | 24 | nested mount layout |
| `v0.5.0-phases/` | 16 | bucket `_INDEX.md` |
| `v0.6.0-phases/` | 68 | confluence write + tree-recursive index + refresh + others |
| `v0.7.0-phases/` | 63 | hardening + comments + attachments + whiteboards + docs reorg |
| `v0.8.0-phases/` | 31 | JIRA |
| `v0.9.0-phases/` | 41 | architecture pivot |

#### `.planning/archive/scripts/` (3 files)
**Disposition:** keep. Pre-v0.1 phase exit scripts; historical.

#### `.planning/SESSION-*.md`
- `SESSION-5-RATIONALE.md` (Phase 14 rationale) — keep, historical
- `SESSION-7-BRIEF.md` (Phase 16+ session brief) — keep, historical

---

## `.github/` review

| File | Disposition | Notes |
|---|---|---|
| `workflows/ci.yml` | keep (minor) | dark-factory job + integration-contract-{conf,gh,jira}-v09 jobs are live. The `integration-contract` (legacy github octocat) test is still wired alongside the new `-v09` variants — **redundant**; consolidate. |
| `workflows/docs.yml` | keep | clean; `mkdocs build --strict` + gh-deploy. |
| `workflows/release.yml` | rewrite-needed | **Broken:** tarball-staging step (line 75) `for bin in reposix reposix-sim reposix-fuse git-remote-reposix; do cp ... ; done` will fail because `reposix-fuse` no longer builds. Drop `reposix-fuse` from the binary list. |

---

## `.claude/` review

| File | Disposition | Notes |
|---|---|---|
| `settings.json` | keep | minimal allow-list |
| `skills/reposix-agent-flow/SKILL.md` | keep | The dark-factory regression skill; load-bearing for OP-4 (self-improving infrastructure) per CLAUDE.md. |

---

## `scripts/` review

| File | Disposition | Notes |
|---|---|---|
| `bench_token_economy.py` | keep | counts tokens via Anthropic API |
| `test_bench_token_economy.py` | keep | pytest harness |
| `check_clippy_lint_loaded.sh` | keep | invariant test for `clippy.toml` |
| `check_fixtures.py` | keep | benchmark fixture validator |
| `dark-factory-test.sh` | keep | the v0.9.0 regression script |
| `demo.sh` | rewrite-needed | shim that execs `scripts/demos/full.sh` (FUSE-era). Either delete the shim or repoint to `dark-factory-test.sh`. |
| `green-gauntlet.sh` | keep (audit) | full pre-tag check; verify no FUSE refs remain inside |
| `install-hooks.sh` | keep | git hooks installer |
| `tag-v0.3.0.sh` … `tag-v0.9.0.sh` (7 scripts) | keep | release-tag scripts; auditable |
| `v0.9.0-latency.sh` | keep | latency-table regenerator |
| `dev/list-confluence-spaces.sh` | keep | dev tool |
| `dev/probe-confluence.sh` | keep | dev tool |
| `dev/test-bucket-index.sh` | **delete** | tested FUSE bucket `_INDEX.md` synthesis — feature deleted with `reposix-fuse` |
| `dev/test-tree-index.sh` | **delete** | tested FUSE `tree/` overlay — feature deleted with `reposix-fuse` |
| `migrations/fix_demos_index_links.py` | keep | one-shot migration |
| `migrations/mermaid_divs_to_fences.py` | keep | one-shot migration |
| `hooks/pre-push` | keep | credential-leak guard |
| `hooks/test-pre-push.sh` | keep | hook self-test (wired into CI) |
| `demos/01-edit-and-push.sh` | rewrite-needed | uses `reposix mount` + `reposix-fuse` |
| `demos/02-guardrails.sh` | rewrite-needed | uses `reposix mount` |
| `demos/03-conflict-resolution.sh` | rewrite-needed | uses `reposix mount` (per `IssueBackend`-trace mention) |
| `demos/04-token-economy.sh` | keep (audit) | benchmark; verify it doesn't depend on the FUSE mount |
| `demos/05-mount-real-github.sh` | **delete** | mounts real GitHub via FUSE |
| `demos/06-mount-real-confluence.sh` | **delete** | mounts real Confluence via FUSE |
| `demos/07-mount-real-confluence-tree.sh` | **delete** | tree/ overlay demo for FUSE |
| `demos/08-full-backend-showcase.sh` | rewrite-needed | full-backend showcase; uses FUSE |
| `demos/_lib.sh`, `_record.sh` | keep (audit) | helpers; ensure they don't bake in FUSE assumptions |
| `demos/assert.sh` | keep | smoke-test asserter |
| `demos/full.sh` | **delete** | the 9-step Tier 2 walkthrough; entirely FUSE-based; replaced by `scripts/dark-factory-test.sh` |
| `demos/parity.sh` + `parity-confluence.sh` | keep (audit) | sim-vs-real parity comparison; should not depend on FUSE — verify |
| `demos/smoke.sh` | rewrite-needed | runs the Tier-1 demos; rewire to the surviving subset (or delete altogether) |
| `demos/swarm.sh` | keep (audit) | swarm-mode demo; verify it uses `sim-direct` not `fuse` mode |

---

## `benchmarks/` review

| File | Disposition | Notes |
|---|---|---|
| `README.md` | keep | clean |
| `RESULTS.md` | keep | historical results (v0.7-era 89.1% number) — current; v0.10.0 may add a v0.9.0-vintage refresh |
| `fixtures/README.md` | keep | clean |
| `fixtures/{github_issues, confluence_pages, mcp_jira_catalog, reposix_session}.{json/txt}` + `.tokens.json` siblings (8 files) | keep | benchmark fixtures + cached token counts |

### `requirements-bench.txt`
**Disposition:** keep. Python deps for the token-economy benchmark.

---

## Root files

| File | Disposition | Notes |
|---|---|---|
| `Cargo.toml` | keep | clean; workspace v0.9.0; nine members |
| `Cargo.lock` | keep | committed |
| `rust-toolchain.toml` | keep | pins stable |
| `rustfmt.toml` | keep | clean |
| `clippy.toml` | keep | the `disallowed-methods` ban for `reqwest::Client::*` |
| `LICENSE-MIT`, `LICENSE-APACHE` | keep | dual license |
| `.gitignore` | keep | clean |
| `.env.example` | keep | clean |
| `mkdocs.yml` | keep | nav fully reflects post-pivot site (Concepts / How it works / Tutorials / Guides / Reference / Benchmarks / Decisions / Research). Redirect stubs are excluded via `not_in_nav`. |
| `README.md` | rewrite-needed | 27 FUSE refs; "Quickstart (v0.7.x — pre-FUSE-deletion)" section, the "Folder-structure mount" section, the swarm-tree mention, the demo gif annotation, the prebuilt-binary list — all need v0.9.0-shaped updates. **Owner-flagged for Phase 45.** |
| `CHANGELOG.md` | keep | post-pivot v0.9.0 entry is authoritative; older FUSE-era entries are correctly historical. |
| `CLAUDE.md` | keep | rewritten in Phase 36 to steady-state git-native; one transitional reference to `crates/reposix-fuse` "BEING DELETED in v0.9.0" remaining — minor cleanup once everyone agrees the deletion has fully landed. |
| `HANDOFF.md` | **delete** | v0.7-era doc; superseded by `.planning/STATE.md` + `.planning/MILESTONES.md` + `.planning/v0.9.0-MILESTONE-AUDIT.md`. Contains 26 FUSE refs, `OP-1..OP-11` (all closed), Phase 27+ direction (now shipped). Anything still load-bearing should migrate to `STATE.md` or to a `docs/development/` page first. |
