# reposix file catalogue (2026-04-24/25)

**Total tracked files:** 619
**Auditor:** general-purpose subagent for the overnight polish session
**Authoritative:** this file is a SNAPSHOT — git history is canonical source.

> **Scope.** Read-only audit of every git-tracked file. Decisions on each: keep / rename / move / delete / rewrite-needed / investigate. The owner is preparing reposix for a publishable state after the v0.9.0 architecture pivot from FUSE to git-native partial clone; the v0.10.0 milestone (Docs & Narrative Shine) is currently scaffolded.
>
> **Working assumption.** v0.9.0 deleted `crates/reposix-fuse/` entirely. References to FUSE / `fuser` / `fusermount3` / `reposix mount` / `reposix demo` / `reposix-fuse` binary in *active code paths*, *user-facing docs*, *current CI*, and *current CLI surface* are stale. References to those terms in `.planning/milestones/` (archived phase records), `CHANGELOG.md` historical sections, `.planning/research/v0.1-fuse-era/`, and `docs/archive/` are correctly historical and stay.

---

## Top-level disposition matrix

| Bucket | Files | Default disposition | Exceptions |
|---|---:|---|---|
| `crates/reposix-core/` | 23 | keep | `IssueId`/`Issue` rename (owner-flagged), FUSE comments in `backend.rs` + `path.rs` + `backend/sim.rs` need rewrite |
| `crates/reposix-cache/` | 25 | keep | none — clean post-pivot crate (Phase 31) |
| `crates/reposix-cli/` | 14 | keep | `refresh.rs` retains `mount_point` field name + FUSE-active guard; `cli.rs` test still uses `mount_point` arg name; FUSE-residue migration tests are appropriate |
| `crates/reposix-remote/` | 11 | keep | `main.rs` hardcodes `SimBackend` (known tech debt — see `v0.9.0-MILESTONE-AUDIT.md` §1); `fast_import.rs` mentions FUSE in comment |
| `crates/reposix-sim/` | 15 | keep | clean |
| `crates/reposix-confluence/` | 5 | keep | `lib.rs` doc comment mentions FUSE history |
| `crates/reposix-github/` | 3 | keep | clean |
| `crates/reposix-jira/` | 4 | keep | clean |
| `crates/reposix-swarm/` | 14 | rewrite-needed | `fuse_mode.rs` + `Mode::Fuse` enum variant + `FuseWorkload` use are **dead code** (FUSE crate deleted) |
| `docs/concepts/` | 2 | keep | post-pivot, clean |
| `docs/how-it-works/` | 3 | keep | post-pivot, clean |
| `docs/tutorials/` | 1 | keep | post-pivot, clean |
| `docs/guides/` | 3 | keep | post-pivot, clean |
| `docs/reference/` | 8 | rewrite-needed | `cli.md`, `crates.md`, `git-remote.md`, `confluence.md`, `jira.md`, `http-api.md` all FUSE-era; `simulator.md` + `testing-targets.md` clean |
| `docs/decisions/` | 5 | keep (annotate) | ADR-001/002/003 reference deleted FUSE layer; need scope-superseded notice; ADR-004/005 are recent |
| `docs/research/` | 2 | keep | both have explicit pre-v0.1 status banners; correctly historical |
| `docs/archive/` | 2 | keep | already labelled archived |
| `docs/development/` | 2 | rewrite-needed | `roadmap.md` stops at v0.7; `contributing.md` lists `reposix-fuse/` |
| `docs/connectors/`, `docs/architecture.md`, `docs/security.md`, `docs/why.md`, `docs/demo.md` | 5 | keep | redirect-stub pages — all already converted to "(moved)" pointers |
| `docs/index.md` | 1 | keep | post-pivot hero, clean |
| `docs/benchmarks/v0.9.0-latency.md` | 1 | keep | sim column populated; real cells `pending-secrets` |
| `docs/social/` | 17 | keep | promo assets; demo gif is FUSE-era (flagged but low priority) |
| `docs/screenshots/` | 6 | keep | mostly v0.2/landing |
| `docs/demos/` | 13 | rewrite-needed | `index.md` heavily FUSE-era; recordings are FUSE-era artefacts (delete or re-record per Phase 45) |
| `scripts/demos/*.sh` | 16 | rewrite-needed/delete | most reference deleted `reposix mount`/`reposix-fuse` binary; `dark-factory-test.sh` replaces them |
| `scripts/dev/*.sh` | 4 | delete | `test-bucket-index.sh`, `test-tree-index.sh`, `probe-confluence.sh`, `list-confluence-spaces.sh` — `bucket`/`tree` index were FUSE features |
| `scripts/migrations/*.py` | 2 | keep | one-time migrations; harmless |
| `scripts/hooks/*` | 2 | keep | pre-push credential guard still relevant |
| `scripts/tag-v0.X.0.sh` | 7 | keep | each gates its own historical release (auditable) |
| `scripts/*` other | 8 | keep mostly | `demo.sh` (top-level shim) is FUSE-era — delete or repoint to `dark-factory-test.sh` |
| `benchmarks/` | 11 | keep | token-economy benchmark; all four fixture pairs current |
| `.github/workflows/` | 3 | rewrite-needed | `release.yml` still tarballs `reposix-fuse` binary (broken); `ci.yml` recently updated; `docs.yml` clean |
| `.claude/` | 2 | keep | `settings.json`, `reposix-agent-flow/SKILL.md` (post-pivot) |
| `.planning/` (top + active phases + active milestone) | 14 | keep | living planning artefacts |
| `.planning/milestones/v0.X.0-phases/` | 307 | keep | historical phase records — archival; do not edit |
| `.planning/research/v0.1-fuse-era/` | 4 | keep | path explicitly marks era |
| `.planning/research/v0.9-fuse-to-git-native/` | 9 | keep | the design doc set for the pivot — load-bearing |
| `.planning/research/v0.10.0-post-pivot/` | 1 | keep | active milestone research |
| `.planning/notes/` | 2 | keep (annotate) | `phase-30-narrative-vignettes.md` predates pivot; banned-word list updated in REQUIREMENTS — note that |
| `.planning/archive/scripts/` | 3 | keep | historical |
| Root: `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml`, `LICENSE-{MIT,APACHE}`, `.gitignore`, `.env.example` | 9 | keep | clean |
| Root: `README.md`, `CHANGELOG.md`, `mkdocs.yml`, `CLAUDE.md` | 4 | rewrite-needed (README) / keep (others) | README has FUSE-era Quickstart sections + 27 FUSE refs |
| Root: `HANDOFF.md` | 1 | delete | v0.7-era doc; superseded by `.planning/STATE.md` + `.planning/MILESTONES.md` + `v0.9.0-MILESTONE-AUDIT.md` |

---

## Chapters

- **[Crates bucket-by-bucket review](./crates.md)** — per-file dispositions for all `crates/reposix-*` workspace members (core, cache, cli, remote, sim, confluence, github, jira, swarm).
- **[Docs bucket-by-bucket review](./docs.md)** — per-file dispositions for `docs/` subtrees (index, concepts, how-it-works, tutorials, guides, reference, decisions, research, archive, development, demos, social, screenshots, benchmarks).
- **[Planning, GitHub, Claude, Scripts, Root review](./planning-github-scripts.md)** — per-file dispositions for `.planning/`, `.github/workflows/`, `.claude/`, `scripts/`, `benchmarks/`, and root files.
- **[Cleanup decisions — prioritized work list](./cleanup-decisions.md)** — must-do-before-launch items, should-do-this-milestone items, and nice-to-have backlog.
- **[Stale refs audit, naming generalization, recommended sequencing](./stale-naming-sequencing.md)** — grep-pass stale-reference counts, `IssueId`/`Issue`/`IssueStatus` rename candidates, and suggested milestone sequencing.
