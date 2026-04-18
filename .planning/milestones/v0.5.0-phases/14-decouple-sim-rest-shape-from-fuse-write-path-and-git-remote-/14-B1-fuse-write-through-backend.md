---
phase: 14
wave: B1
slug: fuse-write-through-backend
serial: false             # parallel with B2
depends_on_waves: [A]
blocks_waves: [C]
parallel_with: [B2]
estimated_wall_clock: 45m
executor_role: gsd-executor
---

# Wave B1 — reposix-fuse write path through `IssueBackend`

> Read `14-CONTEXT.md` and `14-RESEARCH.md` (whole file) before executing.
> This wave runs **in parallel** with Wave B2 (reposix-remote refactor). The filesets
> are disjoint: B1 touches `crates/reposix-fuse/**` and `crates/reposix-core/src/backend/sim.rs`
> (tests only); B2 touches `crates/reposix-remote/**`. Shared file: `Cargo.lock`
> (see R11 below).

## Scope in one sentence

Route `ReposixFs::release` and `ReposixFs::create` through `IssueBackend::update_issue`
and `IssueBackend::create_issue` via two new timeout wrappers in `fs.rs`; delete
`crates/reposix-fuse/src/fetch.rs` and `crates/reposix-fuse/tests/write.rs`; re-home
the load-bearing write-path tests onto `SimBackend` in `sim.rs`.

## What NOT to touch (locked)

- Do **not** rename `FetchError`. Keep the enum name to minimize diff (R12 in `14-PLAN.md`).
  A "FsError" rename is reserved for a future cleanup phase.
- Do **not** modify `crates/reposix-core/src/backend/sim.rs` except to **add** the test
  re-homes enumerated below. Do not touch `SimBackend::render_patch_body`,
  `render_create_body`, `json_headers`, `agent_only`, or any trait method body (LD-14-01,
  LD-14-06, R1 resolution).
- Do **not** add a `reposix_core::Error::VersionMismatch` variant (LD-14-04, R5).
- Do **not** push the outer `tokio::time::timeout` into `SimBackend` (RESEARCH Q3).
  The wrapper stays in `fs.rs`.
- Do **not** attempt to preserve the old `X-Reposix-Agent` header value verbatim. Use
  `SimBackend::with_agent_suffix(origin, Some("fuse"))` (RESEARCH Q4 option (a); R2
  accepted). The audit attribution change is a documented v0.4.1 `### Changed` item.
- Do **not** try to preserve the old PATCH body's skip-if-none behavior for `assignee`.
  Option (A) from RESEARCH Q9 is accepted: the sim-backend's explicit `null` emission on
  None is the honest interpretation (R1 accepted, documented in CHANGELOG by Wave D).

## Files to touch

- `crates/reposix-fuse/src/fs.rs` — write callbacks, error mapping, two new timeout
  wrappers.
- `crates/reposix-fuse/src/lib.rs` — drop `pub mod fetch;` (expected line 23) and update
  the module doc if it mentions the old "still speaks the sim REST shape" prose (RESEARCH
  Q8 notes the comment near lines 9-10 and 38-43 of `lib.rs`).
- `crates/reposix-fuse/src/main.rs` — the FUSE binary constructs the backend; change
  to `SimBackend::with_agent_suffix(origin, Some("fuse"))`. Drop any `agent: String`
  plumbing that previously flowed through.
- `crates/reposix-fuse/Cargo.toml` — likely no change; `reposix-core` stays a dep,
  `tokio` stays. If `serde_json` or any other dep was pulled in solely for `fetch.rs`,
  audit and drop (unlikely — most of those were already transitive through
  `reposix-core`).
- `crates/reposix-core/src/backend/sim.rs` — **tests only**. Add the re-homed
  assertions per RESEARCH Q10 into the existing `#[cfg(test)] mod tests` block. Do not
  touch any production function in this file.

## Files to delete (atomically, in this wave's single commit)

