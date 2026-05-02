# v0.11.0 CATALOG-v2 (cold review)

> **Cold-eyed re-audit, 2026-04-25 morning.** Read-only walk of all 679 git-tracked files. The previous overnight `.planning/CATALOG.md` is taken as input but every claim was re-verified against the current tree; several CATALOG-v1 statements are now stale (FUSE-era release.yml line 75 is fixed; `crates/reposix-swarm/src/fuse_mode.rs` is already deleted; `Mode::Fuse` is gone). This catalog supersedes v1 for the v0.11.0 cleanup pass.

## Headline

- **679** files audited (vs CATALOG-v1's 619 — an additional 60 net-new files landed overnight: 3 new ADRs, 3 new CLI subcommands and tests, 5 examples, blog post, RELEASE-NOTES, asciinema script, etc.).
- **DELETE: 24** files. Mostly FUSE-era demo scripts/recordings, broken stubs, orphan screenshots, archival walkthrough.
- **REFACTOR: 38** files. Tech debt clusters around (1) FUSE residue in code/doc comments, (2) duplicated `cache_path_from_worktree`/`backend_slug_from_origin`/`git_config_get` triplet across 4 CLI modules, (3) `refresh.rs`'s dead FUSE guard.
- **REVIEW: 11** files. Owner judgement calls (blog post wiring, screenshots, release-notes location, tag-script duplication).
- **KEEP: ~605** files including all of `.planning/milestones/` (write-once log).

## Top recommendations (the 10 highest-value moves)

The owner explicitly invited cold-hard decisions. These are ordered by ratio of "improves new-reader grasp" to "lines of work":

1. **Bump `Cargo.toml` workspace `version` from `0.9.0` to `0.10.0`** (and follow with `0.11.0-pre`). v0.10.0 shipped + tagged but the package metadata still says 0.9.0. `cargo run -p reposix-cli -- --version` lies. **One-line fix; high embarrassment-on-discovery cost.**
2. **Delete `scripts/demo.sh`.** It's a shim execing `scripts/demos/full.sh`, which was deleted in the v0.9.0 pivot. A new contributor running `bash scripts/demo.sh` gets a confusing `No such file or directory`. The CHANGELOG link from v0.2.0-alpha is the only reason to keep it; cite history in the changelog instead.
3. **Delete the entire FUSE-era demo set under `scripts/demos/`** (`01-edit-and-push.sh`, `02-guardrails.sh`, `03-conflict-resolution.sh`, `08-full-backend-showcase.sh`, `_lib.sh`, `_record.sh`, `assert.sh`, `parity.sh`, `parity-confluence.sh`, `smoke.sh`, `swarm.sh`, `04-token-economy.sh` keep). All except 04 invoke `reposix-fuse` / `reposix mount`; replaced by `scripts/dark-factory-test.sh` + the agent-flow skill. Six scripts can be deleted outright; replace `_lib.sh`/`assert.sh` if you need them for `04-token-economy.sh`.
4. **Delete the FUSE-era recordings under `docs/demos/recordings/`** plus the orphaned `docs/demo.transcript.txt` + `docs/demo.typescript` at root. Each recording lists `reposix-fuse` as a workspace crate. They have ZERO inbound links from the live mkdocs nav (`demos/index.md` is in `not_in_nav`). Cold reader who finds them will be confused. Re-record after v0.11.0 if needed; until then, delete.
5. **Delete `docs/demos/index.md`.** Tier 1/2/3 demo lineup explicitly references the deleted `scripts/demos/full.sh`. Already in `not_in_nav`. The new equivalent is `docs/tutorials/first-run.md` + the `examples/` directory. One stale page, four dead links.
6. **Consolidate the four `cache_path_from_worktree` / `backend_slug_from_origin` / `git_config_get` triplets** into `crates/reposix-cli/src/worktree.rs`. They're verbatim-copied across `doctor.rs`, `history.rs`, `gc.rs`, `tokens.rs`. Each duplication is ~50 lines including tests; net saving is ~150 lines and one source of truth for how the CLI maps a working tree → cache path. **Single highest-leverage code refactor.**
7. **Extract / inline `crates/reposix-cache/src/cli_compat.rs`.** It's a 252-line v0.8 `refresh_meta` schema clearly tagged "Phase-31 Plan-02 holdover; refresh subcommand will migrate in Phase 35." Phase 35 shipped without the migration (the cache crate has BOTH a `cli_compat` module and a `db` module, two SQLite schemas). Either (a) migrate `refresh.rs` to use `cache::db` and delete `cli_compat.rs`, or (b) move `cli_compat` back into `reposix-cli` as `cache_db.rs` (where it lived pre-Phase-31). Current placement contaminates the cache crate's public API.
8. **Delete `refresh.rs`'s `is_fuse_active` guard + `mount_point` field name.** It checks for `.reposix/fuse.pid` which is never created in v0.9.0+. The check IS harmless (always returns `false`) but the doc comment, error message, and field name are pure FUSE residue that confuses any reader of `cargo doc`. Rename `mount_point: PathBuf` → `working_tree: PathBuf` in lockstep. Catalog v1 noted this; it remains undone.
9. **Delete `MORNING-WALKTHROUGH-2026-04-25.md`, `RELEASE-NOTES-v0.10.0.md`, `RELEASE-NOTES-v0.11.0-PREVIEW.md`, `PUBLIC-LAUNCH-CHECKLIST.md` from the repo root.** They were cathartic to write but they are session journals and one-off launch checklists. CHANGELOG.md already covers the release notes. The walkthrough is dated and obsolete. Move to `.planning/archive/2026-04-25-launch/` if archival value is desired; otherwise just delete (git history preserves them).
10. **Sweep the ~12 FUSE-era doc comments out of source files in one mechanical pass.** `crates/reposix-core/src/lib.rs` (line 3 "FUSE daemon"), `core/src/path.rs` (lines 3,13), `core/src/backend.rs` (lines 7,9,22,72,106), `core/src/backend/sim.rs` (lines 499,629-632,909), `core/src/project.rs` (line 8), `confluence/src/lib.rs` (lines 433, 1484), `cli/src/refresh.rs` (lines 55, 63, 75, 77, 248-275, 368-383), `remote/src/fast_import.rs` (line 5). All cosmetic but they're the first thing a new reader sees in `cargo doc`. Single-PR ~30-line diff.

---

## Chapters

- [DELETE](./delete.md)
- [REFACTOR](./refactor.md)
- [REVIEW + KEEP](./review-keep.md)
- [Architecture + Naming + Closing](./architecture.md)
