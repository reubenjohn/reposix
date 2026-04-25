---
phase: 31
plan: 01
status: complete
completed_at: 2026-04-24
---

# Phase 31 Plan 01 — Summary

## Objective achieved

Scaffolded `crates/reposix-cache` and landed the ARCH-01 substrate:
`Cache::open` + `Cache::build_from` producing a valid bare git repo on
disk with a populated tree (one `issues/<id>.md` entry per seeded
issue) and zero blob objects (lazy-blob invariant).

## Tasks completed

- **Task 1** — Workspace added `crates/reposix-cache` as a member and
  `gix = "=0.82.0"` + `dirs = "6"` to workspace dependencies. New
  crate manifest with `[dev-dependencies] trybuild = "1"` for later
  plans. `gix_api_smoke` test pins the 0.82 API surface used by the
  builder.
- **Task 2** — Modules `error`, `path`, `cache`, `builder`; two
  integration tests proving tree correctness and blob laziness.

## Commits

- `ee48a46` feat(31-01): scaffold reposix-cache crate with gix 0.82 API smoke test
- `d475c88` feat(31-01): implement `Cache::build_from` — lazy-blob tree + commit

## Tests added

- `gix_api_smoke.rs` — 1 test (`gix_082_exposes_expected_surface`).
- `tree_contains_all_issues.rs` — 2 tests (N=1 and N=10).
- `blobs_are_lazy.rs` — 1 test.
- `path::tests::env_var_wins` — unit test.

Total new tests: **5**.

## gix 0.82 API surface (confirmed)

| Use | Signature / location |
| --- | --- |
| Init bare | `gix::init_bare(&Path) -> Result<Repository, init::Error>` |
| Hash object (no-persist) | `gix::objs::compute_hash(hash_kind, object::Kind, &[u8]) -> Result<ObjectId, _>` |
| Write blob (persist) | `Repository::write_blob(&[u8]) -> Result<Id, _>` |
| Write tree object | `Repository::write_object(impl WriteTo) -> Result<Id, _>` |
| Commit to ref | `Repository::commit(ref, msg, tree, parents) -> Result<Id, _>` |
| Edit tree (NOT used) | `Repository::edit_tree(ObjectId)` — requires the `tree-editor` feature AND validates that every referenced blob already exists. We enable the feature (to satisfy gix_api_smoke) but bypass the editor in `build_from` because the lazy invariant requires writing trees whose blobs have NOT been persisted. Hand-build `gix::objs::Tree` + `write_object` instead. |

## Deviations from the plan sketch

1. **Integration tests' backend.** The plan assumed
   `reposix_sim::SimBackend::new_seeded(project, N)`. In reality
   `SimBackend` lives at `reposix_core::backend::sim::SimBackend` and
   requires an HTTP origin. The test harness (`tests/common/mod.rs`)
   uses `wiremock::MockServer` to serve the sim's
   `GET /projects/<p>/issues[/<id>]` shape; `SimBackend::new(uri)`
   then talks to it. Keeps the simulator-first principle (no real
   egress) and reuses the sanctioned backend client instead of
   hand-rolling a `BackendConnector`.
2. **`edit_tree` bypass.** Plan text suggested using
   `Repository::edit_tree().upsert().write()`. That API validates
   referenced objects exist, fatally breaking the lazy invariant. We
   construct `gix::objs::Tree` + `Entry` directly and write via
   `write_object`. Documented inline in `builder.rs`.
3. **`Tainted` return type** is deferred to Plan 02's `read_blob`.
   Plan 01 does NOT expose any path returning user bytes.
4. **Cargo feature flag.** Enabled `gix` feature `tree-editor` on the
   crate because the smoke test verifies the editor surface even
   though `build_from` doesn't use it. No runtime cost; the smoke
   test is the pinning mechanism for Pitfall 4.

## Notes for Plan 02

- Add `read_blob` to `impl Cache` in `builder.rs` (same file is
  fine — keeps the builder-vs-materialize pair colocated).
- Plan 02 needs a `Mutex<rusqlite::Connection>` field on `Cache` per
  the plan; the existing struct already uses `pub(crate)` fields so
  additions are straightforward.
- The `Error::Sqlite(String)` variant is already scaffolded via
  `From<rusqlite::Error>` mapping.
- Public API landed: `reposix_cache::{Cache, Error, Result,
  resolve_cache_path, CACHE_DIR_ENV}`. Phase 32 consumers should
  construct via `Cache::open(Arc<dyn BackendConnector>,
  backend_name, project)`.
- gix 0.82 `find_reference` returns a `Reference`, `peel_to_id()` is
  the current method (`peel_to_id_in_place` is deprecated). Tests use
  the non-deprecated path.

## Acceptance status

All 12 acceptance criteria from Task 1 + Task 2 satisfied. `cargo
check --workspace`, `cargo clippy -p reposix-cache --all-targets --
-D warnings`, and `cargo test -p reposix-cache` all green.