- `crates/reposix-fuse/src/fetch.rs` — entire file (596 lines, RESEARCH Q8).
- `crates/reposix-fuse/tests/write.rs` — entire file (the only external importer of
  `reposix_fuse::fetch`).

## Tasks

### Task B1.1 — Fold surviving `fetch.rs` types into `fs.rs`

The `release` callback's diagnostic arm (currently at `fs.rs:1052-1058`) needs
`FetchError::Conflict { current }` to emit the `current=N` warn log. Move into `fs.rs`
(above the `ReposixFs` impl block, near the existing `backend_err_to_fetch` at
`fs.rs:108-116`):

1. A private `ConflictBody` struct (`serde::Deserialize`), mirroring today's
   `fetch.rs:103-106` with fields `current: u64` (and ignore all other fields — use
   `#[serde(deny_unknown_fields)]` sparingly or not at all; the sim's body also has
   `error` and `sent`).
2. A private `FetchError` enum with variants `{NotFound, Origin(String), Timeout,
   Transport(String), Status(u16), Parse(String), Core(String), Conflict { current: u64 }}`.
   **Drop `BadRequest(String)`** — no caller produces it post-refactor (RESEARCH Q2, R4).
3. Keep `fs.rs::fetch_errno` (~lines 534-540) as the error→`libc` int mapper. After
   dropping `BadRequest`, the arm listing in `fetch_errno` must also be adjusted — remove
   the `FetchError::BadRequest(_) => libc::EIO` row. `Core`, `Timeout`, `Transport`,
   `Status`, `Parse`, and `Conflict { .. }` continue to map to `EIO`; `NotFound` continues
   to `ENOENT`; `Origin(_)` continues to `EACCES` (or whatever its current mapping is —
   preserve whatever today's `fetch_errno` returns).

### Task B1.2 — Extend `backend_err_to_fetch` to recover `current`

The current function at `fs.rs:108-116` has these arms:

```rust
InvalidOrigin(o) => FetchError::Origin(o),
Http(t)          => FetchError::Transport(t),
Json(j)          => FetchError::Parse(j),
Other(msg) if msg.starts_with("not found") => FetchError::NotFound,
other            => FetchError::Core(other.to_string()),
```

Add a new arm **before** the catch-all, matching the `"version mismatch: "` prefix:

- Strip the prefix → yields the JSON body as a `&str`.
- `serde_json::from_str::<ConflictBody>(json_str)` → on `Ok`, return `FetchError::Conflict { current: body.current }`.
- On `Err` (sim ever changes its body format), fall back to
  `FetchError::Conflict { current: 0 }` — mirrors the existing unwrap-or behavior at
  `fetch.rs:246`.

This preserves SC-14-09: version-mismatch still hits the `Conflict`-diagnostic arm in
`release`, logs `current=N`, and replies `EIO`.

### Task B1.3 — Add `update_issue_with_timeout` and `create_issue_with_timeout`

Pattern after the existing `list_issues_with_timeout` / `get_issue_with_timeout` in
`fs.rs:117-140` (RESEARCH Q3 recommendation; shape sketched verbatim there).

Both wrappers:

- Take `backend: &Arc<dyn IssueBackend>` (or `&Arc<SimBackend>` if that's simpler; the
  read-path uses `dyn` so stick with that for consistency).
- Wrap the trait call in `tokio::time::timeout(READ_GET_TIMEOUT, ...)` — the same
  `READ_GET_TIMEOUT` constant already in scope at `fs.rs:103`.
- On timeout, return `Err(FetchError::Timeout)`.
- On trait-call `Err`, return `backend_err_to_fetch(e)` — **critical**: this is what
  restores the `FetchError::Conflict { current: N }` surface from Task B1.2.
- On success, return the `Issue`.

Signature templates (RESEARCH Q3 gives the full shape):

```rust
async fn update_issue_with_timeout(
    backend: &Arc<dyn IssueBackend>,
    project: &str,
    id: IssueId,
    patch: Untainted<Issue>,
    expected_version: Option<u64>,
) -> Result<Issue, FetchError> { /* as RESEARCH Q3 */ }

async fn create_issue_with_timeout(
    backend: &Arc<dyn IssueBackend>,
    project: &str,
    issue: Untainted<Issue>,
) -> Result<Issue, FetchError> { /* same pattern */ }
```

### Task B1.4 — Rewrite `ReposixFs::release`

Today (approx `fs.rs:1029` per CONTEXT, `fs.rs:1052-1058` for the diagnostic arm):
`release` calls `fetch::patch_issue(&self.http, &self.origin, &self.project, id, &egress, version, &self.agent).await`.

After refactor:

- Call `update_issue_with_timeout(&self.backend, &self.project, id, sanitize(...), Some(version)).await`
  instead.
- Preserve the existing `match` on `Result<Issue, FetchError>`:
  - `Ok(updated)` → same downstream handling as today (cache update, reply, etc.).
  - `Err(FetchError::Conflict { current })` → **same existing warn + `reply.error(libc::EIO)`**
    at `fs.rs:1052-1058`. Unchanged.
  - All other `Err(...)` → same today's handling (maps through `fetch_errno` or the
    existing match arms).

