---
phase: 14
reviewed: 2026-04-14T17:00:05Z
reviewer: claude (gsd-code-reviewer)
depth: deep
commits_in_scope:
  - 7510ed1 test(14-A) sim 409-body pin tests
  - cd50ec5 test(14-B1) SG-03 re-home onto SimBackend
  - bdad951 refactor(14-B1) fs.rs write path through IssueBackend
  - 938b8de refactor(14-B2) remote helper through IssueBackend
files_reviewed: 7
files_reviewed_list:
  - crates/reposix-core/src/backend/sim.rs
  - crates/reposix-fuse/src/fs.rs
  - crates/reposix-fuse/src/lib.rs
  - crates/reposix-fuse/src/main.rs
  - crates/reposix-fuse/Cargo.toml
  - crates/reposix-remote/src/main.rs
  - crates/reposix-remote/Cargo.toml
findings:
  high: 0
  medium: 0
  low: 2
  info: 4
  total: 6
verdict: PASS
---

# Phase 14 CODE REVIEW

> Reviewer: Claude (gsd-code-reviewer), 2026-04-14T17:00Z.
> Scope: four commits on `main` — `7510ed1`, `cd50ec5`, `bdad951`, `938b8de`.
> Explicitly out of scope: Wave C verification doc (`4301d0d`) and Wave D docs (`547d9e0`, `142f761`).
> Method: read-only inspection via `Read`, `Grep`, `git show`. No `cargo` invocation per instructions; Wave C already verified green.

## Verdict: PASS

The refactor is clean. No HIGH- or MEDIUM-severity findings. The trait rewire
correctly routes both the FUSE write callbacks (`release`, `create`) and the
git-remote helper's `execute_action` through `IssueBackend::{update_issue,
create_issue, delete_or_close}`. Deletions are complete (no `#[allow(dead_code)]`
residue introduced by this phase, no stale `use crate::fetch::*` imports, no
`api::` references in remote). The `Untainted<Issue>` sanitize discipline holds
end-to-end: SG-03 is re-proved at the SimBackend layer, and the call-sites in
both FUSE and remote correctly pass sanitized values to the trait.

