# Phase 14 — Research

> Researcher: Claude Opus 4.6 (opus-4-6[1m]), 2026-04-14.
> Input: `14-CONTEXT.md` in this directory (read).
> Scope: internal refactor inside `reposix-fuse` and `reposix-remote`, wiring the write path onto the already-existing `IssueBackend` trait. No wire-shape changes; the sim is the only concrete backend.
> Output consumers: the planner that will split this into waves, and the executors in Waves B1 and B2.

All file/line citations are against the tree as of this commit; no speculative code is offered.

---

## tl;dr

1. **The 409 conflict `current` value is already present in the `SimBackend::update_issue` error string — but embedded in a JSON blob, not as a structured field.** `crates/reposix-core/src/backend/sim.rs:273-280` returns `Error::Other(format!("version mismatch: {raw_body}"))` where `raw_body` is the JSON `{"error":"version_mismatch","current":7,"sent":"1"}`. The planner should **extend `fs.rs::backend_err_to_fetch` to recognize the `"version mismatch:"` prefix and re-parse the tail as JSON** to recover the `current` field into `FetchError::Conflict { current }`. No new `reposix_core::Error` variant is needed (aligns with `LD-14-04`). If the JSON parse fails (e.g. the sim ever changes the body format), degrade gracefully to `FetchError::Conflict { current: 0 }` just like the old parser in `fetch.rs:245-249` did.

2. **Delete the entire `reposix-remote/src/client.rs` file and all of `reposix-fuse/src/fetch.rs` in the same refactor commit.** Both files are completely unreferenced outside their own test blocks once `fs.rs` and `main.rs` stop calling them. The FUSE `fetch::fetch_issue` / `fetch::fetch_issues` read-path helpers are **already unused in the production fs.rs read path** (Phase 10 replaced them with `list_issues_with_timeout` / `get_issue_with_timeout`) — they only survive because the `#[cfg(test)]` block inside `fetch.rs` still exercises them. Drop `pub mod fetch;` from `crates/reposix-fuse/src/lib.rs:23`. Delete `crates/reposix-fuse/tests/write.rs` (the only external importer of `reposix_fuse::fetch`) and port its assertions into `crates/reposix-core/src/backend/sim.rs` tests (which already have a solid wiremock scaffold at `sim.rs:363-513`).

3. **There is one wire-shape difference the planner must reconcile before this passes CI.** `crates/reposix-fuse/src/fetch.rs:227` sends `If-Match: 3` (unquoted). `crates/reposix-core/src/backend/sim.rs:263,266` sends `If-Match: "3"` (RFC-7232 quoted). The sim accepts both (`crates/reposix-sim/src/routes/issues.rs:128-133` strips outer quotes), so functionality is preserved. But the integration test `crates/reposix-fuse/tests/write.rs:42` hard-asserts `header("If-Match", "1")` (unquoted), and the port-target `crates/reposix-core/src/backend/sim.rs:424` asserts `header("If-Match", "\"5\"")` (quoted). When tests get re-homed onto `SimBackend`, they MUST update to the quoted form. Call this out in the plan so the executor doesn't get whiplash between the two assertion styles.

---

## Phase Requirements

Mapping from `14-CONTEXT.md` success criteria to this research:

| ID | SC summary | Research support |
|----|------------|-------------------|
| SC-14-01 | `fs.rs::release` calls `backend.update_issue` | Q1 + Q3 (error mapping + timeout wrapper needed) |
| SC-14-02 | `fs.rs::create` calls `backend.create_issue` | Q2 + Q3 (BadRequest surface + timeout) |
| SC-14-03 | `fetch.rs` write-path removed | Q5 + Q8 (read helpers already unused; full inventory in Q8) |
| SC-14-04 | `main.rs::execute_action` uses `IssueBackend` | Q6 + Q7 (list shape + delete reason mapping) |
| SC-14-05 | `client.rs` deleted | Q8 (inventory + `Cargo.toml` impact) |
| SC-14-06 | Tests re-homed onto `SimBackend` | Q10 (test migration table) + tl;dr #3 (If-Match quoting change) |
| SC-14-07 | `cargo test --workspace` + clippy green, green-gauntlet --full green | Depends only on successful implementation; Q9 confirms no new egress |
| SC-14-08 | Smoke/E2E unchanged | Q6, Q9 — no wire-shape regression |
| SC-14-09 | Conflict still renders as EIO / `error refs/heads/main version-mismatch` | Q1 (error translation mechanism) |
| SC-14-10 | Docs sweep | Q8 (docs strings in `lib.rs:9-10,38-43`, `fs.rs:42-43,201-204` need editing) |

---

## Q1 — Error mapping: how does `SimBackend` surface the 409 conflict today?

### Finding

Today, `SimBackend::update_issue` detects the 409, reads the body bytes, and returns `Error::Other("version mismatch: <raw-body-as-utf8-lossy>")`. The `current` integer is **present as a substring** inside that UTF-8-lossy render of the raw JSON body (`{"error":"version_mismatch","current":7,"sent":"1"}`), but it is **not a structured field of the error enum**. So option (a) in the question's taxonomy — "string-parse the `Error::Other` message to recover `current`" — is the correct description, but with an important nuance: what's embedded is **JSON**, not the decimal integer alone, so the "string parse" is really a JSON parse of the tail of the message.

Today's FUSE `fs.rs::backend_err_to_fetch` does **not** special-case this pattern (it would currently fall through to `FetchError::Core(..)` on any version mismatch from the trait path). So to preserve the current diagnostic quality — where `release` logs the `current` version alongside the EIO reply — the planner must add a new arm.

### Evidence

- `crates/reposix-core/src/backend/sim.rs:272-280` — the 409 arm:
  ```
  let status = resp.status();
  if status == StatusCode::CONFLICT {
      let bytes = resp.bytes().await?;
      return Err(Error::Other(format!(
          "version mismatch: {}",
          String::from_utf8_lossy(&bytes)
      )));
  }
  ```
- `crates/reposix-sim/src/error.rs:83-91` — the sim always emits the 409 body as JSON with `{"error":"version_mismatch","current":<u64>,"sent":<string>}`:
  ```
  Self::VersionMismatch { current, sent } => {
      json!({
          "error": kind,
          "current": current,
          "sent": sent,
      })
  }
  ```
- `crates/reposix-sim/src/routes/issues.rs:587-607` is the `patch_with_bogus_if_match_returns_409` route test that pins the body shape. Planner can rely on this: if the shape ever changes, the sim's own test catches it.
- `crates/reposix-fuse/src/fs.rs:108-116` — the current `backend_err_to_fetch` has no arm for version-mismatch:
  ```
  fn backend_err_to_fetch(e: reposix_core::Error) -> FetchError {
      match e {
          reposix_core::Error::InvalidOrigin(o) => FetchError::Origin(o),
          reposix_core::Error::Http(t) => FetchError::Transport(t),
          reposix_core::Error::Json(j) => FetchError::Parse(j),
          reposix_core::Error::Other(msg) if msg.starts_with("not found") => FetchError::NotFound,
          other => FetchError::Core(other.to_string()),
      }
  }
  ```
  So a 409 from the trait path ends up in the catch-all `FetchError::Core(..)` arm. That still maps to `EIO` via `fs.rs::fetch_errno` (line 537 lumps `Core` and `Conflict` together), but the diagnostic branch at `fs.rs:1052-1058` — which logs `current` alongside the warning — would become unreachable.
