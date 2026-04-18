# Deferred items from 13-C execution

## 1. `contract_github` test requires `REPOSIX_ALLOWED_ORIGINS`

**File:** `crates/reposix-github/tests/contract.rs::contract_github`
**Kind:** Test hygiene / env-gating inconsistency
**Impact:** `cargo test --workspace --release -- --ignored --test-threads=1`
fails on a fresh clone unless `REPOSIX_ALLOWED_ORIGINS` includes
`api.github.com`.

The test is `#[ignore]`-gated but when unlocked via `--ignored`, it
hard-asserts env presence rather than skipping. Its Confluence sibling
(`contract_confluence_live`) uses the `skip_if_no_env!` macro to skip
cleanly when env is unset; `contract_github` predates that macro and
panics instead.

**Fix (future plan):** Apply the `skip_if_no_env!` pattern to
`contract_github`. After the change:
- `--ignored` without env → SKIP (test returns cleanly).
- `--ignored` with env set but wrong value → still panics (correct —
  invoker explicitly expects a live run).

**Scope:** Out of scope for Wave C. `crates/reposix-github/tests/` is
not in 13-C's file list and the failure is pre-existing behavior, not
caused by any 13-C FUSE wiring change. Verify by checking out `main`
before `13-C-1` and running the same command — the failure is
identical.

## 2. `reposix-remote` still emits `{:04}.md` paths

**Files:**
- `crates/reposix-remote/src/diff.rs` (2 sites)
- `crates/reposix-remote/src/fast_import.rs` (1 site + a comment)
- `crates/reposix-remote/tests/protocol.rs` (1 test fixture)

**Impact:** The git-remote-reposix helper produces a fast-import stream
where issues live at `<id-4-digit>.md` at the git tree root. The FUSE
mount (post-13-C) surfaces them at `<bucket>/<id-11-digit>.md`. The two
padding schemes now disagree.

**Scope:** Out of scope for Wave C (file list only covers
`crates/reposix-fuse/*`). The `reposix-remote` crate is a separate
integrator that Wave D1 (`13-D1-breaking-migration-sweep`) is planned
to handle as part of the BREAKING-change unification pass. Wave C
lands only the FUSE-side numeric truth.

**Verify before fix:** Once D1 lands its sweep, the reposix-remote
tests should still pass because they assert the local git-tree shape,
which is independent of the FUSE mount-time padding. The user-visible
git-tree path should become `<bucket>/<11-digit>.md` to match the FUSE
mount, so `git diff` between two clones of the same mount yields
identical output.
