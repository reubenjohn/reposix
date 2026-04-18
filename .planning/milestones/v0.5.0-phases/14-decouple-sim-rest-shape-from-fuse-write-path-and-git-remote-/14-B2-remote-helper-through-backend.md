---
phase: 14
wave: B2
slug: remote-helper-through-backend
serial: false             # parallel with B1
depends_on_waves: [A]
blocks_waves: [C]
parallel_with: [B1]
estimated_wall_clock: 45m
executor_role: gsd-executor
---

# Wave B2 — `git-remote-reposix` through `IssueBackend`

> Read `14-CONTEXT.md` and `14-RESEARCH.md` (whole file) before executing.
> This wave runs **in parallel** with Wave B1 (reposix-fuse refactor). Disjoint
> filesets — B2 touches only `crates/reposix-remote/**`. Shared file: `Cargo.lock`
> (see parallel-safety section below).

## Scope in one sentence

Rewrite `crates/reposix-remote/src/main.rs` so `execute_action` and the two
`api::list_issues` call sites dispatch to `IssueBackend::{list_issues, create_issue,
update_issue, delete_or_close}`, delete `crates/reposix-remote/src/client.rs` entirely,
and prune the now-unused `thiserror` / `reqwest` deps from `Cargo.toml`.

## What NOT to touch (locked)

- Do **not** widen the remote-helper URL syntax. The `reposix::http://host:port/projects/slug`
  form is frozen (LD-14-02 + session-5 non-goals). No `--backend` flag; no scheme-based
  dispatch.
- Do **not** touch `crates/reposix-core/src/remote.rs` or `parse_remote_url`.
- Do **not** invent a new `"version-mismatch"` wire kind on the git-remote's
  `error refs/heads/...` line. The existing `"some-actions-failed"` string is the correct
  wire kind (R5 in `14-PLAN.md`). SC-14-09's prose mentioning `version-mismatch` is
  aspirational; Wave D clarifies the docs.
- Do **not** preserve the old `X-Reposix-Agent: git-remote-reposix-<pid>` header value.
  Use `SimBackend::with_agent_suffix(origin, Some("remote"))` (RESEARCH Q4 option (a);
  R2 accepted).
- Do **not** touch `crates/reposix-remote/src/diff.rs`, `fast_import.rs`, or
  `protocol.rs` beyond cosmetic compile-fix tweaks if a type signature changes. `plan`
  logic is order-independent (RESEARCH Q6).
- Do **not** add a new `trait Error` layer. `anyhow::Error` wrapping `reposix_core::Error`
  is the surface (LD-14-04).

## Files to touch

- `crates/reposix-remote/src/main.rs` — the big rewrite. Construct `Arc<SimBackend>` once,
  store on `State`, dispatch `execute_action` and the two list sites through the trait.
- `crates/reposix-remote/Cargo.toml` — drop `thiserror` dep (line ~24 per RESEARCH Q8).
  Audit + drop `reqwest` dep (line ~16) iff `main.rs` no longer imports anything from it
  directly post-refactor. Keep `serde`, `serde_json`, `serde_yaml`, `anyhow`, `tokio`,
  `clap`, `tracing`, `tracing-subscriber`.
- `crates/reposix-remote/src/diff.rs` — **read-only** unless a type signature change
  from `main.rs` forces a compile fix. RESEARCH Q6 says `plan` is order-independent and
  needs no behavior change.
- `crates/reposix-remote/src/fast_import.rs` — **read-only** unless a type signature
  change forces a compile fix.
- `crates/reposix-remote/src/protocol.rs` — **read-only**.

## Files to delete (atomically, in this wave's single commit)

- `crates/reposix-remote/src/client.rs` — entire file (236 lines, RESEARCH Q8).

## Tasks

### Task B2.1 — Audit existing `main.rs` structure

Read the current `State` struct, its construction site, and the three dispatch sites.
RESEARCH pinpoints:

- `main.rs:82` — current agent: `let agent = format!("git-remote-reposix-{}", std::process::id());`
- `main.rs:181-196` — `api::list_issues` in the import path.
- `main.rs:228-244` — `api::list_issues` in the export-prior-state path.
- `main.rs:296-358` — `execute_action` with `api::{post_issue, patch_issue, delete_issue}`.

Grep for every reference to `client`, `api::`, `ClientError`, and the `http` / `agent`
fields on `State`:

```bash
git grep -n 'client\|api::\|ClientError' crates/reposix-remote/src/main.rs
git grep -n 'state\.http\|state\.agent\|state\.origin' crates/reposix-remote/src/
```

Enumerate every call site. These are what Task B2.3 rewrites.

### Task B2.2 — Update `State` to carry `Arc<SimBackend>`

Today's `State` carries (per inspection at the top of `main.rs`, ~lines 60-100):

- `rt: tokio::runtime::Runtime`
- `http: HttpClient` (built via `reposix_core::http::client(ClientOpts::default())?`)
- `origin: String`
- `project: String`
- `agent: String`

After refactor:

- `rt: tokio::runtime::Runtime` — keep.
- `backend: Arc<SimBackend>` (or `Arc<dyn IssueBackend>`; RESEARCH Q6 notes either
  works because dyn-compat is proven — pick `Arc<SimBackend>` if only sim-capable
  methods are called, otherwise `Arc<dyn IssueBackend>` for consistency with `fs.rs`.
  Recommend `Arc<dyn IssueBackend>` to mirror the FUSE side — same mental model).
- `project: String` — keep.
- Drop `http`, `origin`, `agent` — the backend holds them internally.

Construction (replacing today's ~line 82 agent + http setup):

```rust
let backend: Arc<dyn IssueBackend> = Arc::new(
    SimBackend::with_agent_suffix(spec.origin, Some("remote"))
);
```

Where `spec: RemoteSpec` is the parse output of `parse_remote_url` (unchanged).

### Task B2.3 — Rewrite the three dispatch sites

All three sites live inside `state.rt.block_on(...)` because the helper is a sync
binary embedding a tokio runtime.

**Two list-sites (RESEARCH Q6):**

Old:

```rust
state.rt.block_on(api::list_issues(&state.http, &state.origin, &state.project, &state.agent))
```

New:

```rust
state.rt.block_on(state.backend.list_issues(&state.project))
```

Two call sites. Keep the surrounding `.context("cannot list prior issues")` or
equivalent `anyhow` wrap — RESEARCH Q6 confirms the existing `protocol.rs` test
asserting stderr `"cannot list prior issues"` OR `"backend"` still holds.

**`execute_action` branches (RESEARCH Q6, Q7):**

Today (`main.rs:296-358`):

```rust
PlannedAction::Create { issue, .. }    => api::post_issue(...),
PlannedAction::Update { id, patch, .. } => api::patch_issue(...),
PlannedAction::Delete { id, .. }        => api::delete_issue(...),
```

New:

```rust
PlannedAction::Create { issue, .. } => state.rt.block_on(
    state.backend.create_issue(&state.project, issue)
).with_context(|| format!("create issue"))?,

PlannedAction::Update { id, patch, expected_version } => state.rt.block_on(
    state.backend.update_issue(&state.project, id, patch, Some(expected_version))
).with_context(|| format!("patch issue {}", id.0))?,

PlannedAction::Delete { id, .. } => state.rt.block_on(
    state.backend.delete_or_close(&state.project, id, DeleteReason::Abandoned)
).with_context(|| format!("delete issue {}", id.0))?,
```

Key decisions (locked — do not deviate):

- **`DeleteReason::Abandoned`** (RESEARCH Q7): the honest mapping for "git tree removed
  this file." Do NOT invent a new `DeleteReason::Unspecified`.
- **`expected_version`**: the existing `PlannedAction::Update` variant already carries a
  version. If the variant doesn't surface it today (verify by reading `diff.rs`'s
  `PlannedAction::Update` definition), add it in B2 as a compile-fix — but this is a
  quiet internal API change, not a wire change. If the variant does carry it, pass it
  through as `Some(v)`.
- **`create_issue` argument type**: `state.backend.create_issue` takes
  `Untainted<Issue>`. The `PlannedAction::Create { issue, .. }` variant must already
  yield an `Untainted<Issue>` (the sanitizer runs at the git-import blob-parse boundary);
  if it yields a raw `Issue`, wrap via the existing `sanitize` helper. Read
  `fast_import.rs` + `diff.rs` to confirm — the sanitizer is mandatory; do not bypass.

Imports to add:

```rust
use reposix_core::backend::{DeleteReason, IssueBackend, SimBackend};
use std::sync::Arc;
```

Imports to remove:

```rust
mod client;
use crate::client as api;
```