Two LOW findings are doc-drift on `ReposixFs::new` / `Mount::open` (stale "HTTP
client init" language after the HttpClient moved into SimBackend); neither
affects runtime behaviour. Four INFO notes record near-duplicate tests, a
one-liner unwrap hardening opportunity, the `_reason` parameter silently ignored
in `SimBackend::delete_or_close`, and a visibility tightening observation that
could be mentioned in CHANGELOG.

Shipping is cleared. Cross-reference verdict: matches Wave C's
`14-VERIFICATION.md` PASS verdict (Workspace tests 274/0/11, clippy -D warnings
clean, green-gauntlet --full 6/6, smoke 4/4, live demo 01 OK).

## Counts

| Severity | Count |
|----------|-------|
| HIGH     | 0     |
| MEDIUM   | 0     |
| LOW      | 2     |
| INFO     | 4     |
| **Total**| **6** |

## Focus-area summary

| Focus area                               | Verdict | Notes                                                                                            |
|------------------------------------------|---------|--------------------------------------------------------------------------------------------------|
| Trait rewire correctness (fs.rs/remote)  | PASS    | Timeouts match read-path; error mapping complete; Conflict re-parse graceful on malformed body.  |
| `update_issue_with_timeout` shape        | PASS    | 5s outer timeout via `READ_GET_TIMEOUT`; mirrors `get_issue_with_timeout` exactly.               |
| `backend_err_to_fetch` version-mismatch  | PASS    | `starts_with("version mismatch:")` + `strip_prefix` + `trim_start` tolerates with/without space. |
| `with_agent_suffix("fuse"/"remote")`     | PASS    | Both call sites pass the right literal; audit log confirms in verification.                      |
| `DeleteReason::Abandoned` in remote      | PASS    | Matches Q7 recommendation; sim discards reason but signature is honest for other backends.       |
| Untainted<Issue> discipline end-to-end   | PASS    | Traced fs.rs::release → sanitize → backend.update_issue → render_patch_body.                     |
| Test coverage re-home                    | PASS    | 16 deleted / 16 added (10 sim.rs + 6 fs.rs); SG-01, SG-03, SG-05, R13 all preserved.             |
| Dead-code removal hygiene                | PASS    | fetch.rs, tests/write.rs, client.rs absent; no `#[allow(dead_code)]` added; Cargo.toml minimal.  |
| Threat-model deltas                      | PASS    | No new egress; `X-Reposix-Agent` still on every call; audit attribution honest (R2 visible).     |
| Error surface (FUSE EIO / remote wire)   | PASS    | All variants flow correctly; `some-actions-failed` wire kind preserved (R5).                     |
| R1 assignee-clear semantic               | PASS    | No leftover "assignee untouched" tests; CHANGELOG documents (not in review scope).               |
| Adversarial `unwrap()` / `expect()`      | PASS    | Only `unwrap()` calls are in `#[cfg(test)]`; production path uses `unwrap_or(0)` for degradation.|

---

## Findings

### LOW-01 — Stale "HTTP client init" in `ReposixFs::new` doc-comment

**Severity rationale:** Doc drift only — no runtime behaviour affected. But a
reader of the `# Errors` section will be mislead about which code-path can fail
(HttpClient construction moved to `SimBackend::with_agent_suffix` in Phase 14;
`ReposixFs::new` no longer constructs an `HttpClient` at all). LOW because the
rest of the `# Errors` section is still accurate on the Tokio-runtime failure
mode.

**File:** `crates/reposix-fuse/src/fs.rs:383-387`

**Issue:**
```rust
/// Build a new FUSE filesystem whose read path is served by `backend`.
///
/// # Errors
/// Returns any error constructing the Tokio runtime or the sealed
/// [`HttpClient`] (e.g. `REPOSIX_ALLOWED_ORIGINS` un-parseable).
pub fn new(
    backend: Arc<dyn IssueBackend>,
    origin: String,
    project: String,
) -> anyhow::Result<Self> { ... }
```

After the refactor, the function body (lines 388-463) no longer calls
`client(ClientOpts::default())?` — the only fallible step is
`tokio::runtime::Builder::new_multi_thread().build()?`. The
`REPOSIX_ALLOWED_ORIGINS` unparseable case is now handled inside
`SimBackend::with_agent_suffix` (upstream of `ReposixFs::new`).

Also stale: the prose "whose **read path** is served by `backend`" — after
Phase 14 both read *and* write I/O route through the backend. Consistent with
the module-level rewrite already done at lines 37-44, but this per-method
comment was missed.

**Suggested fix:** Tighten to the actual failure mode:

```rust
/// Build a new FUSE filesystem whose read and write I/O route through
/// `backend` (Phase 10 + Phase 14).
///
/// # Errors
/// Returns any error constructing the Tokio runtime
/// (`tokio::runtime::Builder::new_multi_thread().build()`). Backend-side
/// allowlist / credential failures surface earlier in the caller
/// (`main.rs::build_backend`).
```

---

### LOW-02 — Stale "HTTP client init" and "read path" in `Mount::open` doc-comment

**Severity rationale:** Same class of doc drift as LOW-01, different site. LOW
because runtime is correct; the misleading prose is in a public-API
`# Errors`/`# Security` block that a downstream embedder would read.

**File:** `crates/reposix-fuse/src/lib.rs:65-73`

**Issue:**
```rust
/// Spawn a FUSE mount at `cfg.mount_point` whose read path is served by
/// `backend`. The mount lives until the returned [`Mount`] is dropped.
///
/// # Errors
/// Returns an error if:
/// - the mount point cannot be created,
/// - the [`ReposixFs`] fails to construct (HTTP client init, runtime),
/// - `fuser::spawn_mount2` fails ...
```

"Read path is served by `backend`" and "HTTP client init" both predate the
Phase-14 rewire. The in-line module doc at `lib.rs:1-14` was updated in this
commit to say "Both read and write I/O flow through the
[`reposix_core::IssueBackend`] trait"; this item-level doc missed the same
sweep.

**Suggested fix:** Replace the two stale phrasings:

```rust
/// Spawn a FUSE mount at `cfg.mount_point` whose read and write I/O is
/// served by `backend`. The mount lives until the returned [`Mount`] is
/// dropped.
///
/// # Errors
/// Returns an error if:
/// - the mount point cannot be created,
/// - the [`ReposixFs`] fails to construct (Tokio runtime build),
/// - `fuser::spawn_mount2` fails ...
```

---

### INFO-01 — `update_issue_409_*` pin tests nearly duplicate

**Severity rationale:** Not a defect — intentional belt+suspenders per R13's
"fires in the core crate rather than silently degrading the FUSE write path"
mitigation. The redundancy is defensible. Noting for future cleanup.

**File:** `crates/reposix-core/src/backend/sim.rs:464-548`

**Observation:** The two tests `update_issue_409_prefix_is_version_mismatch`
and `update_issue_409_current_field_present_as_json` both mount the same
wiremock response (409 with `{"error":"version_mismatch","current":7,"sent":"1"}`)
and drive the same `SimBackend::update_issue` call. The assertions differ only
in what they strip and how strictly they parse:

- First test: `msg.starts_with("version mismatch:")` + `msg.contains("\"current\":7")`
- Second test: `strip_prefix("version mismatch: ").expect(...)` + JSON parse + `current = 7`

Both cover the same wire contract from slightly different angles. The wire-level
mocked response is identical (set-up block is copy-pasted). If the sim response
shape changes, both fail — so the additional coverage from the second test is
marginal over the first.

**Suggested fix (optional):** Consolidate into one test asserting both the
prefix-form AND the JSON-parse path. A single test with two blocks of asserts
is a cheaper grounding artifact than two near-duplicates. Not a blocker — R13's
"fires loudly if contract changes" is satisfied by either test. Non-action is
fine.

---

### INFO-02 — `_reason` parameter silently discarded in `SimBackend::delete_or_close`

**Severity rationale:** Not introduced by Phase 14 — pre-existing from Phase 10.
But Phase 14 makes it live-reachable from the remote helper (which now passes
`DeleteReason::Abandoned`). Documenting for the record so future reviewers don't
re-discover it.

**File:** `crates/reposix-core/src/backend/sim.rs:283-310`

**Observation:** The sim's `delete_or_close` signature takes `_reason:
DeleteReason` but discards it — the sim performs a hard DELETE regardless of
reason. The leading-underscore prefix is the standard Rust signal that this is
intentional. The comment on line 289-292 is explicit:

> The sim performs a real DELETE regardless of reason — the reason is
> meaningful only to backends (GitHub) that close with state_reason. We
> preserve the argument in the signature so callers can write
> backend-agnostic code.

This is the correct behaviour for the sim (no state_reason concept). The remote
helper's choice of `DeleteReason::Abandoned` (Q7 recommendation) is honest and
documented.

**Suggested fix:** None. Recording this so `DeleteReason::Abandoned` vs
`::Completed` vs future variants don't accidentally get treated as
functionally different for the sim.

---

### INFO-03 — `backend_err_to_fetch` prefix-match is a string contract, not a type contract

**Severity rationale:** Not a defect — the pattern is explicitly sanctioned by
RESEARCH Q1's "extend `backend_err_to_fetch`" recommendation (string match
preferred over adding an `Error::VersionMismatch` variant per LD-14-04). But it
creates a fragile coupling across crates that merits recording.

**File:** `crates/reposix-fuse/src/fs.rs:170-194` ←→
`crates/reposix-core/src/backend/sim.rs:273-278` ←→
`crates/reposix-core/src/backend.rs:175` (doc comment)

**Observation:** The match arm at `fs.rs:176`
(`Error::Other(msg) if msg.starts_with("version mismatch:")`) depends on the
SimBackend's format-string at `sim.rs:275-278`
(`format!("version mismatch: {}", String::from_utf8_lossy(&bytes))`). Change
either in isolation and the FUSE log silently loses the `current=N` diagnostic
(the R13 sim.rs pin tests catch sim-side changes; nothing mechanically pins the
FUSE-side consumption).

The IssueBackend trait doc at `backend.rs:175` codifies this contract:
> `Err(Error::Other("version mismatch: ..."))`

So the string is the trait's documented shape, not sim-private. This is fine —
just worth knowing.

**Suggested fix:** None today. A future cleanup could introduce a
`reposix_core::Error::VersionMismatch { current: u64 }` variant if more backends
surface conflicts; for now the trait doc is the mechanical link and the R13 pin
tests are the alarm. If LD-14-04's prohibition is ever lifted, a typed variant
would be cheaper to maintain than the prefix string.

---

### INFO-04 — `FetchError` visibility tightened from `pub` to private (unnoted improvement)

**Severity rationale:** Observation, not a defect. Visibility tightening is a
positive side-effect of Phase 14; worth noting for future consumers who might
otherwise search for `pub use fetch::FetchError` in another crate.

**File:** `crates/reposix-fuse/src/fs.rs:115-150` (vs
deleted `crates/reposix-fuse/src/fetch.rs:37-72` which declared `pub enum
FetchError`).

**Observation:** The old `fetch::FetchError` was `pub` (the `pub mod fetch;`
declaration in `lib.rs` propagated to the enum). After Phase 14 the `FetchError`
enum is declared at module scope inside `fs.rs` with no visibility modifier —
effectively private to the `fs` module. Nothing outside `fs.rs` can construct
or pattern-match a `FetchError` anymore.

This is strictly better: the type was never intended as a cross-crate surface,
and the new privacy prevents accidental surface growth. No downstream code
pattern-matched `FetchError` (confirmed by zero hits on `use reposix_fuse::fetch`
before the refactor and zero hits on anything `FetchError`-shaped in other
crates today).

**Suggested fix:** None. Consider adding a one-line note to `CHANGELOG.md`
`### Changed` that `FetchError` is now private (low probability of impact, but
honest documentation is cheap).