- `crates/reposix-fuse/src/fs.rs:1052-1058` — the existing diagnostic arm:
  ```
  Err(FetchError::Conflict { current }) => {
      warn!(
          ino = ino_u,
          current, "release: 409 conflict — user must git pull --rebase"
      );
      reply.error(fuser::Errno::from_i32(libc::EIO));
  }
  ```

### Recommendation for planner

The planner should **extend `backend_err_to_fetch`** (not introduce a new `reposix_core::Error` variant — LD-14-04 forbids that) with an arm that matches the `"version mismatch:"` prefix, strips it, and `serde_json::from_str::<ConflictBody>`s the JSON tail. Fall back to `FetchError::Conflict { current: 0 }` on JSON parse failure, mirroring the existing unwrap-or at `fetch.rs:246`. Concretely:

- Add an arm before the catch-all: `reposix_core::Error::Other(msg) if msg.starts_with("version mismatch:") => { ... parse tail ... FetchError::Conflict { current } }`.
- Keep `ConflictBody` (currently a private struct in `fetch.rs:103-106`) — move it into `fs.rs` next to `backend_err_to_fetch`. The `FetchError::Conflict` variant itself is retained for the `release` callback's diagnostic.
- This preserves SC-14-09 verbatim: version-mismatch surfaces as `libc::EIO`, AND the "current=N" log line survives.

Alternative considered and rejected: extending `reposix_core::Error` with `VersionMismatch { current: u64 }`. That's a cleaner error model long-term, but it (a) breaks the "do not add new Error variants" envelope in LD-14-04, (b) ripples through every backend impl that would have to map its own version-mismatch taxonomy onto it (future GitHub `If-Unmodified-Since`, Confluence optimistic-concurrency response, etc.), and (c) is irrelevant for v0.4.1 scope.

---

## Q2 — BadRequest surface on create

### Finding

`SimBackend::create_issue` does **not** produce a `FetchError::BadRequest`-equivalent variant — any 4xx response body is collapsed into `Error::Other(format!("sim returned {status} for {context}: {body}"))` via the shared `decode_issue` helper. The 4xx body **is preserved as a substring of the `Error::Other` message**, so no information is silently dropped, but the discriminating `FetchError::BadRequest` variant disappears from the FUSE errno mapping.

Today's `fs.rs::fetch_errno` maps both `FetchError::BadRequest` and `FetchError::Core` to `libc::EIO` (see `fs.rs:534-539`), so the user-visible errno is **identical**. The information loss is limited to (a) the `warn!` log line no longer distinguishing "bad request" from "transport error" — both just say "PATCH failed" / "create: POST failed", and (b) `FetchError::BadRequest(msg)` being a more operator-readable error message than `FetchError::Core(msg)` because `Core` prefixes `"core: "` via its `thiserror` derive. Marginal degradation. Acceptable given LD-14-04.

### Evidence

- `crates/reposix-core/src/backend/sim.rs:242-250` — `create_issue` delegates every non-success branch to `decode_issue`:
  ```
  async fn create_issue(&self, project: &str, issue: Untainted<Issue>) -> Result<Issue> {
      let url = format!("{}/projects/{}/issues", self.base(), project);
      let body = render_create_body(issue.inner_ref())?;
      let resp = self
          .http
          .request_with_headers_and_body(Method::POST, &url, &self.json_headers(), Some(body))
          .await?;
      decode_issue(resp, &url).await
  }
  ```
- `crates/reposix-core/src/backend/sim.rs:106-120` — `decode_issue` surfaces the body:
  ```
  async fn decode_issue(resp: reqwest::Response, context: &str) -> Result<Issue> {
      let status = resp.status();
      let bytes = resp.bytes().await?;
      if status == StatusCode::NOT_FOUND {
          return Err(Error::Other(format!("not found: {context}")));
      }
      if !status.is_success() {
          return Err(Error::Other(format!(
              "sim returned {status} for {context}: {}",
              String::from_utf8_lossy(&bytes)
          )));
      }
      ...
  }
  ```
- `crates/reposix-fuse/src/fs.rs:532-540` — `BadRequest` and `Core` share the `EIO` bucket:
  ```
  FetchError::Timeout
  | FetchError::Transport(_)
  | FetchError::Status(_)
  | FetchError::Parse(_)
  | FetchError::Core(_)
  | FetchError::Conflict { .. }
  | FetchError::BadRequest(_) => libc::EIO,
  ```
- The only wiremock test exercising the POST 4xx shape today is implicit — `post_issue_sends_egress_shape_only` at `crates/reposix-fuse/src/fetch.rs:546-569` and `crates/reposix-fuse/tests/write.rs:215-236` both exercise 201 happy-path only.

### Recommendation for planner

Accept the information loss. **Do not** try to preserve `FetchError::BadRequest` by pattern-matching on the `Error::Other` message — the match surface would be brittle (the format string is `"sim returned {status} for {context}: {body}"` which could legitimately come from a 500 as well, not just a 4xx).

Two concrete actions:

1. Drop the `FetchError::BadRequest` variant entirely (see Q8 dead-code inventory).
2. Verify via a new test in `crates/reposix-core/src/backend/sim.rs` that a sim 400 on POST surfaces as an `Err(Error::Other)` whose message contains the body bytes — a low-value but cheap regression guard.

Note SC-14-06 re-homing: there was no `post_issue_4xx_is_bad_request` test in the tree to re-home. The "POST 4xx body preserved" assertion is new (if the planner wants it) but not required.

---

## Q3 — Timeout discipline

### Finding

Today there are **two independent 5-second ceilings** that protect the FUSE write callback:

- **Outer (belt):** `tokio::time::timeout(FETCH_TIMEOUT, http.request_with_headers_and_body(...))` in `fetch::patch_issue` and `fetch::post_issue`. Same `FETCH_TIMEOUT = Duration::from_secs(5)` (`fetch.rs:27`).
- **Inner (suspenders):** `ClientOpts::default().total_timeout = Duration::from_secs(5)` set inside the sealed `HttpClient` at construction time (`http.rs:40-55`).

`SimBackend` uses the **inner** timeout only. The `HttpClient` is constructed with `ClientOpts::default()` at `crates/reposix-core/src/backend/sim.rs:71`, so every HTTP request from the trait impl already carries reqwest's 5s total-timeout. **There is no outer `tokio::time::timeout` wrapper inside `SimBackend` for any method** (`list_issues`, `get_issue`, `create_issue`, `update_issue`, `delete_or_close` all go straight through to `self.http.request_with_headers_and_body(...)` without a Tokio timeout).

This means removing the FUSE outer wrapper is a **real loss of defence in depth**. The two cases the outer timeout protects that the inner timeout might miss:

1. **Body-streaming stalls after the response headers arrived.** reqwest's `total_timeout` covers up through header-read; then `resp.bytes()` is a separate future. The existing `fetch::patch_issue` wraps `resp.bytes()` in a second `tokio::time::timeout(FETCH_TIMEOUT, resp.bytes())` (lines 241-244 and 257-260). `SimBackend` has no such wrapping — `resp.bytes().await?` at `sim.rs:108` is unbounded.
2. **The `HttpClient` gate itself stalling.** Theoretically the allowlist check is sync and fast, but if the env-var parse hit a pathological input, the inner timeout would not fire until after reqwest started work.

In practice (1) is the real concern — a backend that returns a 200 header line and then stops sending bytes. The existing FUSE read path already has this gap covered by `list_issues_with_timeout` / `get_issue_with_timeout` wrapping the *whole* backend call (`fs.rs:122-126, 134-138`).

### Evidence

- `crates/reposix-fuse/src/fetch.rs:27` — `FETCH_TIMEOUT`.
- `crates/reposix-fuse/src/fetch.rs:231-237, 241-244, 257-260` — the triple `tokio::time::timeout(FETCH_TIMEOUT, ...)` wrappers in `patch_issue`.
- `crates/reposix-core/src/http.rs:40-55` — `ClientOpts::default().total_timeout = Duration::from_secs(5)`.
- `crates/reposix-core/src/backend/sim.rs:71` — `let http = client(ClientOpts::default())?;`.
- `crates/reposix-core/src/backend/sim.rs:225-231, 234-239, 244-250, 263-281, 293-310` — none of `SimBackend`'s methods wrap their HTTP call in an outer `tokio::time::timeout`.
- `crates/reposix-fuse/src/fs.rs:117-127` — the existing template for wrapping a backend call:
  ```
  async fn list_issues_with_timeout(
      backend: &Arc<dyn IssueBackend>,
      project: &str,
  ) -> Result<Vec<Issue>, FetchError> {
      match tokio::time::timeout(READ_LIST_TIMEOUT, backend.list_issues(project)).await {
          Ok(Ok(v)) => Ok(v),
          Ok(Err(e)) => Err(backend_err_to_fetch(e)),
          Err(_) => Err(FetchError::Timeout),
      }
  }
  ```

### Recommendation for planner

**Add matching `update_issue_with_timeout` / `create_issue_with_timeout` helpers in `fs.rs`, parallel to the existing `list_issues_with_timeout` / `get_issue_with_timeout`.** Use `READ_GET_TIMEOUT = Duration::from_secs(5)` (already in scope at `fs.rs:103`). This restores the outer ceiling and keeps SC-14-07's `patch_issue_times_out_within_budget` regression covered (that test at `fetch.rs:507-543` will be re-homed onto `SimBackend`-via-`fs.rs-helper`; see Q10).

Concrete shape to communicate to the executor:

```rust
async fn update_issue_with_timeout(
    backend: &Arc<dyn IssueBackend>,
    project: &str,
    id: IssueId,
    patch: Untainted<Issue>,
    expected_version: Option<u64>,
) -> Result<Issue, FetchError> {
    match tokio::time::timeout(
        READ_GET_TIMEOUT,
        backend.update_issue(project, id, patch, expected_version),
    ).await {
        Ok(Ok(v)) => Ok(v),
        Ok(Err(e)) => Err(backend_err_to_fetch(e)),
        Err(_) => Err(FetchError::Timeout),
    }
}
```

And `create_issue_with_timeout` with the same shape.

Do **not** push the outer timeout into `SimBackend` — that would mutate the trait's timeout behaviour for every caller (CLI, parity demos, swarm harness), which is out of scope for Phase 14. The `fs.rs`-local helper is the right seam.

---

## Q4 — `X-Reposix-Agent` attribution

### Finding

`SimBackend` **does** inject an `X-Reposix-Agent` header on every request — but the value is `reposix-core-simbackend-<pid>` (see `sim.rs:73-74`), not `reposix-fuse-<pid>` (see `fs.rs:285`) nor `git-remote-reposix-<pid>` (see `main.rs:82`). The trait has no per-call `agent` parameter and the construction-time API only accepts a custom suffix, not a full override.

This is an **observable audit-attribution regression** if unaddressed. After the refactor, every write that currently attributes to `reposix-fuse-12345` or `git-remote-reposix-12345` will instead attribute to `reposix-core-simbackend-12345`. The sim's audit log would lose the caller-role distinction. SG-05 (audit attribution) test evidence: `crates/reposix-fuse/src/fetch.rs:407-421` asserts `header("X-Reposix-Agent", "reposix-fuse-42")`.

There are two in-scope fixes, and one clean design:

- **(a) Use the `with_agent_suffix` API** that `SimBackend::with_agent_suffix(origin, Some(suffix))` already exposes (`sim.rs:70-81`). The emitted header becomes `reposix-core-simbackend-<pid>-<suffix>`. FUSE passes `Some("fuse")`, remote passes `Some("remote")`. This is minimal-change.
- **(b) Add a `SimBackend::with_agent_header` escape hatch** that takes a fully-formed header value rather than a suffix. Larger surface-area change; needs justification.
- **(c) Add a per-call agent parameter to the `IssueBackend` trait.** Breaks LD-14-01 explicitly. Rejected.

### Evidence

- `crates/reposix-core/src/backend/sim.rs:36-40`:
  ```
  pub struct SimBackend {
      http: Arc<HttpClient>,
      origin: String,
      agent_header: String,
  }
  ```
- `crates/reposix-core/src/backend/sim.rs:52-81` — the two construction paths: `new(origin)` yields `reposix-core-simbackend-<pid>`; `with_agent_suffix(origin, Some(s))` yields `reposix-core-simbackend-<pid>-<s>`.
- `crates/reposix-core/src/backend/sim.rs:91-100` — the `agent_only()` / `json_headers()` helpers emit `("X-Reposix-Agent", self.agent_header.as_str())`.
- `crates/reposix-fuse/src/fs.rs:285` — current FUSE agent: `let agent = format!("reposix-fuse-{}", std::process::id());`.
- `crates/reposix-remote/src/main.rs:82` — current remote agent: `let agent = format!("git-remote-reposix-{}", std::process::id());`.
- `crates/reposix-fuse/src/fetch.rs:407-421` — the existing agent-header assertion test.

### Recommendation for planner

Use option (a). The executor should:

1. In `ReposixFs::new` (`fs.rs:272-326`): drop the `agent: String` field and `agent` constructor-local. Change the `SimBackend` construction where it happens (i.e. where the caller builds the `Arc<dyn IssueBackend>` for the FUSE mount — which is `crates/reposix-fuse/src/main.rs`, not `lib.rs::Mount::open`) to pass `Some("fuse")`. FUSE's `ReposixFs` struct no longer needs to know the agent string.
2. In `reposix-remote/src/main.rs`: when constructing the `Arc<SimBackend>` from the `RemoteSpec`, call `SimBackend::with_agent_suffix(spec.origin, Some("remote"))`. Drop the `agent: String` field on `State`.
3. **Update the test at `fetch.rs:407-421` when re-homing** (see Q10). The wiremock matcher will assert `header_exists("X-Reposix-Agent")` rather than `header("X-Reposix-Agent", "reposix-fuse-42")`, because the value now embeds `std::process::id()` which varies. The test at `sim.rs:463-497` (`update_without_expected_version_is_wildcard`) uses a closure matcher to assert a header is absent — the same pattern works for "present, value doesn't matter".