and any `use reposix_core::http::{...}` lines that become unused (`client`, `ClientOpts`,
`HttpClient` may still be imported transitively elsewhere — grep to confirm).

### Task B2.4 — `fail_push` error-path retains `some-actions-failed` kind (R5)

The existing `fail_push` call at the end of `handle_export` emits
`error refs/heads/{ref} {kind}`. Today's kinds include `backend-unreachable`,
`parse-error`, `bulk-delete`, `invalid-blob:<path>`, `some-actions-failed`.

After the refactor, if any `execute_action` call returns an error (create, update, or
delete — including the version-mismatch case), the `any_failure = true` branch flips
the kind to `some-actions-failed`, unchanged. **Do not add a `version-mismatch` kind.**

Wave D's docs sweep clarifies the SC-14-09 prose. B2's only responsibility is to
preserve today's `some-actions-failed` behavior. Sanity: after the refactor, run the
existing `tests/protocol.rs` suite and the `tests/bulk_delete_cap.rs` suite; both assert
wire-level protocol strings and should pass unchanged.

### Task B2.5 — Delete `client.rs`

```bash
git rm crates/reposix-remote/src/client.rs
```

After Task B2.3 removes the `mod client;` and `use crate::client as api;` lines, the
file is unreferenced. `cargo check -p reposix-remote` confirms.

### Task B2.6 — Prune `Cargo.toml` deps

Edit `crates/reposix-remote/Cargo.toml`:

- **Drop `thiserror`** (expected ~line 24 per RESEARCH Q8). The only user was
  `ClientError`.
- **Audit `reqwest`** (expected ~line 16). After the refactor, `main.rs` no longer
  imports `reposix_core::http::{client, ClientOpts, HttpClient}` — all HTTP machinery
  lives inside `SimBackend`. If `main.rs` (or `diff.rs`, `fast_import.rs`,
  `protocol.rs`) still imports anything from `reqwest` directly (e.g. `reqwest::Url` for
  URL validation), keep the dep. Otherwise drop it.

Confirm via:

```bash
git grep -n '^use reqwest\|^use thiserror' crates/reposix-remote/src/
```

Both should return zero hits after the refactor. Then drop the deps.

Keep: `serde`, `serde_json`, `serde_yaml`, `anyhow`, `tokio`, `clap`, `tracing`,
`tracing-subscriber`.

`cargo check -p reposix-remote --locked` green confirms the prune is correct.

### Task B2.7 — Compile + test

```bash
cargo check -p reposix-remote --locked
cargo check --workspace --locked                 # catch transitive issues
cargo test -p reposix-remote --locked            # drives the compiled binary via tests
cargo test --workspace --locked                  # full baseline
cargo clippy -p reposix-remote --all-targets --locked -- -D warnings
```

Green on all.

### Task B2.8 — Verify the remote-helper's own integration tests still drive the binary

The tests at `crates/reposix-remote/tests/protocol.rs` and
`crates/reposix-remote/tests/bulk_delete_cap.rs` exercise the **compiled binary**
end-to-end over wiremock; they assert on stdout/stderr protocol lines and on wiremock
request counts. RESEARCH Q6 + Q7 confirm these transparently continue to work after
the refactor because they assert on wire-level behavior, not code-path identity.

**Still verify** by running explicitly:

```bash
cargo test -p reposix-remote --test protocol --locked
cargo test -p reposix-remote --test bulk_delete_cap --locked
```

If either fails, the refactor broke something observable — investigate before
committing. Expected: both green with identical output to pre-refactor.

## Commit

One atomic commit:

```
refactor(14-B2): route reposix-remote through IssueBackend trait

- main.rs::State now carries Arc<dyn IssueBackend> (a SimBackend
  constructed via with_agent_suffix(spec.origin, Some("remote"))).
  Drops the old http / agent / origin fields.
- The two api::list_issues sites call state.backend.list_issues.
- execute_action dispatches to backend.create_issue / update_issue /
  delete_or_close. Delete maps to DeleteReason::Abandoned (honest
  mapping for "git tree removed this file"; RESEARCH Q7).
- fail_push retains the existing some-actions-failed wire kind for
  any execute error (including version-mismatch). SC-14-09's prose
  mentioning a distinct version-mismatch kind is aspirational;
  no new wire kind introduced (RESEARCH R5).
- reposix-remote/src/client.rs deleted (entire file). The ClientError
  enum disappears; anyhow::Error wrapping reposix_core::Error is the
  remaining error surface.
- Cargo.toml prunes thiserror; reqwest dropped iff no direct import
  remains.
- X-Reposix-Agent now emitted as reposix-core-simbackend-<pid>-remote
  (was git-remote-reposix-<pid>). Documented in CHANGELOG [Unreleased]
  ### Changed (Wave D).
- No change to parse_remote_url or the reposix:: URL syntax (LD-14-02).
- No change to diff.rs, fast_import.rs, or protocol.rs beyond compile
  fixes.
- protocol.rs + bulk_delete_cap.rs integration tests pass unchanged
  (assertions are on wire strings, not code paths).

Closes HANDOFF "Known open gaps" item 8.
See .planning/phases/14-.../14-CONTEXT.md and 14-RESEARCH.md.
```

