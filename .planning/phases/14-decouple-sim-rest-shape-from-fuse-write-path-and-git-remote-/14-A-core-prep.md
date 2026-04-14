---
phase: 14
wave: A
slug: core-prep
serial: true
depends_on_waves: []
blocks_waves:
  - B1
  - B2
estimated_wall_clock: 15m
executor_role: gsd-executor
---

# Wave A — core-crate prep

> Read `14-CONTEXT.md` and `14-RESEARCH.md` (whole file) before executing. The
> decisions locked by the orchestrator are in `14-PLAN.md` under the risk log; do not
> re-litigate R1/R2/R5 inside this wave.

## Goal

Confirm `reposix-core` needs **no** trait-surface changes for Phase 14. If any helper,
`pub` re-export, or module path is missing for B1 or B2 to compile, add exactly that
— no more. The default outcome is that Wave A closes without a code commit and only
records a "core is ready" note on the tracking branch.

## What NOT to touch (locked by RESEARCH + orchestrator)

- Do **not** modify `crates/reposix-core/src/backend.rs` — the `IssueBackend` trait,
  `BackendFeature`, and `DeleteReason` are frozen (LD-14-01).
- Do **not** modify `crates/reposix-core/src/backend/sim.rs` except to **add** the
  assertion-pin tests enumerated in the "Tasks" section below. `render_patch_body`,
  `render_create_body`, `json_headers`, `agent_only`, and all four trait methods are
  frozen (LD-14-01 + LD-14-06 + R1 resolution).
- Do **not** add a new `reposix_core::Error::VersionMismatch` variant (LD-14-04; R5
  resolution). Version-mismatch stays a string-matched `Error::Other("version mismatch: ...")`.
- Do **not** touch `crates/reposix-core/src/remote.rs` or `parse_remote_url` (LD-14-02).
- Do **not** touch `crates/reposix-core/src/http.rs` — the allowlist + `ClientOpts`
  behavior is frozen (LD-14-06).

## Files to touch (most likely: **none**)

**Probable outcome:** zero code change. The wave's single commit (if any) is a
status-only `docs(14-A):` marker.

**Contingent outcome (only if B1 or B2 genuinely needs a `pub` visibility change):**
- `crates/reposix-core/src/lib.rs` — export a helper if B1 or B2 would otherwise have to
  reach into a private module. As of RESEARCH's full read, B1 uses only
  `reposix_core::{Error, backend::{IssueBackend, SimBackend, DeleteReason}, Untainted, Issue, IssueId}`,
  all of which are already public. B2 uses the same set. So this is a safety net, not
  an expected change.

## Files to delete

None in Wave A.

## Tasks

### Task A.1 — Read-verification sanity pass (**read-only**, ~5 min)

Grep the current public surface of `reposix-core` to confirm B1/B2 have what they need:

```bash
# From repo root.
cargo doc -p reposix-core --no-deps --document-private-items 2>&1 | head -40
# Or faster: rg on the public items.

git grep -n '^pub ' crates/reposix-core/src/backend.rs
git grep -n '^pub ' crates/reposix-core/src/backend/sim.rs
git grep -n '^pub use\|^pub mod' crates/reposix-core/src/lib.rs
```

Confirm these are all `pub`:

- `reposix_core::Error` (all variants: `InvalidOrigin`, `Http`, `Json`, `Other`).
- `reposix_core::Issue`, `reposix_core::IssueId`, `reposix_core::Untainted<Issue>`.
- `reposix_core::backend::IssueBackend` (trait).
- `reposix_core::backend::DeleteReason` (all four variants).
- `reposix_core::backend::sim::SimBackend` + `SimBackend::new` + `SimBackend::with_agent_suffix`.

If any are private, make them `pub` in this wave. If all are public (expected), record
"Wave A verified reposix-core public surface is sufficient; no code change needed" and
move to Task A.2.

### Task A.2 — Add assertion-pin tests to `sim.rs` (R13 mitigation)

The 409 conflict body shape and the `"version mismatch: "` prefix in `Error::Other` are
load-bearing contracts B1's `backend_err_to_fetch` will pattern-match on. If the sim
ever changes either, we want a **core-crate** test to fail loudly rather than a silent
FUSE log degradation.

