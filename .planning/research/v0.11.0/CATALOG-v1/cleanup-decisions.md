← [back to index](./index.md)

# Cleanup decisions — prioritized work list

## Must do before public launch

1. **Fix release.yml** — `.github/workflows/release.yml:75` lists `reposix-fuse` in the binary tarball; this will break the next `git push v0.10.0`. One-line fix.
2. **Delete `crates/reposix-swarm/src/fuse_mode.rs` + `Mode::Fuse` enum + `FuseWorkload` references** — dead code that imports the deleted `reposix-fuse` crate is a ticking compile bomb. (Currently survives because the swarm crate doesn't actually depend on `reposix-fuse`; it `std::fs::*`s a path. Still: dead code.)
3. **Delete `scripts/demos/full.sh`, `scripts/demos/05-mount-real-github.sh`, `scripts/demos/06-mount-real-confluence.sh`, `scripts/demos/07-mount-real-confluence-tree.sh`, `scripts/dev/test-bucket-index.sh`, `scripts/dev/test-tree-index.sh`** — all reference the deleted `reposix-fuse` binary; if a contributor `bash`-runs any of them the error will be confusing.
4. **Rewrite `docs/reference/cli.md`** — currently advertises the deleted `mount` and `demo` subcommands. This is the single most user-misleading doc page in the repo.
5. **Rewrite `docs/reference/crates.md`** — lists `reposix-fuse` as a workspace crate; missing `reposix-cache` + `reposix-jira`; uses old `IssueBackend` name. This is the discoverability page for the Rust API surface.
6. **Delete `HANDOFF.md`** — v0.7-era stub doc with stale OP-1..OP-11 closed-items table that pretends to be the source of truth (the actual source is `.planning/STATE.md`). Risk: a new contributor reads HANDOFF and thinks the project is at v0.7.
7. **Rewrite `README.md` Quickstart section** — already partially flagged as "v0.10.0 Phase 45". The "Quickstart (v0.7.x — pre-FUSE-deletion)" section should be removed or moved to a History block; the new five-line quickstart for `reposix init sim::demo` should be the first runnable thing.

## Should do this milestone (v0.10.0 — Docs & Narrative Shine)

1. **Annotate ADR-001/002/003** with "scope-superseded by v0.9.0 partial-clone" headers. The mappings (status, page-id) are still authoritative; only the rendering layer changed.
2. **Update `docs/development/contributing.md`** — workspace tree listing + non-negotiable invariants (`#1 references FUSE callbacks`).
3. **Update `docs/development/roadmap.md`** — extend through v0.8 / v0.9 / v0.10 (currently stops at v0.7); revise long-term north stars (macFUSE / ProjFS no longer applicable).
4. **Rewrite `docs/reference/git-remote.md`** — add `stateless-connect` capability; remove FUSE references; refresh the v0.2 backlog section.
5. **Rewrite `docs/reference/{confluence,jira}.md`** — replace "Mount as FUSE" sections with `reposix init` flows. Drop the `reposix mount` CLI line.
6. **Rewrite `docs/demos/index.md`** — recast the demo lineup around `reposix init` + git workflow; the FUSE-era recordings can stay as archived assets but the index page must lead with the post-pivot UX.
7. **Move `.planning/phases/30-docs-ia-and-narrative-overhaul-...` into `.planning/milestones/v0.9.0-phases/`** — it's a deferred-then-superseded phase; living in `phases/` next to the active 40–45 dirs is confusing.
8. **Rewrite `scripts/demos/01-edit-and-push.sh`, `02-guardrails.sh`, `03-conflict-resolution.sh`, `08-full-backend-showcase.sh`** — into the post-pivot equivalents (`reposix init` + git-push round-trip). The `dark-factory-test.sh` is the template.
9. **Rewrite `scripts/demo.sh`** — currently a shim to `demos/full.sh`; either delete or repoint to `dark-factory-test.sh`.
10. **Resolve helper-hardcodes-SimBackend** (audit §1, audit-v0.9.0). Tracked for v0.10.0/v0.11.0 per STATE.md.

## Nice-to-have backlog

1. **`IssueId` → `RecordId` rename.** Owner-flagged. ~304 call sites; biggest type-rename in the project. See "Naming generalization" section. Plan as a dedicated phase; coordinate with frontmatter compat (the YAML field name is `id` so on-disk format is unaffected).
2. **`Issue` → `Record` rename** (or keep `Issue` as one valid record kind; promote `Record` as alias). Pre-1.0 hard rename is acceptable per the project's history (cf. ADR-004 `IssueBackend → BackendConnector`).
3. **Consolidate `integration-contract` and `integration-contract-github-v09`** — both hit GitHub. Drop the legacy job once the v09 variant is proven stable.
4. **`crates/reposix-cli/src/refresh.rs`** — rename `mount_point` field to `working_tree`; delete `is_fuse_active` guard (looks for a never-created `.reposix/fuse.pid` file). Lock-step rename in `cli.rs` + `refresh_integration.rs`.
5. **Re-record demo gifs.** README itself flags `docs/social/assets/demo.gif` as "FUSE-era — Phase 45 will re-record". Same for `architecture.mmd`/`.png`.
6. **Annotate `.planning/notes/phase-30-narrative-vignettes.md`** with "banned-word list updated in REQUIREMENTS.md DOCS-07 for git-native; vignette V1 still applicable".
7. **`docs/research/initial-report.md` + `agentic-engineering-reference.md`** are stylistically dense and academic. Owner consideration: they're cited from CLAUDE.md as "the architectural argument"; some readers will bounce off them. Lower priority than user-facing surface.
8. **Sweep `crates/reposix-{core,confluence,remote}/src/*.rs` doc comments** for "FUSE" / "mount" framing in module-level docs. Cosmetic but signals the post-pivot maturity. ~10 lines total.
