# Phase 14 CONTEXT

> Status: scoped in session 5, 2026-04-14. Orchestrator: Claude Opus 4.6 1M.
> Rationale: `.planning/SESSION-5-RATIONALE.md`.

## Phase identity

**Name:** Decouple sim REST shape from FUSE write path and git-remote helper — route through `IssueBackend` trait.

**Cluster label:** Session-5 Cluster B.

**Scope tag:** v0.4.1 (bugfix/refactor — no user-visible feature changes, no CHANGELOG `### Added`). Reason: behavior on the sim remains identical; what changes is who speaks the sim's REST shape. External CLI, FUSE mount semantics, and remote-helper URL syntax are all unchanged.

**Requirements mapping:** Closes v0.3-era HANDOFF "Known open gaps" items 7 and 8 — (7) "FUSE write path through `IssueBackend::update_issue`" and (8) "`git-remote-reposix` rewire through `IssueBackend`." Both flagged in session-3 handoff, reaffirmed in session-4 open-problems rollup, reaffirmed again in session-5 brief as load-bearing.

## Goal (one paragraph)

Remove the last two hardcoded references to the simulator's REST shape outside `reposix-sim` itself. Today `crates/reposix-fuse/src/fetch.rs::{patch_issue, post_issue}` and `crates/reposix-remote/src/client.rs` speak `PATCH /projects/{p}/issues/{id}` and `POST /projects/{p}/issues` directly; both are unreachable by any non-sim backend. After this phase, every write the FUSE mount performs (`release`, `create`) and every write the git-remote helper performs (`export` path) goes through the `IssueBackend` trait's `create_issue` / `update_issue` / `delete_or_close` methods. The trait is the seam; the sim becomes one implementation among many.

## Success criteria

- **SC-14-01 (FUSE refactor):** `crates/reposix-fuse/src/fs.rs::release` calls `self.backend.update_issue(project, id, patch, Some(version))` — not `fetch::patch_issue(...)`. Prove by grep: zero `patch_issue` / `post_issue` references in `crates/reposix-fuse/src/fs.rs`.
- **SC-14-02 (FUSE create):** `crates/reposix-fuse/src/fs.rs::create` calls `self.backend.create_issue(project, issue)` — not `fetch::post_issue(...)`. Grep proof as above.
- **SC-14-03 (FUSE dead-code removal):** `crates/reposix-fuse/src/fetch.rs` retains only the read-path helpers (`fetch_issues`, `fetch_issue`) OR is deleted entirely if the read helpers also move. `EgressPayload`, `ConflictBody`, `FetchError::Conflict`, `FetchError::BadRequest` are removed iff unused. `patch_issue`, `post_issue` are removed.
- **SC-14-04 (remote refactor):** `crates/reposix-remote/src/main.rs::execute_action` dispatches to `IssueBackend::{create_issue, update_issue, delete_or_close}` — not `api::{post_issue, patch_issue, delete_issue}`. Import-batch path uses `IssueBackend::list_issues`. Grep proof: zero `api::` references in `main.rs`.
- **SC-14-05 (remote dead-code removal):** `crates/reposix-remote/src/client.rs` is deleted. The `mod client;` line in `main.rs` is removed. `ClientError` variants are replaced with either `reposix_core::Error` (directly) or a small local enum if needed for protocol diagnostics.
- **SC-14-06 (tests follow):** The `#[cfg(test)] mod tests` block formerly in `fetch.rs` that exercised `patch_issue` / `post_issue` is re-homed onto `SimBackend` against a wiremock server — same assertions (409 conflict, 5s timeout, If-Match, egress shape).
- **SC-14-07 (integration still green):** `cargo test --workspace --locked` green. `cargo clippy --workspace --all-targets --locked -- -D warnings` green. `bash scripts/green-gauntlet.sh --full` green (includes `--ignored` FUSE integration tests which mount a real FUSE filesystem over a sim, exercise write path, read back).
- **SC-14-08 (smoke / E2E unchanged):** `bash scripts/demos/smoke.sh` 4/4. `bash scripts/demos/02-remote-helper.sh` (or equivalent write demo) exits 0 — the user-visible behavior of writing through FUSE and pushing through git-remote has not regressed.
- **SC-14-09 (error surface):** Version-mismatch errors from the sim still render as `libc::EIO` to the kernel via FUSE and as `error refs/heads/main version-mismatch` (or equivalent) to git via the remote helper. Specifically: the trait's `Result<_, reposix_core::Error>` + `"version mismatch"` string match lands in the same FUSE warn log and same remote-helper `fail_push` path.
- **SC-14-10 (documentation):** `docs/architecture.md` (or wherever the "FUSE talks to backend via trait" diagram lives) has no lingering prose about "the write path still speaks the sim REST shape." CLAUDE.md sub-section mentioning the v0.3 deferral is updated to note Phase 14 closed it.