Add exactly **two** tests to the `#[cfg(test)] mod tests` block at the bottom of
`crates/reposix-core/src/backend/sim.rs`:

1. **`update_409_error_starts_with_version_mismatch_prefix`** — set up a wiremock that
   responds 409 with body `{"error":"version_mismatch","current":7,"sent":"1"}`; call
   `backend.update_issue("demo", IssueId(1), sanitize_fixture(), Some(1)).await`;
   assert the error is `Err(Error::Other(msg))` where `msg.starts_with("version mismatch: ")`.
2. **`update_409_body_contains_current_field`** — same setup; assert the tail of
   `msg` after the prefix parses as JSON with a top-level `"current"` key whose value
   is `7` (a positive u64).

These tests pin the contract `backend_err_to_fetch` will string-match and JSON-parse in
Wave B1. If the sim ever refactors the error body, these fire before B1's regression.

**Why add in Wave A and not B1:** The contract lives in `reposix-core`; the assertion
belongs in that crate's test suite. Also, if the sim doesn't emit what we think, B1's
plan is wrong — better to learn it now.

**Why not also a timeout test here:** Timeout behavior is a FUSE-callback concern
(outer-wrapper-in-fs.rs). B1 adds that test against the new `update_issue_with_timeout`
helper.

### Task A.3 — Commit (only if Task A.1 or A.2 produced a change)

**If Task A.1 yielded a `pub` change:**

```
feat(14-A): expose <symbol> from reposix-core for B1/B2

Required by Phase 14 trait rewire. See 14-PLAN.md.
```

**If Task A.2 added the pin tests:**

```
test(14-A): pin 409-body contract used by backend_err_to_fetch in fs.rs

Adds two wiremock-backed tests asserting SimBackend::update_issue surfaces
409 conflicts as Error::Other("version mismatch: {json-body}") with a
parseable "current" field. Wave B1's backend_err_to_fetch will string-match
+ JSON-parse this; if sim's error body ever changes, this fires loudly.

See 14-RESEARCH.md#Q1, 14-PLAN.md risk R13.
```

**If both Task A.1 and Task A.2 produced changes:** single commit bundling both, message
above combined.

**If neither produced a change:** no commit. Leave a note in the phase-tracking scratch
pad (or the executor's return message) that "Wave A verified: no core-crate change
required; moving to B1 + B2." This is the expected outcome.

## Tests to pass before commit

If Task A.2 added pin tests:

```bash
cargo test -p reposix-core --locked
```

Expected: all existing tests still pass, two new pin tests pass. Test count on
reposix-core rises by exactly 2.

Also run:

```bash
cargo check --workspace --locked
cargo clippy -p reposix-core --all-targets --locked -- -D warnings
```

Both green.

## Acceptance criteria

Wave A is done when:

- [ ] `reposix-core`'s public surface confirmed sufficient for B1 + B2 (or extended,
      with a commit explaining what and why).
- [ ] If A.2 was added: two pin tests green; `cargo test -p reposix-core --locked`
      passes.
- [ ] `cargo check --workspace --locked` green.
- [ ] Executor's return message explicitly states: "Wave A gate passed. B1 and B2 may
      start in parallel."

## Non-scope (explicitly reserved for later waves)

- **B1's territory:** anything under `crates/reposix-fuse/`. Do NOT preemptively create
  `update_issue_with_timeout` helpers here.
- **B2's territory:** anything under `crates/reposix-remote/`. Do NOT touch
  `main.rs::execute_action`.
- **C's territory:** green-gauntlet, smoke, live demo.
- **D's territory:** CHANGELOG, docs.

## References

- `14-CONTEXT.md` — locked decisions LD-14-01..08.
- `14-RESEARCH.md#Q1` — 409 body shape; `Error::Other("version mismatch: ...")` evidence.
- `14-RESEARCH.md#Q7` — `DeleteReason::Abandoned` already exists; no new variant needed.
- `14-PLAN.md` risk R13 — why A.2 exists.
- `crates/reposix-core/src/backend/sim.rs:272-280` — the 409 arm this test pins.
- `crates/reposix-sim/src/error.rs:83-91` — the sim's JSON body shape.