**Crucial:** do NOT introduce the agent string into the arguments; the trait call carries
it via `self.backend`'s internal `agent_header` field (set up in `main.rs`).

### Task B1.5 — Rewrite `ReposixFs::create`

Today (approx `fs.rs:1110` per CONTEXT): `create` calls `fetch::post_issue(...)`.

After refactor:

- Call `create_issue_with_timeout(&self.backend, &self.project, sanitize(...)).await`.
- Preserve the existing error handling — there's no `FetchError::Conflict` surface on
  create (POST doesn't return 409 in the sim). Success returns the `Issue` with its
  server-assigned `id` / `version` / `created_at`; the FUSE cache updates as today.

### Task B1.6 — Drop the `agent: String` field on `ReposixFs` (RESEARCH Q4)

`fs.rs:272-326` currently builds `agent = format!("reposix-fuse-{}", std::process::id())`
and stores it on `ReposixFs`. After refactor the backend carries the attribution
internally; `self.agent` is unused. Grep for `self.agent` usage in `fs.rs`; if nothing
outside the deleted `fetch::` calls referenced it, drop the field.

Move the `SimBackend::with_agent_suffix(origin, Some("fuse"))` construction into
`crates/reposix-fuse/src/main.rs` (the binary entrypoint) — this is where the
`Arc<dyn IssueBackend>` is built and handed to `ReposixFs::new`. Drop the `agent: String`
constructor parameter from `ReposixFs::new` if present.

### Task B1.7 — Drop `pub mod fetch;` + the old import

- `crates/reposix-fuse/src/lib.rs:23` currently has `pub mod fetch;` — remove this line.
- `crates/reposix-fuse/src/lib.rs:9-10` and `38-43` (RESEARCH Q8, Q10 note) may contain
  module-doc prose mentioning "the write path still speaks the sim REST shape via fetch"
  or similar. Delete those sentences; replace with one-liner (if a replacement is natural)
  saying "writes flow through `IssueBackend::{create,update,delete_or_close}_issue`." No
  heavy rewrite — the full architecture prose lives in `docs/architecture.md` (Wave D).
- `fs.rs:89` currently has `use crate::fetch::{patch_issue, post_issue, FetchError};`.
  Remove. Replace with `use reposix_core::backend::{IssueBackend};` and
  `use reposix_core::{Issue, IssueId, Untainted};` as needed. `FetchError` is now local to
  `fs.rs` (defined in B1.1).

### Task B1.8 — Delete `fetch.rs`

```bash
git rm crates/reposix-fuse/src/fetch.rs
```

Nothing else should import it after Tasks B1.6 and B1.7. `cargo check -p reposix-fuse`
confirms.

### Task B1.9 — Delete `tests/write.rs`

```bash
git rm crates/reposix-fuse/tests/write.rs
```

The re-home of its critical assertions (SG-03 sanitizer proof, etc.) happens in
Task B1.11. See RESEARCH Q10.

### Task B1.10 — Re-home write-path tests to `sim.rs`

Open `crates/reposix-core/src/backend/sim.rs`, navigate to the existing
`#[cfg(test)] mod tests` block (approx lines 313-514 per RESEARCH Q10), and add the
following tests. **Model them on the existing `update_with_expected_version_attaches_if_match`
test at lines 420-462** for shape + wiremock setup.

From `fetch.rs::tests` (to re-home):

1. **`patch_issue_sends_if_match_header` → rename `update_issue_sends_quoted_if_match`.**
   Assert `If-Match: "3"` quoted (not the old unquoted form — R3 in `14-PLAN.md`).
   Call `backend.update_issue("demo", IssueId(1), sanitize_fixture(), Some(3)).await`.
2. **`patch_issue_409_returns_conflict` → rename `update_issue_409_surfaces_version_mismatch`.**
   Assert `Err(Error::Other(msg))` with `msg.starts_with("version mismatch:")` and
   `msg.contains("\"current\":7")`.
3. **`post_issue_sends_egress_shape_only` → rename `create_issue_omits_server_fields`.**
   Assert POST body JSON has `title` and `labels`, and explicitly lacks `version`, `id`,
   `created_at`, `updated_at`.
4. **`fetch_issue_attaches_agent_header` → rename `update_issue_attaches_agent_header`.**
   Use a wiremock `header_exists("X-Reposix-Agent")` matcher (RESEARCH Q4 note about the
   value being process-specific). If the scaffold doesn't have `header_exists`, mimic
   the `update_without_expected_version_is_wildcard` closure matcher pattern at
   `sim.rs:471-477`.

From `tests/write.rs` (to re-home):

5. **`sanitize_strips_server_fields_on_egress` → add as `update_issue_respects_untainted_sanitization`.**
   Construct a deliberately hostile `Issue` (with `version`, `id`, `created_at`,
   `updated_at` set to non-default values); wrap via the standard `sanitize(Tainted::new(...))`
   call; pass into `backend.update_issue`; assert wire body lacks those four keys. This
   is the **SG-03 proof** (RESEARCH Q10 flags it as critical — re-home, do not delete).

From RESEARCH Q2 recommendation:

6. **`create_issue_400_preserves_body_in_error`**. Wiremock 400 + body `"invalid title"`
   on POST; `backend.create_issue(...)` returns `Err(Error::Other(msg))` where
   `msg.contains("invalid title")`.

From RESEARCH Q5 / R6:

7. **`sim_backend_rejects_non_allowlisted_origin`**. Construct
   `SimBackend::new("http://evil.example".into())`; call `list_issues` (or any trait
   method); assert `Err(Error::InvalidOrigin(_))`. This preserves the SG-01 coverage
   formerly in `fetch_issue_origin_rejected`. **Important:** the allowlist gate lives in
   `http.rs`; this test must set `REPOSIX_ALLOWED_ORIGINS=http://127.0.0.1:*` (or default)
   at run-time if the test binary doesn't already. Follow the pattern the existing
   `sim.rs` tests use — check if `REPOSIX_ALLOWED_ORIGINS` is read at `client()` time or
   per-request; adjust test harness accordingly.

Read-path re-homes (RESEARCH Q10 flags some as already covered — execute accordingly):

- `fetch_issues_parses_list`, `fetch_issue_parses_one`, `fetch_issue_404_is_not_found` —
  **delete from `fetch.rs` without re-home**; `sim.rs` already covers (sim.rs:363-417
  per RESEARCH).
- `fetch_issue_500_is_status` — **re-home as `get_issue_500_surfaces_error_other`**;
  assert `Err(Error::Other(msg))` with `msg.contains("sim returned 500")`. RESEARCH Q10
  notes no direct sim equivalent exists.
- `fetch_issues_attaches_agent_header` — covered by re-homed test #4 above (one test
  proves the attribution for the whole backend; adding a second for list_issues is
  redundant).