## Locked decisions

- **LD-14-01 — Keep `IssueBackend` trait unchanged.** We do NOT add new variants to `BackendFeature`, new methods, or break dyn-compatibility. The trait already has `create_issue`, `update_issue`, `delete_or_close` — we just start using them.
- **LD-14-02 — `RemoteSpec` / `parse_remote_url` unchanged.** The URL syntax stays `reposix::http://host:port/projects/slug`. A future phase can extend to `reposix::confluence://tenant/space`; not this one. Reason: scope discipline — this phase is a refactor, not a feature.
- **LD-14-03 — Remote helper stays sim-only behaviorally.** It constructs an `Arc<SimBackend>` internally from the parsed `RemoteSpec`. No `--backend` flag, no scheme-based dispatch. Tests continue to prove sim round-trip only.
- **LD-14-04 — Error translation is one-way.** `reposix_core::Error` → FUSE `FetchError` (or replacement) for the FUSE path; `reposix_core::Error` → `anyhow::Error` + protocol code for the remote path. We do NOT add a new `trait Error` layer; both surfaces consume the existing `reposix_core::Error` variants (`InvalidOrigin`, `Http`, `Json`, `Other(String)`).
- **LD-14-05 — Wiremock tests stay, point at `SimBackend`.** The tests formerly in `fetch.rs` for `patch_issue` / `post_issue` (If-Match header, 409 parsing, 5s timeout, egress shape only) are preserved verbatim in assertion content, but set up via `SimBackend::new(server.uri(), agent)` and call `backend.update_issue(...)`/`create_issue(...)`. This proves the trait impl honors the same wire contract.
- **LD-14-06 — No behavior change on allowlist / timeout.** The 5-second wall-clock ceiling stays. The `HttpClient` with allowlist still wraps every egress. `SimBackend::new` already constructs the `HttpClient` via `client(ClientOpts::default())`; this phase doesn't need to touch that code.
- **LD-14-07 — Deletion of `fetch.rs` write-half is atomic with the fs.rs refactor.** Don't leave `pub fn patch_issue` as dead code behind `#[allow(dead_code)]`. That's exactly the grounding-rot we're paying down.
- **LD-14-08 — Test count must not regress.** Session-4 close: 272 tests. Post-phase-14: ≥ 272, likely slightly up due to new integration tests. If a test is genuinely obsolete (e.g. it tested the old `EgressPayload` struct), explain in the commit message.

## Non-goals / scope boundaries

- Do NOT add real-backend write support (Cluster A). ConfluenceBackend's `create_issue`/`update_issue`/`delete_or_close` remain `not supported`. GitHub backend stays read-only. Adding those is Cluster A scope.
- Do NOT widen the remote-helper URL syntax to support backend schemes.
- Do NOT start Phase 12 subprocess ABI work.
- Do NOT reorg `.planning/phases/` or rename the fetch-module file path (if any survives). Minimal blast radius.
- Do NOT touch the read path (`list_issues` / `get_issue`) unless a test needs it — the read path already goes through `IssueBackend`.