**Document the attribution contract explicitly in the Phase 14 commit message or CHANGELOG:** "FUSE write path now attributes audit rows to `reposix-core-simbackend-<pid>-fuse` (was `reposix-fuse-<pid>`); remote helper to `reposix-core-simbackend-<pid>-remote` (was `git-remote-reposix-<pid>`)." This is a visible behaviour change even though it's a refactor. If the user expects the old labels preserved verbatim, escalate the decision back to them BEFORE executing.

---

## Q5 — `fetch_issues` / `fetch_issue` (read path) in `fetch.rs`

### Finding

**Production code no longer uses them.** `fs.rs::resolve_name` (line 433) and `fs.rs::resolve_ino` (line 456) and `fs.rs::refresh_issues` (line 480) all route through `get_issue_with_timeout` / `list_issues_with_timeout`, which are the `IssueBackend`-trait-driven replacements introduced in Phase 10. The only references to `fetch_issues` / `fetch_issue` that remain are inside the `#[cfg(test)] mod tests` block of `fetch.rs` itself (lines 347-431, 572-595). There are no external importers.

### Evidence

- `crates/reposix-fuse/src/fs.rs:89` — imports only the write halves:
  ```
  use crate::fetch::{patch_issue, post_issue, FetchError};
  ```
- `crates/reposix-fuse/src/fs.rs:442-444, 465-467, 481-483` — read-path production calls go through the `_with_timeout` helpers, which go through the backend trait.
- `grep -rn "fetch_issue\|fetch_issues" crates/reposix-fuse/src/` returns only:
  - Definitions in `fetch.rs:122, 160`.
  - Test calls in `fetch.rs:347-431, 572-595`.
- `crates/reposix-fuse/tests/*.rs` — none of the integration tests (`readdir.rs`, `nested_layout.rs`, `sim_death_no_hang.rs`) import `reposix_fuse::fetch`. Only `tests/write.rs` does (line 17), and only for the write helpers.

### Recommendation for planner

**Delete `fetch_issues` and `fetch_issue` together with the write helpers.** Once `patch_issue` and `post_issue` leave, nothing outside the read-path-in-tests block uses `fetch.rs` at all. Combined with dropping `pub mod fetch;` from `lib.rs:23`, the entire file `crates/reposix-fuse/src/fetch.rs` can be deleted in a single step.

Do **not** attempt an incremental deletion where read helpers survive — that leaves the orphaned `FetchError` enum in a half-used state (only `NotFound`, `Status`, `Transport`, `Origin`, `Parse`, `Core`, `Timeout` variants used; `Conflict`, `BadRequest` dead). Much cleaner to delete the file in one atomic commit and recreate only the pieces `fs.rs` still needs (see Q8 for what to keep).

One minor wrinkle: the test `fetch_issue_origin_rejected` at `fetch.rs:423-431` is the only test that proves a non-allowlisted origin surfaces as `FetchError::Origin(_)`. The equivalent assertion against `SimBackend` already exists implicitly — `SimBackend::new(bad_origin)` doesn't fail, but the first request would. A test like `SimBackend::list_issues` against `"http://evil.example"` producing `Err(Error::InvalidOrigin(_))` could be added to `sim.rs` tests for parity, but `crates/reposix-core/src/http.rs` (the allowlist gate code) already has its own dedicated test coverage. Likely no new test needed; if the planner wants belt-and-suspenders, add one wiremock test in `sim.rs` with a hardcoded `http://evil.example` URL.

---

## Q6 — Remote helper's `api::list_issues` in main.rs

### Finding

The remote helper calls `api::list_issues` in two places and both return `Vec<Issue>` in deserialization order from the sim's JSON array. The sim's SQL `ORDER BY id ASC` (`crates/reposix-sim/src/routes/issues.rs:152`) makes the order deterministic: sorted by `IssueId` ascending. `SimBackend::list_issues` uses the same wire endpoint and feeds the bytes through `serde_json::from_slice::<Vec<Issue>>` (`sim.rs:131`), so the order is preserved.

The `diff::plan` function (`crates/reposix-remote/src/diff.rs:99-204`) does **not** depend on input order for correctness — it builds two lookup maps (`prior_by_id: HashMap<IssueId, &Issue>` at line 100 and `prior_by_path: BTreeMap<String, IssueId>` at line 101) and iterates those. The output `Vec<PlannedAction>` is `creates.extend(...)`, `updates.extend(...)`, `deletes.extend(...)`, where each of those three sub-vecs is appended by iteration-order over `parsed.tree` (which is itself a `BTreeMap<String, u64>`, so alphabetically sorted by path). So plan ordering is independent of the `prior: &[Issue]` argument's order.

### Evidence

- `crates/reposix-remote/src/main.rs:181-196` — import path calls `api::list_issues`.
- `crates/reposix-remote/src/main.rs:228-244` — export path calls `api::list_issues` a second time to compute prior state.
- `crates/reposix-sim/src/routes/issues.rs:141-162` — `async fn list_issues` uses `ORDER BY id ASC`.
- `crates/reposix-core/src/backend/sim.rs:224-231` — `SimBackend::list_issues` preserves the bytes verbatim:
  ```
  async fn list_issues(&self, project: &str) -> Result<Vec<Issue>> {
      let url = format!("{}/projects/{}/issues", self.base(), project);
      let resp = self
          .http
          .request_with_headers(Method::GET, &url, &self.agent_only())
          .await?;
      decode_issues(resp, &url).await
  }
  ```
- `crates/reposix-remote/src/diff.rs:99-204` — the `plan` function; see in particular the `HashMap::from_iter(prior.iter()...)` build and the `BTreeMap` for paths.

### Recommendation for planner

Swap both call sites directly:

```
state.rt.block_on(api::list_issues(&state.http, &state.origin, &state.project, &state.agent))
```
→
```
state.rt.block_on(state.backend.list_issues(&state.project))
```

Where `state.backend: Arc<SimBackend>` (or `Arc<dyn IssueBackend>` — either is fine, dyn-compat is proved; see `backend.rs:214-215`). The `state.http`, `state.origin`, `state.agent` fields become redundant (agent handling per Q4).

The `fail_push` error-path invocation can continue using `anyhow::Error` as its diagnostic type — `reposix_core::Error` is `std::error::Error` and feeds the `format!("{e:#}")` formatter at `main.rs:193, 240` correctly.

One caveat: the existing `handle_export` path discards the `e` from `api::list_issues` with a generic `"backend-unreachable"` tag on the wire. The error text on stderr is preserved. With `SimBackend::list_issues`, a 5xx from the sim comes back as `Error::Other("sim returned 500 ... for GET .../issues: kaboom")` — the `"kaboom"` substring becomes visible. Existing test `crates/reposix-remote/tests/protocol.rs:178-237` asserts stderr contains `"cannot list prior issues"` OR `"backend"` — both still hold after the swap because `main.rs:240` still `format!("cannot list prior issues: {e:#}")`s the error. Green, no test change.

---

## Q7 — Delete mapping

### Finding

