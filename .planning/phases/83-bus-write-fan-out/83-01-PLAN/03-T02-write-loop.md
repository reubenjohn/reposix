← [back to index](./index.md) · phase 83 plan 01

# Task 83-01-T02 — Lift `handle_export` write loop into `write_loop::apply_writes`

**Goal:** Extract the REST-write logic from `handle_export` (lines 343-606) into a shared `write_loop::apply_writes` function with a narrow-deps signature. The single-backend path (`handle_export`) and bus path (`bus_handler`) will both call this shared function.

**Scope:** Single atomic refactor commit.

**Success criteria:**
1. New `crates/reposix-remote/src/write_loop.rs` module with `apply_writes` function (min 200 lines).
2. `execute_action` signature narrowed to exclude `&mut State`.
3. `handle_export` body shrinks to a wrapper shape: read stdin, call `apply_writes`, emit response lines.
4. Existing single-backend integration tests (mirror_refs, push_conflict, bulk_delete_cap, perf_l1, stateless_connect) all GREEN post-refactor.

## Chapters

- **[Step 2a: New module](./T02-subchs/2a.md)** — Create `crates/reposix-remote/src/write_loop.rs` with the lifted `apply_writes` function and `WriteOutcome` enum.

- **[Steps 2b–2c: Refactor signatures](./T02-subchs/2b-2c.md)** — Narrow `execute_action` to exclude `&mut State`, replace `handle_export` body with wrapper shape.

- **[Steps 2d–2f: Finalize and commit](./T02-subchs/2d-2f.md)** — Add module declaration, verify existing tests GREEN, commit atomically.