## Tests to pass before commit

- `cargo test --workspace --locked` — green.
- `cargo test -p reposix-remote --test protocol --locked` — green.
- `cargo test -p reposix-remote --test bulk_delete_cap --locked` — green.
- `cargo clippy --workspace --all-targets --locked -- -D warnings` — clean.
- Grep proofs (run before committing):
  - `git grep -n 'api::\(list_issues\|patch_issue\|post_issue\|delete_issue\)' crates/reposix-remote/src/main.rs` → zero hits.
  - `git grep -n 'mod client\|use crate::client' crates/reposix-remote/src/main.rs` → zero hits.
  - `test ! -e crates/reposix-remote/src/client.rs` → exits 0.
  - `git grep -n 'state\.backend\.' crates/reposix-remote/src/main.rs` → at least 4 hits (list ×2, create, update, delete).

## Acceptance criteria

- [ ] `main.rs::execute_action` uses `backend.{create_issue, update_issue, delete_or_close}`.
- [ ] Both `list_issues` call sites use `backend.list_issues`.
- [ ] `DeleteReason::Abandoned` used for all deletes.
- [ ] `State` no longer carries `http`, `agent`, or `origin`.
- [ ] `client.rs` deleted. `mod client;` and `use crate::client as api;` removed.
- [ ] `thiserror` dropped from Cargo.toml. `reqwest` dropped iff unreferenced.
- [ ] `SimBackend::with_agent_suffix(origin, Some("remote"))` used for construction.
- [ ] `protocol.rs` + `bulk_delete_cap.rs` tests pass unchanged.
- [ ] Clippy green. Workspace green.
- [ ] Cargo.lock included in the commit if Cargo machinery regenerated it.

## Non-scope (reserved for other waves)

- Anything under `crates/reposix-fuse/` — B1 owns.
- Running the green-gauntlet or live write-demo — C owns.
- CHANGELOG entry for R1 and R2 — D owns. B2 only references these in the commit body.
- Any extension to `parse_remote_url` or the `reposix::` URL syntax — out of phase,
  explicitly deferred.

## Parallel-safety notes vs. B1

- B2 edits `Cargo.lock` iff dep changes cause resolver reshuffling (unlikely — dropping
  `thiserror` and `reqwest` are removals; the workspace-level lock shouldn't change
  since other crates still depend on both transitively).
- If B1 merged first, B2 rebases on `main`, re-runs `cargo check`, commits rebased
  lockfile. No manual conflict expected.
- No source files overlap with B1.
- Tests: B2 adds no tests to `sim.rs` (that's B1's territory). B2 relies on the
  existing `protocol.rs` + `bulk_delete_cap.rs` passing unchanged to prove the wire
  behavior is preserved. If adding a B2-specific test feels necessary (e.g. a unit
  test that `execute_action` correctly maps `PlannedAction::Delete` to
  `DeleteReason::Abandoned`), add it in `crates/reposix-remote/tests/` under a new
  file — do NOT add to `sim.rs`.

## References

- `14-CONTEXT.md` SC-14-04, SC-14-05, SC-14-09.
- `14-RESEARCH.md#Q4` — agent attribution (Task B2.2).
- `14-RESEARCH.md#Q6` — `api::list_issues` call sites (Task B2.3).
- `14-RESEARCH.md#Q7` — `DeleteReason::Abandoned` mapping (Task B2.3).
- `14-RESEARCH.md#Q8` — `client.rs` inventory + Cargo.toml dep prune (Tasks B2.5-B2.6).
- `14-PLAN.md` risk log — R2, R5, R9, R11.