---

## Audit checklist (focus-area mapping)

| Focus area                                                      | Evidence                                                                                                                                                    |
|-----------------------------------------------------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------|
| 1. `update_issue_with_timeout` / `create_issue_with_timeout`    | `fs.rs:219-257`. 5s `READ_GET_TIMEOUT`. Pattern matches `list_issues_with_timeout:200-205` / `get_issue_with_timeout:207-217`. Error mapping via `backend_err_to_fetch`.                            |
| 1. `backend_err_to_fetch` version-mismatch arm                  | `fs.rs:176-191`. `starts_with("version mismatch:")` guard, `strip_prefix` + `trim_start` tolerates the trailing-space variant sim emits, `unwrap_or(0)` graceful degradation.                       |
| 1. `Arc<dyn IssueBackend>` in fs.rs and remote                  | `fs.rs:388-89`, `remote/main.rs:78-79`. Both call `with_agent_suffix(..., Some("fuse"|"remote"))`. Literal values match R2.                                  |
| 1. `DeleteReason::Abandoned` in remote                           | `remote/main.rs:326`. Matches Q7 recommendation. Sim ignores (pre-existing).                                                                               |
| 1. Untainted<Issue> end-to-end                                   | fs.rs::release (lines 1122-1145): `frontmatter::parse → Tainted::new → sanitize → update_issue_with_timeout`. No `.into_inner()` / `.inner_ref()` in FUSE/remote. Only SimBackend unwraps (`sim.rs:244,260`). |
| 2. SG-03 re-home                                                 | `sim.rs:733-799` `update_issue_respects_untainted_sanitization`. Injects `version=999_999` via Tainted, asserts wire body has no `version`/`id`/`created_at`/`updated_at` keys.                     |
| 2. Re-home preserves assertions                                 | If-Match quoted etag `sim.rs:619` (`"\"3\""`); 409 re-parse `sim.rs:464-548`; timeout `fs.rs:1417-1471`; wildcard-If-Match preserved pre-existing at `sim.rs:545-585`.                               |
| 2. Silent test drops                                             | None. Delta: -16 (11 fetch.rs + 5 tests/write.rs) / +16 (10 sim.rs + 6 fs.rs). Commit body is accurate.                                                    |
| 2. R13 adequacy                                                  | `sim.rs:464-548` two pin tests. Both assert the `{"current":7,"sent":"1"}` body shape and `version mismatch:` prefix. INFO-01 flags near-duplication.                                                |
| 3. Deleted files absent                                          | `ls crates/reposix-fuse/src/`: no `fetch.rs`. `ls crates/reposix-fuse/tests/`: no `write.rs`. `ls crates/reposix-remote/src/`: no `client.rs`.                                                       |
| 3. No stale imports                                              | `grep -n 'crate::fetch' crates/reposix-fuse/src/`: only `fs.rs:108` (historical doc-comment). `grep -n 'use crate::client\|mod client' crates/reposix-remote/src/`: zero. `grep -n 'api::' crates/reposix-remote/src/`: zero.                                                                                                                                                     |
| 3. Cargo.toml dep list                                           | `reposix-remote/Cargo.toml`: `reqwest` dropped ✓, `thiserror` retained (diff.rs::PlanError uses `#[derive(Error)]` ✓). `reposix-fuse/Cargo.toml`: `reqwest` retained because `FetchError::Transport(#[from] reqwest::Error)` still uses the type. Minimum-scope prune stands.    |
| 3. `#[allow(dead_code)]` residue                                 | Two pre-existing in `remote/src/protocol.rs:93,141` and one in `remote/src/diff.rs:26`, all from Phase S. None added in Phase 14. LD-14-07 satisfied.                                                |
| 4. No new egress / allowlist widening                            | `fs.rs` no longer owns `HttpClient`; all egress goes through `SimBackend`'s sealed `HttpClient`. `remote/main.rs` dropped its own `HttpClient` too. Allowlist gate unchanged.                       |
| 4. `X-Reposix-Agent` on every call                               | `sim.rs:91-100`: `agent_only()` and `json_headers()` both include the header. All trait methods call one of the two (GET/DELETE → `agent_only`; POST/PATCH → `json_headers`).                       |
| 4. R2 honesty                                                    | `main.rs:89` (`fuse`), `remote/main.rs:79` (`remote`). Audit log verification in `14-VERIFICATION.md` confirms `reposix-core-simbackend-{pid}-{fuse,remote}` rows present.                            |
| 5. Error mapping exhaustive                                      | `fs.rs:170-194`. `InvalidOrigin → Origin`; `Http → Transport`; `Json → Parse`; `Other("not found...") → NotFound`; `Other("version mismatch:...") → Conflict{current}`; else `Other → Core`.         |
| 5. Remote `fail_push` on backend errors                           | `remote/main.rs:178-186, 220-228` (list), and `257-271` (execute_action loop). `some-actions-failed` wire kind on any `Err` from `execute_action`. R5 satisfied.                                    |
| 6. R1 test hygiene                                               | No test asserts "assignee untouched across PATCH" (verified by grep `assignee.*untouched|preserve.*assignee` → zero hits). Correctly removed.                                                       |
| 7. Timeout value + error map                                     | `fs.rs:104`: `const READ_GET_TIMEOUT: Duration = Duration::from_secs(5);` used in `update_issue_with_timeout:233` and `create_issue_with_timeout:252`. `fetch_errno:645` maps `Timeout → EIO`.       |
| 8. No added `unwrap()`/`expect()` on adversarial input           | Production: only `.unwrap_or(0)` on JSON re-parse fallback (fs.rs:189), no unwraps on Tainted/Untainted values. Tests: unwraps confined to `#[cfg(test)]` blocks. No adversarial path panics.        |
| 8. Untainted not unwrapped in FUSE/remote                         | `grep -n '\.inner_ref()' crates`: only `sim.rs:244,260` (correct — consumption point just before wire rendering). FUSE and remote never unwrap Untainted.                                              |