- `fetch_issue_times_out_within_budget` — re-home to `fs.rs::get_issue_with_timeout`
  test (RESEARCH notes this may already be covered — check `fs.rs` test block; if
  yes, delete without re-home).

### Task B1.11 — Add `fs.rs` tests for the new helpers

Add to `#[cfg(test)] mod tests` in `fs.rs`:

1. **`backend_err_to_fetch_maps_version_mismatch_with_current`**. Feed
   `reposix_core::Error::Other("version mismatch: {\"error\":\"version_mismatch\",\"current\":7,\"sent\":\"1\"}".into())`;
   assert returned `FetchError::Conflict { current: 7 }`. Proves Task B1.2's JSON
   parse.
2. **`backend_err_to_fetch_maps_malformed_version_mismatch_to_current_zero`**. Feed
   `Error::Other("version mismatch: garbage".into())`; assert returned
   `FetchError::Conflict { current: 0 }`. Proves the graceful-degradation fallback.
3. **`update_issue_with_timeout_times_out_within_budget`**. Stand up a wiremock that
   delays its 200 response by ~6s; call `update_issue_with_timeout(...)`; assert
   `Err(FetchError::Timeout)` within ~5.5s wall-clock. This is the re-home of the old
   `patch_issue_times_out_within_budget` test.