The remote helper's `execute_action` at `main.rs:344-356` issues a plain DELETE via `api::delete_issue` with no reason argument. The `IssueBackend::delete_or_close(project, id, reason: DeleteReason)` signature requires a `DeleteReason` variant. `SimBackend::delete_or_close` **ignores `reason` entirely** (see the `_reason: DeleteReason` underscore-prefixed binding at `sim.rs:287` and the comment at lines 288-292), so semantically any variant passed by the caller produces identical sim behaviour.

Among the four existing `DeleteReason` variants (`Completed`, `NotPlanned`, `Duplicate`, `Abandoned` at `backend.rs:84-95`), `Abandoned` is the closest semantic match for "git tree removed this file; no editorial reason surfaces through the git protocol." The variant doc-string reads: *"Generic abandonment, no specific reason. Reserved for compatibility with trackers that don't carry a reason field."*

A future Confluence or GitHub backend would translate `Abandoned` into its respective close-with-reason shape (e.g. GitHub `state_reason: not_planned`). But the remote helper's git-protocol caller cannot tell the difference between "delete because completed" and "delete because not planned" — the git delete is an editorial void. `Abandoned` is the only honest mapping.

### Evidence

- `crates/reposix-remote/src/main.rs:344-356`:
  ```
  PlannedAction::Delete { id, .. } => {
      state
          .rt
          .block_on(api::delete_issue(
              &state.http,
              &state.origin,
              &state.project,
              id,
              &state.agent,
          ))
          .with_context(|| format!("delete issue {}", id.0))?;
      Ok(())
  }
  ```
- `crates/reposix-core/src/backend.rs:82-95` — the four variants.
- `crates/reposix-core/src/backend/sim.rs:283-310`:
  ```
  async fn delete_or_close(
      &self,
      project: &str,
      id: IssueId,
      _reason: DeleteReason,
  ) -> Result<()> {
      // The sim performs a real DELETE regardless of reason — the reason
      // is meaningful only to backends (GitHub) that close with
      // state_reason. We preserve the argument in the signature so callers
      // can write backend-agnostic code.
      let url = format!("{}/projects/{}/issues/{}", self.base(), project, id.0);
      ...
  }
  ```

### Recommendation for planner

Use `DeleteReason::Abandoned`. **Do not** invent a new `DeleteReason::Unspecified` variant — the trait's `DeleteReason` enum is `#[non_exhaustive]` (`backend.rs:84`), so adding a variant is non-breaking, but it's unnecessary churn and slightly worse semantics (it implies "no reason" as a deliberate value; `Abandoned` already fills that slot).

The call becomes:

```rust
state.rt.block_on(state.backend.delete_or_close(
    &state.project,
    id,
    DeleteReason::Abandoned,
))
```

Add `use reposix_core::backend::DeleteReason;` to `main.rs`. No test change required — the bulk-delete-cap tests (`crates/reposix-remote/tests/bulk_delete_cap.rs`) assert on wire-level DELETE count, which is unchanged.

---

## Q8 — Thrown-away code inventory

### Finding

Comprehensive inventory. Every symbol below becomes dead on the post-refactor tree and must be deleted in the same commits (LD-14-07 forbids `#[allow(dead_code)]` survival).

#### `crates/reposix-fuse/src/fetch.rs` — delete the entire file

| Symbol | Location | Notes |
|--------|----------|-------|
| `FETCH_TIMEOUT` | line 27 | `fs.rs` gets its own via `READ_GET_TIMEOUT`/`READ_LIST_TIMEOUT` already in scope |
| `FetchError` enum | lines 32-70 | Move to `fs.rs` as a private enum — only the FUSE callback still needs a sum of `NotFound`/`Origin`/`Timeout`/`Transport`/`Status`/`Parse`/`Core`/`Conflict`. Drop `BadRequest` per Q2. |
| `FetchError::Conflict { current }` | lines 58-65 | **Keep variant**, move to `fs.rs` — still used by release's diagnostic log |
| `FetchError::BadRequest(String)` | lines 66-69 | **Delete** — no caller produces this after refactor (per Q2) |
| `EgressPayload<'a>` | lines 77-98 | **Delete** — `SimBackend::render_patch_body` / `render_create_body` (sim.rs:140-198) replace it |
| `ConflictBody` | lines 103-106 | **Move to `fs.rs`** private, used by the new `backend_err_to_fetch` JSON re-parse (Q1) |
| `from_core` | lines 108-114 | **Delete** — `backend_err_to_fetch` in `fs.rs:108-116` supersedes |
| `fetch_issues` | lines 122-151 | **Delete** — orphaned (Q5) |
| `fetch_issue` | lines 160-194 | **Delete** — orphaned (Q5) |
| `patch_issue` | lines 208-263 | **Delete** — superseded by `update_issue_with_timeout` (Q3) |
| `post_issue` | lines 274-318 | **Delete** — superseded by `create_issue_with_timeout` (Q3) |
| `#[cfg(test)] mod tests` | lines 320-596 | **Delete, re-home to `sim.rs` per Q10** |

Net effect: `fetch.rs` drops from 596 lines to zero. Delete the file. Drop `pub mod fetch;` from `crates/reposix-fuse/src/lib.rs:23`. Update the module doc `lib.rs:9-10` and `lib.rs:38-43` (the `MountConfig::origin` comment) to remove the "still speaks the simulator REST shape via fetch" language.

Also delete:

- `crates/reposix-fuse/tests/write.rs` — the only external consumer (Q10 re-homes its assertions to `sim.rs` where overlap exists, and to a new `fs.rs` integration test where not).

#### `crates/reposix-remote/src/client.rs` — delete the entire file