---

## Non-blocking observations

- **`self.origin` in ReposixFs is live-dead.** The `origin` field is stored in
  `ReposixFs` only for the `Debug` impl (`fs.rs:374`). All runtime I/O uses
  `self.backend.*`. Could be dropped in a future cleanup, but keeping it
  documents the mount's intent for diagnostics. Already noted in the
  doc-comment (`fs.rs:329-330`). No action.

- **`MountConfig.origin` similarly retained for diagnostics.** The
  doc-comment at `lib.rs:31-35` correctly explains this. No action.

- **`remote/main.rs` retains the `#[allow(clippy::print_stderr)]` on
  `diag()`.** Correct — this is pre-existing and needed for stderr diagnostic
  output. Documented in the opening doc-comment block.

- **`ConflictBody` doc-comment claim of `#[serde(default)]` is slightly
  imprecise.** The comment at `fs.rs:152-156` says:
  > The `#[serde(default)]` container implicit in omitting other fields
  > means extra keys are ignored (forward-compatible).

  serde's default behaviour (*without* `deny_unknown_fields`) is to ignore
  unknown fields. No `#[serde(default)]` attribute is actually present. The
  behaviour described is correct; the mechanism description is imprecise.
  Too trivial to flag as a finding; noting here.

- **Test for `create_issue_with_timeout` happy path is missing.** There's
  `update_issue_with_timeout_happy_path_returns_issue` (`fs.rs:1473-1502`)
  but no symmetric `create_issue_with_timeout_happy_path_returns_issue`.
  The timeout-limb test (`fs.rs:1449-1471`) covers failure-path; the
  Ok-path of the wrapper is effectively tested via
  `create_issue_returns_authoritative_issue` in sim.rs (which tests
  `SimBackend::create_issue` directly, not the wrapper). Not a gap serious
  enough to flag — the wrapper is 7 lines of trivial timeout dispatch
  code; the update variant's happy-path test proves the pattern, and
  symmetry with `create` is visible at a glance. Could add 15 lines if a
  future phase wants perfect symmetry. No action.

---

## Reviewer sign-off

This review is cleared to land. The two LOW findings are doc-drift only and can
be addressed in a follow-up commit (or rolled into the next adjacent docs
commit). Tag `v0.4.1` is not blocked.

If the fix-before-tag option is desired, a 10-line edit to the two
doc-comments (LOW-01, LOW-02) closes the drift. Either path is acceptable.

_Reviewed: 2026-04-14T17:00:05Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: deep (cross-file trace of the PATCH path through fs.rs → IssueBackend → SimBackend::render_patch_body; cross-check of test re-home deltas and R13 pin adequacy)_