4. **(optional)** `create_issue_with_timeout_times_out_within_budget`. Symmetric to
   (3) for POST. Skip if test count already ≥ 272 and wall-clock budget is tight;
   otherwise add for belt-and-suspenders.

### Task B1.12 — Compile + test

Run in this order:

```bash
cargo check -p reposix-fuse --locked
cargo check -p reposix-core --locked
cargo check --workspace --locked          # catches any transitive breakage
cargo test -p reposix-core --locked       # sim.rs re-homed tests
cargo test -p reposix-fuse --locked       # fs.rs tests + any surviving unit tests
cargo test --workspace --locked           # full baseline
cargo clippy -p reposix-fuse --all-targets --locked -- -D warnings
cargo clippy -p reposix-core --all-targets --locked -- -D warnings
```

Green on all. If the full workspace test count dips below 272 (LD-14-08), investigate —
the RESEARCH Q10 analysis projected -15 FUSE + +5-7 core, netting somewhere around
262-269. Add the optional B1.11 task (4) and/or a write-path `fs.rs` test if needed to
close the gap. If genuine test obsolescence (e.g. `fetch_issues_parses_list` was
redundant with `sim.rs` coverage) accounts for the dip, document in the commit message.

## Commit

One atomic commit:

```
refactor(14-B1): route fs.rs write path through IssueBackend trait

- fs.rs::{release, create} now call
  update_issue_with_timeout / create_issue_with_timeout, which wrap
  self.backend.{update,create}_issue in the same 5s tokio::time::timeout
  pattern the read path already uses.
- backend_err_to_fetch grows an arm for Error::Other("version mismatch: ...")
  that JSON-parses the tail to recover the 409 `current` value into
  FetchError::Conflict { current }, preserving the existing release
  diagnostic log. Malformed bodies degrade to current: 0 (matches
  fetch.rs:246's old unwrap-or).
- FetchError::BadRequest dropped; no caller produces it post-refactor.
- FetchError + ConflictBody moved from fetch.rs into fs.rs (private).
- reposix-fuse/src/fetch.rs deleted (entire file).
- reposix-fuse/tests/write.rs deleted; load-bearing assertions (SG-03
  sanitizer proof, If-Match header, 409 surface, agent attribution,
  egress shape) re-homed into crates/reposix-core/src/backend/sim.rs
  tests.
- X-Reposix-Agent now emitted as reposix-core-simbackend-<pid>-fuse
  (was reposix-fuse-<pid>). Documented in CHANGELOG [Unreleased] ###
  Changed (Wave D).
- PATCH body now emits explicit "assignee": null when frontmatter
  lacks the field (sim's three-value Clear semantic). This is a
  behavior change from the old skip-if-none; the FUSE mount's
  "file is source of truth" design makes the new semantic honest.
  Documented in CHANGELOG ### Changed (Wave D).

Closes HANDOFF "Known open gaps" item 7.
See .planning/phases/14-.../14-CONTEXT.md and 14-RESEARCH.md.
```