## Canonical refs

- `crates/reposix-core/src/backend.rs` — trait definition, `BackendFeature`, `DeleteReason`.
- `crates/reposix-core/src/backend/sim.rs` — `SimBackend` impl to study the wire shape.
- `crates/reposix-fuse/src/fs.rs:89` — `use crate::fetch::{patch_issue, post_issue, FetchError};` (the enemy).
- `crates/reposix-fuse/src/fs.rs:1029` — call site 1 (PATCH on release).
- `crates/reposix-fuse/src/fs.rs:1110` — call site 2 (POST on create).
- `crates/reposix-fuse/src/fetch.rs` — write helpers to remove.
- `crates/reposix-remote/src/main.rs:181` — `api::list_issues` import-path call.
- `crates/reposix-remote/src/main.rs:228` — `api::list_issues` export prior-state call.
- `crates/reposix-remote/src/main.rs:296-358` — `execute_action` with `api::post_issue`/`patch_issue`/`delete_issue`.
- `crates/reposix-remote/src/client.rs` — module to delete.
- `crates/reposix-remote/Cargo.toml` — may drop `thiserror` / `serde` deps if `ClientError` goes.
- `.planning/SESSION-5-RATIONALE.md` — decision doc for this phase.
- HANDOFF.md "Known open gaps" items 7 + 8 — the motivating requirements.

## Dependency on prior phases

- Phase 10 (FUSE rewire through `IssueBackend`) made the read path trait-driven; this phase does the same for the write path. Read-path pattern in `fs.rs::list_issues_with_timeout` / `get_issue_with_timeout` is the template.
- Phase 13 Wave C bucket-dir + tree overlay is unchanged by this phase — we're touching the write callbacks, not the inode layout.

## Threat model deltas

No new egress. No new allowlist widening. The trait is equivalent-or-narrower than `fetch::patch_issue` in surface area. Audit log behavior is unchanged — the SimBackend delegates to the same sim endpoint, which writes the same audit row.

One subtle consideration: `fetch::patch_issue` returned a `FetchError::Conflict { current }` parsed out of the 409 body. The `IssueBackend::update_issue` contract currently surfaces "version mismatch: ..." as `Error::Other(String)`. The FUSE callback's branch that used `FetchError::Conflict` to log `current` and emit `EIO` needs to match on the `Other` string or have the sim-backend conflict path extract `current` into a structured error. Preferred: pattern-match on `"version mismatch"` in the Other string (cheap, consistent with the existing "not found" pattern). Alternative: extend `reposix_core::Error` with a typed `VersionMismatch { current: u64 }` variant — bigger change, skip unless the `Other` pattern is too brittle.

## Waves (sketch — plan phase will finalize)

- **Wave A (serial, 15 min):** Research the exact error-mapping decision (VersionMismatch variant vs string-match). Confirm SimBackend already returns the `current` value in an `Error::Other("version mismatch: current=N")` string — if yes, string-match is a trivial refactor; if no, add it.
- **Wave B1 (parallel, 45 min):** `crates/reposix-fuse` refactor. One agent. Touches `fs.rs`, `fetch.rs`, `fetch.rs` tests, possibly `lib.rs`.
- **Wave B2 (parallel, 45 min):** `crates/reposix-remote` refactor. Second agent, disjoint filesystem. Touches `main.rs`, deletes `client.rs`, re-homes `client.rs` tests if any.
- **Wave C (serial, 30 min):** Green-gauntlet --full, smoke, FUSE --ignored integration. Live-run write demos. Fix anything red.
- **Wave D (serial, 15 min):** CHANGELOG entry under `[Unreleased]`. Docs sweep (CLAUDE.md + `docs/architecture.md` if deferral prose exists there).

Total est. wall-clock: ~2h15m (vs the 7× parallelism budget available, this is comfortable).