| Symbol | Location | Notes |
|--------|----------|-------|
| `REQ_TIMEOUT` | line 17 | Analogous to `FETCH_TIMEOUT`; replaced by `HttpClient` inner timeout + any outer wrapper the remote path wants (likely none needed — the remote helper is a single-shot process, unlike FUSE's kernel callback) |
| `ClientError` enum | lines 20-43 | **Delete** — `anyhow::Error` flowing from `reposix_core::Error` replaces. `fail_push` already takes a `&str` detail; `format!("{e:#}", e = core_err)` fills it. |
| `from_core` | lines 45-51 | **Delete** |
| `EgressPayload<'a>` | lines 56-77 | **Delete** |
| `ConflictBody` | lines 80-83 | **Delete** — remote helper never inspects the `current` integer; the `.context("patch issue {id}")` wrap in `execute_action` (line 341) suffices for the git-protocol error-line |
| `issue_url` | lines 85-92 | **Delete** |
| `list_url` | lines 94-100 | **Delete** |
| `list_issues` | lines 106-128 | **Delete** |
| `patch_issue` | lines 135-175 | **Delete** |
| `post_issue` | lines 181-209 | **Delete** |
| `delete_issue` | lines 215-236 | **Delete** |

Net effect: `client.rs` drops from 236 lines to zero. Delete the file. Remove `mod client;` from `crates/reposix-remote/src/main.rs:22` and `use crate::client as api;` from line 27. Drop `crate::client` references anywhere else (none expected; `grep` to confirm).

#### `crates/reposix-remote/Cargo.toml` — review dep list

| Dep | Still needed? |
|-----|--------------|
| `thiserror` (line 24) | **Drop** — only `ClientError` used it in this crate |
| `serde` (line 17) | **Keep** — still needed by `fast_import` and `diff` |
| `serde_json` (line 18) | **Keep** — still used |
| `serde_yaml` (line 19) | **Keep** — frontmatter |
| `reqwest` (line 16) | **Drop** iff no other remote-helper code builds a `reqwest::Method` etc. directly after the refactor. Check: `main.rs` currently imports `reposix_core::http::{client, ClientOpts, HttpClient}`. Those all come from `reposix-core`, which transitively depends on `reqwest`. After refactor, the remote helper no longer builds `HttpClient` itself — the `SimBackend::new` path does that internally. So the explicit `reposix-remote -> reqwest` dependency becomes unused. **Drop.** |

`anyhow`, `tokio`, `clap`, `tracing`, `tracing-subscriber` stay.

### Evidence

- All symbol locations cited above are verified against the respective files (`fetch.rs`, `client.rs`, `main.rs`, `Cargo.toml`) as of the current commit.

### Recommendation for planner

**Phrase this to the executor as "delete, don't deprecate."** Two sweep commits are the minimum:

1. Wave B1 commit (FUSE): rewrite `fs.rs:89` imports, `fs.rs::release`, `fs.rs::create`, `fs.rs::backend_err_to_fetch`; define `fs.rs`-local `FetchError` sum type (possibly renamed to `FsError` for clarity — but the rename is optional, not required by any SC) and `ConflictBody`; add `update_issue_with_timeout` and `create_issue_with_timeout` helpers. Delete `src/fetch.rs`. Delete `tests/write.rs`. Remove `pub mod fetch;` from `lib.rs:23`. Update `lib.rs` module doc and `fs.rs` module doc (lines 42-43, 201-204) to drop "write path still speaks sim REST shape" language.

2. Wave B2 commit (remote): rewrite `main.rs::execute_action` and the two `api::list_issues` sites; construct `Arc<SimBackend>` from `RemoteSpec` once; drop the `http`/`agent` fields from `State`. Delete `src/client.rs`. Remove `mod client;` and `use crate::client as api;`. Prune `thiserror` and `reqwest` from `Cargo.toml`.

Do **not** gate the sweep behind a flag or leave dead code behind a `#[allow(dead_code)]` — LD-14-07 explicitly prohibits that grace period.

---

## Q9 — Threat-model sanity

### Finding

**No new egress, no allowlist widening.** The refactor routes the same PATCH/POST/DELETE URLs (`{origin}/projects/{project}/issues[/{id}]`) through the same sealed `HttpClient` with the same allowlist. The only observable wire changes are:

- `If-Match` header value: `"3"` becomes `"\"3\""` (quoted etag) — see tl;dr #3. The sim accepts both; this is not a widening.
- `X-Reposix-Agent` header value: `reposix-fuse-<pid>` becomes `reposix-core-simbackend-<pid>-fuse` (Q4). Same header, different string. Not a widening.
- Body shape: `EgressPayload` (five fields) becomes `render_patch_body`/`render_create_body` output (same five fields, but via a `serde_json::Map<String, Value>` rather than a dedicated struct — see `sim.rs:140-198`). The emitted JSON key set is identical. `assignee: None` emission differs in one case: `EgressPayload::from_untainted` skips it entirely (`#[serde(skip_serializing_if = "Option::is_none")]` at `fetch.rs:81-82`); `render_patch_body` emits `"assignee": null` (sim.rs:151-154). For PATCH this is a **meaningful difference** — the sim's `FieldUpdate<String>` deserializer treats absent as `Unchanged` and `null` as `Clear` (sim/routes/issues.rs:289-298). So the new-style body with `"assignee": null` explicitly clears the assignee on every PATCH; the old-style body with `assignee` absent left it untouched. **This is a behaviour change outside the refactor's no-behaviour-change envelope.** Flag to the planner.

For POST, `render_create_body` skips `assignee` when `None` (`sim.rs:184-186`), matching the old skip-if-none. So create is unaffected.

All other wire bytes are identical.

### Evidence

- `crates/reposix-core/src/backend/sim.rs:71` — `HttpClient` constructed with `ClientOpts::default()`; allowlist gate is mandatory per `http.rs:245-293`.
- `crates/reposix-fuse/src/fetch.rs:77-98` vs `crates/reposix-core/src/backend/sim.rs:140-198` — field-set comparison.
- `crates/reposix-sim/src/routes/issues.rs:289-298` — `FieldUpdate` three-valued deserializer.
- `crates/reposix-fuse/src/fs.rs:1012-1027` — the `release` callback parses the user's file content via `frontmatter::parse(&text)` and produces a `parsed: Issue` whose `assignee` field is whatever the user wrote. So if the user's file has no `assignee:` line, the old wire body omitted the key (untouched); the new wire body will clear the assignee. If the file has `assignee: alice`, both old and new emit `"assignee":"alice"`. If the file has `assignee: null` or the key is missing entirely, the effective server result differs.

### Recommendation for planner

**Escalate to the user BEFORE writing code.** The `assignee`-clear-on-untouched behaviour change is a genuine semantic drift, not a pure refactor. Three options:

- **(A) Accept it.** Plausible: the FUSE mount is designed for an agent rewriting the whole frontmatter on every edit; the "preserve assignee across partial edits" use-case is rare, and the new behaviour is more honest (what the user wrote is what the server gets).
- **(B) Teach `render_patch_body` to three-value the assignee.** Requires a small `SimBackend` change to accept a `patch: Untainted<Issue>` + a per-field `dirty` mask, or to read `None` as "unchanged" unconditionally (i.e. drop the `"assignee": null` branch). The latter is safer but loses the ability to clear an assignee via PATCH at all. Non-trivial.
- **(C) Keep the old semantic by synthesising a `prior` check** in `fs.rs::release`: if the cached issue's assignee equals the parsed issue's assignee, pass `Untainted<Issue>` with assignee stripped somehow. Ugly, would require a new `update_issue` variant.

Note: the `Untainted<Issue>` type does not model field-presence; it models "the server-controlled fields have been stripped." There is currently no way to express "don't touch this field" except by not including it in the wire body. Option (B) either needs a new type or a "None-means-untouched" sim-backend convention.

**My recommended escalation:** present options A/B/C to the user with the sim's current `FieldUpdate<>` three-valued semantics called out, let them choose. The simplest viable path is (A) — it's the honest interpretation of "the file is the source of truth" that the FUSE design is built around. But the user should know.

If the user hasn't been flagged about this before Phase 14 execution starts, the executor (Wave B1) will hit it in tests. Better to spend 5 minutes on a discussion turn now than debug it during B1.

---

## Q10 — Tests to re-home

### Finding

Inventory of all tests exercising the write path outside the `SimBackend` unit tests:

#### `crates/reposix-fuse/src/fetch.rs` `#[cfg(test)] mod tests` (lines 320-596)

| Test | Line | Category | Re-home decision |
|------|------|----------|------------------|
| `fetch_issues_parses_list` | 347 | Read path (orphaned) | Delete — `SimBackend::list_issues` already tested at `sim.rs:363-382` |
| `fetch_issue_parses_one` | 364 | Read path (orphaned) | Delete — covered by `sim.rs:384-397` |
| `fetch_issue_404_is_not_found` | 379 | Read path | Delete — covered by `sim.rs:399-417` |
| `fetch_issue_500_is_status` | 393 | Read path | Delete — no direct sim equivalent; add a new `sim.rs` test asserting 500 → `Error::Other(msg.contains("sim returned 500"))`. |
| `fetch_issues_attaches_agent_header` | 407 | Read path (SG-05) | Re-home to `sim.rs` with a `header_exists`-style matcher (value is process-specific per Q4) |
| `fetch_issue_origin_rejected` | 424 | Read path (SG-01) | Re-home (or rely on `http.rs` own allowlist tests — see Q5) |
| `patch_issue_sends_if_match_header` | 434 | **Write** | Re-home to `sim.rs`. Assert `If-Match: "3"` (quoted, not `"3"`). Use `backend.update_issue(..., Some(3)).await`. An existing test at `sim.rs:420-462` already covers this shape with `.header("If-Match", "\"5\"")`; the re-home can mimic verbatim. |
| `patch_issue_409_returns_conflict` | 468 | **Write** | Re-home to `sim.rs`. Call `backend.update_issue` with a mismatching version, assert `Err(Error::Other(msg)) if msg.starts_with("version mismatch:") && msg.contains("\"current\":7")`. Also add a test of `backend_err_to_fetch` in `fs.rs` to prove the string→`FetchError::Conflict { current: 7 }` conversion (Q1). |
| `patch_issue_times_out_within_budget` | 507 | **Write** | Re-home to `fs.rs::update_issue_with_timeout` (the Q3 helper); assert the wrapper returns `FetchError::Timeout` within 5.5s. The underlying `SimBackend::update_issue` without a wrapper would still time out via reqwest's 5s total-timeout as a floor, but the assertion is cleaner against the FS helper. |
| `post_issue_sends_egress_shape_only` | 546 | **Write** | Re-home to `sim.rs`. Assert POST body contains `"title"` and lacks `"version"`/`"id"`/`"created_at"`/`"updated_at"`. |
| `fetch_issue_times_out_within_budget` | 572 | Read path | Re-home to `fs.rs::get_issue_with_timeout` (already exists). May already be covered by the existing Phase 10 test corpus — verify. |

#### `crates/reposix-fuse/tests/write.rs` (lines 37-236)

| Test | Line | Decision |
|------|------|----------|
| `release_patches_with_if_match` | 38 | Near-duplicate of `patch_issue_sends_if_match_header`. Re-home to `sim.rs` (or delete as redundant; the sim test already covers it). |
| `release_409_returns_conflict` | 77 | Duplicate of `patch_issue_409_returns_conflict` but with slightly different response body (no `sent` field). Re-home, or delete as redundant. |
| `release_timeout_returns_eio_flavored_error` | 113 | Same as `patch_issue_times_out_within_budget`. Re-home to `fs.rs::update_issue_with_timeout` or delete. |
| `sanitize_strips_server_fields_on_egress` | 151 | **This is SG-03 coverage, critical.** Re-home to `sim.rs`. Assert that `SimBackend::update_issue(..., sanitize(Tainted::new(hostile)), Some(v))` produces a wire body without `"version"`/`"id"`/`"created_at"`/`"updated_at"`. **The test must continue to fire.** This is a proof that the `Untainted<Issue>` discipline holds at the trait-impl layer too. |
| `create_posts_and_returns_issue` | 215 | Covered by `post_issue_sends_egress_shape_only`. Delete if redundant, or re-home as simple parity test. |

#### `crates/reposix-remote/src/client.rs` — no `#[cfg(test)]` tests

`client.rs` has zero inline tests. All remote-helper test coverage lives in `crates/reposix-remote/tests/protocol.rs` and `tests/bulk_delete_cap.rs`, which drive the **compiled binary** end-to-end. These will transparently continue to work because they assert on stdout/stderr protocol lines and on the wiremock request counts — not on any code-path identity inside the helper.

### Evidence

- All test locations cited above are verified line-numbers from `rg` on the current tree.
- `SimBackend` already has strong wiremock scaffolding: `sim.rs:313-514` contains `list_builds_the_right_url`, `get_builds_the_right_url`, `get_maps_404_to_not_found`, `update_with_expected_version_attaches_if_match`, `update_without_expected_version_is_wildcard`, `supports_reports_full_matrix_for_sim`. Re-homing slots into this file naturally.
- The custom `wiremock::Match` pattern for "header absent" is at `sim.rs:471-477`; the mirror for "header present any value" is trivial (`request.headers.contains_key("x-reposix-agent")`).

### Recommendation for planner

**Enforce LD-14-08 (test count ≥ 272) explicitly.** The clean re-homing maps:

- Fetch read-path tests (6 tests) → delete after verifying `sim.rs` and `http.rs` coverage is equivalent.
- Fetch write-path tests (4 tests) → re-home: 3 into `sim.rs` + 1 into `fs.rs::update_issue_with_timeout`.
- `tests/write.rs` (5 tests) → re-home 1 critical SG-03 proof into `sim.rs`; the other 4 are redundancies (delete, note in commit message).

Net: -15 tests in the FUSE crate, +5-7 tests in the core `sim.rs` mod. If total count would drop below 272, add:

- A new `sim.rs` test for POST 4xx → `Error::Other("sim returned 400")` shape (Q2 recommendation).
- A new `fs.rs` test for `backend_err_to_fetch` on a version-mismatch `Error::Other` surfaces as `FetchError::Conflict { current }` (Q1 fix proof).
- Optionally a new `fs.rs` test for `create_issue_with_timeout` timeout (symmetric to the existing `list_issues_with_timeout` test, if one exists).

Carefully, if this ends up below 272, call it out in the phase's final report rather than forging test count. LD-14-08 allows genuine obsolescence as long as it's documented in the commit message.

---

## Risks and unknowns

Items the planner must **decide**, not just describe.

### R1 — Assignee-clear regression on untouched PATCH (Q9)

**Hard decision.** Without handling, every PATCH from FUSE's `release` callback will now clear the assignee unless the user's file explicitly carries `assignee: <someone>`. Today's `EgressPayload` skips-if-none, today's `SimBackend::render_patch_body` emits explicit `null`. The sim distinguishes absent vs. null in `FieldUpdate<>`.

**Recommended resolution:** escalate to the user before Wave B1. If the user picks option (A) "accept it", document in CHANGELOG under `### Changed`. If they pick option (B), expect +30 minutes on Wave B1 for the sim-backend tweak. If option (C), expect more.

### R2 — X-Reposix-Agent attribution change (Q4)

**Visible behaviour change** in the audit log. Sim rows currently tagged `reposix-fuse-*` will be tagged `reposix-core-simbackend-*-fuse`. This is technically LD-14-04-compliant (no new error variant) but may surprise operators running `SELECT agent, COUNT(*) FROM audit GROUP BY agent`.

**Recommended resolution:** call it out in CHANGELOG. No code option to preserve the old labels without adding a `with_agent_header(full_value)` escape hatch — and that hatch would risk the attribution contract being spoofed by any future caller of `SimBackend::new`. Stick with the suffix approach.

### R3 — If-Match quoting change on the wire (tl;dr #3, Q10)

The sim already accepts both. No functional risk. **But** if `grep -r "If-Match" /home/reuben/workspace/reposix/` turns up any asserted expectation of the unquoted form outside `fetch.rs`'s own tests (e.g. a third-party client, a documented wire contract, a wiremock in an archived fixture), that expectation fails. I grepped and found only:

- `crates/reposix-fuse/src/fetch.rs:226` (the code we're deleting)
- `crates/reposix-fuse/src/fetch.rs:438` and `tests/write.rs:42` (tests we're re-homing)
- `crates/reposix-remote/src/client.rs:148` (code we're deleting)
- `crates/reposix-sim/src/routes/issues.rs:126-133` (the sim's quote-stripping)
- `crates/reposix-sim/src/routes/issues.rs:574, 596` (sim tests asserting `"\"1\""` and `"\"bogus\""` — already quoted)
- `crates/reposix-core/src/backend/sim.rs:263, 266, 425` (SimBackend uses quoted)

No unaccounted-for callers. **Low risk.**

### R4 — `post_issue_sends_egress_shape_only` coverage gap for 4xx POST

Neither the existing `fetch.rs` test suite nor the `tests/write.rs` suite exercises a 4xx response on POST. The `FetchError::BadRequest` variant has always been dead-lettered — its only live asserting caller is the error-map function itself. So the "loss" from Q2 (BadRequest variant deleted) is a loss of potential future test coverage, not of existing coverage. **Not blocking.**

### R5 — The remote helper's fail-push error-line format

`fail_push` writes `error refs/heads/main {kind}` where `kind` is a stable string constant (`"backend-unreachable"`, `"parse-error"`, `"bulk-delete"`, `"invalid-blob:<path>"`, `"some-actions-failed"`). After the refactor, `execute_action` errors (create/update/delete) bubble up through `anyhow::Error` and are collected into `any_failure = true`, which flips the `"some-actions-failed"` kind. That's unchanged. **But** SC-14-09's prose says "`error refs/heads/main version-mismatch`" — no such kind is emitted today (the remote helper doesn't distinguish version-mismatch from any other execute failure on the wire). The existing wire string is literally `error refs/heads/main some-actions-failed`. Either the SC prose is aspirational, or someone mis-remembered the wire string. I recommend the planner treat the SC as "conflict STILL surfaces somehow as a push failure," which it does via the `some-actions-failed` kind + stderr diagnostic. **If the user genuinely wants a `version-mismatch` wire kind, that's a scope addition — flag it back.**

### R6 — Deleting `fetch_issue_origin_rejected` coverage without replacement

The allowlist gate has its own tests in `crates/reposix-core/src/http.rs`, and I didn't exhaustively verify those cover the `SimBackend::*` path surfaces `Error::InvalidOrigin(_)`. If they don't, we lose an SG-01 regression guard when `fetch_issue_origin_rejected` is deleted. **Cheap fix:** add a one-liner test in `sim.rs` that constructs `SimBackend::new("http://evil.example".into())` and asserts the first `list_issues` call returns `Err(Error::InvalidOrigin(_))`. I marked this in Q5's recommendation.

### R7 — Read-path tests were already flowing through `SimBackend`, but I did not verify the Phase-10 conversion is complete

I verified `fs.rs::resolve_name`, `resolve_ino`, `refresh_issues` use `_with_timeout` helpers (Q5). I did NOT verify every read-path call site in `fs.rs` — e.g., there are readdir/lookup/read callbacks I didn't read in full. **Non-blocking for Phase 14**, because Phase 14's scope is write-path; but if a planner assumes "Phase 10 completed the read-path rewire," they should sanity-check against `grep -n fetch:: fs.rs` which returns zero after our refactor. If any read callback still imports `fetch::*` post-refactor, the build breaks at that line and we catch it deterministically.

### R8 — No verification that `green-gauntlet.sh --full` actually runs the `#[ignore]`-gated tests

SC-14-07 asserts `bash scripts/green-gauntlet.sh --full` green, which the context says includes FUSE `--ignored` integration tests. I did not read `scripts/green-gauntlet.sh`. **Potential risk:** if the gauntlet doesn't exercise the write path end-to-end (mount → write → PATCH), then "green gauntlet" doesn't prove SC-14-01/02/08. **Recommended:** the planner reads `scripts/green-gauntlet.sh` before signing off on the wave structure. Wave C's "30-min gauntlet run" should include a concrete file write + sim-side verification.

### R9 — `SimBackend::delete_or_close` 404 body text re-renders differently than the old DELETE path

Old `client::delete_issue` on 404 returns `ClientError::Status(404, body)` → remote helper's `.with_context(|| format!("delete issue {id}"))?` → surfaces in stderr as `"delete issue N: backend status 404: <body>"`. New `SimBackend::delete_or_close` on 404 (sim.rs:299-301) returns `Error::Other(format!("not found: {url}"))` → stderr as `"delete issue N: not found: http://.../projects/demo/issues/N"`. Different prose, same errno semantics. The stderr string is not asserted by any existing test I found. **Low risk.**

### R10 — The `Match` trait in wiremock may have slightly different import paths across wiremock versions

The re-homed test at `sim.rs:471-477` uses `use wiremock::Match;`. If the executor is on a newer wiremock, this might need a refactor. **Non-blocking**, but worth noting if CI complains. Current `Cargo.toml`: `wiremock = "0.6"` — matches the `sim.rs` import path. Fine.

---

## Source citations map

Every claim in this research is backed by the specific line(s) from these files:

- `crates/reposix-fuse/src/fetch.rs` (596 lines, full read)
- `crates/reposix-fuse/src/fs.rs:1-260, 418-542, 980-1172` (read path, backend-err mapping, write callbacks)
- `crates/reposix-fuse/src/lib.rs` (full read; `pub mod fetch;` at line 23)
- `crates/reposix-fuse/Cargo.toml` (full read; dev-deps at 43-46)
- `crates/reposix-fuse/tests/write.rs` (full read)
- `crates/reposix-remote/src/main.rs` (full read)
- `crates/reposix-remote/src/client.rs` (full read)
- `crates/reposix-remote/src/diff.rs` (full read; `plan` function 99-204)
- `crates/reposix-remote/tests/protocol.rs` (full read)
- `crates/reposix-remote/tests/bulk_delete_cap.rs` (partial; delete-cap tests 52-144)
- `crates/reposix-remote/Cargo.toml` (full read)
- `crates/reposix-core/src/backend.rs` (full read; trait definition + DeleteReason + BackendFeature)
- `crates/reposix-core/src/backend/sim.rs` (full read, 515 lines)
- `crates/reposix-core/src/error.rs` (full read, 51 lines)
- `crates/reposix-core/src/http.rs:1-326` (ClientOpts + HttpClient newtype + allowlist gate)
- `crates/reposix-core/src/remote.rs` (full read; `parse_remote_url` + `RemoteSpec`)
- `crates/reposix-sim/src/error.rs` (full read; `ApiError::VersionMismatch` body shape 83-91)
- `crates/reposix-sim/src/routes/issues.rs:1-456, 550-607` (handlers + body-shape assertions)

No external documentation was consulted; every finding reflects source code in this repository as of 2026-04-14.