## Tests to pass before commit

- `cargo test --workspace --locked` — green, count ≥ 272 (or document the shortfall).
- `cargo clippy --workspace --all-targets --locked -- -D warnings` — clean.
- Grep proofs (run before committing):
  - `git grep -n 'patch_issue\|post_issue\|EgressPayload' crates/reposix-fuse/src/` → zero hits.
  - `git grep -n 'pub mod fetch' crates/reposix-fuse/src/lib.rs` → zero hits.
  - `git grep -n 'use crate::fetch' crates/reposix-fuse/src/` → zero hits.
  - `test ! -e crates/reposix-fuse/src/fetch.rs` → exits 0.
  - `test ! -e crates/reposix-fuse/tests/write.rs` → exits 0.

## Acceptance criteria

- [ ] `fs.rs::release` calls `update_issue_with_timeout` (via `backend.update_issue`).
- [ ] `fs.rs::create` calls `create_issue_with_timeout` (via `backend.create_issue`).
- [ ] `backend_err_to_fetch` has a `"version mismatch:"` arm.
- [ ] `fetch.rs` deleted. `tests/write.rs` deleted. `pub mod fetch;` removed from `lib.rs`.
- [ ] Version-mismatch test path (sim.rs + fs.rs) proves 409 → `FetchError::Conflict { current: N }`.
- [ ] Sanitizer test (`update_issue_respects_untainted_sanitization`) in `sim.rs` is green.
- [ ] Workspace test count ≥ 272 OR documented shortfall.
- [ ] Clippy green. Workspace green.
- [ ] Cargo.lock included in the commit if Cargo machinery regenerated it.

## Non-scope (reserved for other waves)

- Anything under `crates/reposix-remote/` — B2 owns.
- Running the green-gauntlet or live write-demo — C owns.
- CHANGELOG entry for R1 and R2 — D owns. B1 only references these in the commit body.
- Touching `docs/architecture.md` diagram lines 112, 156 — D owns.

## Parallel-safety notes vs. B2

- B1 edits `Cargo.lock` iff any dep version migrates (unlikely — neither wave adds new
  deps). If B1 and B2 both regenerate the lockfile with **identical** resolver output
  (expected, since neither wave changes direct deps), rebase is trivial. If B2 merged
  first, B1 rebases on `main`, re-runs `cargo check`, and includes the rebased lockfile
  in its final commit. `cargo` is deterministic here; no manual conflict resolution
  should be necessary.
- No source files overlap. B1 touches `crates/reposix-fuse/src/**`, `crates/reposix-fuse/tests/**`,
  and **only the test block of** `crates/reposix-core/src/backend/sim.rs`. B2 touches
  `crates/reposix-remote/**` exclusively.

## References

- `14-CONTEXT.md` SC-14-01, SC-14-02, SC-14-03, SC-14-06, SC-14-09.
- `14-RESEARCH.md#Q1` — version-mismatch error mapping (Task B1.2).
- `14-RESEARCH.md#Q3` — timeout wrappers (Task B1.3).
- `14-RESEARCH.md#Q4` — agent attribution (Task B1.6).
- `14-RESEARCH.md#Q8` — dead-code inventory (Tasks B1.7-B1.9).
- `14-RESEARCH.md#Q10` — test re-homing map (Task B1.10).
- `14-PLAN.md` risk log — R1, R2, R3, R4, R6, R11, R12, R13.
