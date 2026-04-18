# Phase 8: Demo suite + real-backend integration — Context

**Gathered:** 2026-04-13 09:05 PDT
**Deadline:** 12:15 PDT (~3h 10min budget)
**Status:** Post-ship value-additive phase. v0.1.0 already tagged.

<domain>
## Phase Boundary

This phase does **two substantive things**:

1. **Restructure the demo monolith into a suite** — Tier 1 focused one-liners + Tier 2 full walkthrough + Tier 3 backend parity.
2. **Land the seam for real-backend integration** — `IssueBackend` trait + two implementations (SimBackend, GithubReadOnlyBackend) + contract test proving shape parity.

**In scope:**
- `scripts/demos/_lib.sh` — shared setup/teardown/section helpers.
- `scripts/demos/01-edit-and-push.sh`, `02-guardrails.sh`, `03-conflict-resolution.sh`, `04-token-economy.sh` — focused Tier 1 demos, each ≤60s wall clock.
- `scripts/demos/full.sh` — current `scripts/demo.sh` moved here (unchanged substance). Old `scripts/demo.sh` becomes a shim that invokes `full.sh` for backwards compat.
- `scripts/demos/assert.sh` — wrap any demo, capture output, grep ASSERTS markers from header comment. Makes demos into self-asserting integration tests.
- `scripts/demos/smoke.sh` — runs Tier 1 via `assert.sh`, single exit code. CI calls this.
- `docs/demos/index.md` — when-to-use-which table, audience mapping, link to recordings.
- `crates/reposix-core/src/backend.rs` — new module exposing `trait IssueBackend` (async) with 6-8 methods: `list_issues`, `get_issue`, `create_issue`, `update_issue`, `delete_or_close`, `supports` (feature matrix query).
- `crates/reposix-core/src/backend/sim.rs` — `SimBackend` struct wrapping an `HttpClient + origin + project`. Implements the trait by making the existing HTTP calls to the simulator.
- `crates/reposix-github` (new crate) — `GithubReadOnlyBackend` implementing the trait. Read-only for v0.1.5: `list_issues` and `get_issue` only; write methods return `Error::NotSupported`. Uses `gh auth token` via subprocess OR `GITHUB_TOKEN` env var. Honors rate-limit headers. Handles Link-header pagination.
- `crates/reposix-github/tests/contract.rs` — **same** test function run twice, parameterized over `(SimBackend, GithubReadOnlyBackend)`. Proves normalized-shape parity for `list`, `get`, `get_nonexistent`. `#[ignore]`-gated on the GitHub run so CI without auth still passes unit tier.
- `scripts/demos/parity.sh` — CLI demo that invokes both backends in sequence and diffs their normalized output. Tier 3.
- `docs/decisions/001-github-state-mapping.md` — ADR for how Jira-flavored statuses map to GitHub open/closed + state_reason + labels. Single source of truth for the adapter.
- `docs/demos/` subfolder with regenerated recordings per Tier 1 demo.

**Out of scope:**
- Write path against real GitHub (v0.2).
- Jira or Linear adapters (v0.2+).
- TTY-confirmation on `git remote add reposix::...` (tracked for v0.2).
- Rewriting the simulator to match GitHub quirks — the trait IS the shape; sim stays how it is.
- Swarm harness (Phase 6 redux — deferred again, not on this phase's critical path).

</domain>

<decisions>
## Implementation Decisions

### `trait IssueBackend` (the seam)

Lives at `crates/reposix-core/src/backend.rs`. All async via `#[async_trait]` (add `async-trait = "0.1"` to workspace deps). Methods:

```rust
#[async_trait::async_trait]
pub trait IssueBackend: Send + Sync {
    /// A stable, human-readable name for this backend (e.g. "simulator", "github").
    fn name(&self) -> &'static str;

    /// Feature matrix.
    fn supports(&self, feature: BackendFeature) -> bool;

    async fn list_issues(&self, project: &str) -> Result<Vec<Issue>>;
    async fn get_issue(&self, project: &str, id: IssueId) -> Result<Issue>;
    async fn create_issue(&self, project: &str, issue: Untainted<Issue>) -> Result<Issue>;
    async fn update_issue(
        &self,
        project: &str,
        id: IssueId,
        patch: Untainted<Issue>,
        expected_version: Option<u64>,
    ) -> Result<Issue>;
    async fn delete_or_close(
        &self,
        project: &str,
        id: IssueId,
        reason: DeleteReason,
    ) -> Result<()>;
}

pub enum BackendFeature { Delete, Transitions, StrongVersioning, BulkEdit, Workflows }
pub enum DeleteReason { Completed, NotPlanned, Duplicate, Abandoned }
```

- `list_issues`/`get_issue` are read path — only two methods `GithubReadOnlyBackend` must actually implement for v0.1.5.
- `create`/`update`/`delete_or_close` return `Error::NotSupported` on read-only backends.
- `expected_version: Option<u64>` gives a backend-specific way to express optimistic concurrency: simulator maps to `If-Match: "<v>"`, GitHub v0.2 would map to `If-Unmodified-Since`.
- `DeleteReason` lets adapters translate: simulator does real DELETE; GitHub does close with `state_reason`.

### `SimBackend` implementation

In `crates/reposix-core/src/backend/sim.rs`. Wraps `HttpClient` + `origin: String`. Every method is ~10 lines of glue: build URL, call HTTP, deserialize. The existing methods in `reposix-sim`'s route handlers give us the wire shapes; we just call them through `HttpClient`.

### `GithubReadOnlyBackend` implementation

New crate `crates/reposix-github`. Dependencies: `reposix-core`, `reqwest`, `tokio`, `serde`, `async_trait`. No `gh` CLI at library level — takes a `Credentials { token: Option<String> }`; if `token` is `None`, works unauthenticated (60 req/hr limit but fine for tests).

Translation:
- `list_issues(project)` → `GET /repos/{project}/issues?state=all&per_page=100` (project = `owner/repo`). Follow `Link: <...>; rel="next"` pagination. State `open` → `IssueStatus::Open`; `closed` + `state_reason=completed` → `Done`; `closed` + `state_reason=not_planned` → `WontFix`; unknown `closed` → `Done`.
- `get_issue(project, id)` → `GET /repos/{project}/issues/{id}` (where id = `number`).
- Label conventions: labels starting with `status/` are parsed into `IssueStatus` overrides (e.g. `status/in-progress`). Unknown `status/*` labels fall through to the best GitHub state mapping.
- Credentials resolution priority (callers pick): explicit arg → `GITHUB_TOKEN` env → unauthenticated.
- Honors `X-RateLimit-Remaining`; if below 10, logs a WARN and continues.

### Contract test

`crates/reposix-github/tests/contract.rs` — 5 assertions that hold for any well-behaved `IssueBackend`:

1. `list_issues` returns `Ok(vec)` for a known-good project.
2. `list_issues` returns non-empty for a project with issues.
3. `get_issue(project, issue.id)` for the first listed issue returns `Ok(issue)` with matching `id` and `title`.
4. `get_issue` for id `u64::MAX` returns `Err(NotFound)`.
5. Every listed issue has a valid `IssueStatus` (i.e. the adapter didn't leave a raw string).

Two test functions: `contract_sim` runs against a locally-spawned simulator (always in CI); `contract_github` runs against `octocat/Hello-World` (`#[ignore]`-gated; opt-in via `cargo test -- --ignored`). Shared assertion helper `assert_contract<B: IssueBackend>`.

### State-mapping ADR

`docs/decisions/001-github-state-mapping.md` — short, ~50 lines. Decision record:
- `IssueStatus::Open` ↔ `open` (no label).
- `IssueStatus::InProgress` ↔ `open` + label `status/in-progress`.
- `IssueStatus::InReview` ↔ `open` + label `status/in-review`.
- `IssueStatus::Done` ↔ `closed` + `state_reason: completed`.
- `IssueStatus::WontFix` ↔ `closed` + `state_reason: not_planned`.

### Demo suite structure

```
scripts/demos/
├── _lib.sh                     # shared: setup_sim, wait_for_url, cleanup_trap, section
├── 01-edit-and-push.sh         # Tier 1 — core value prop (60s)
├── 02-guardrails.sh            # Tier 1 — security story (60s)
├── 03-conflict-resolution.sh   # Tier 1 — git-merge-as-API-conflict (60s)
├── 04-token-economy.sh         # Tier 1 — bench_token_economy.py runner (10s)
├── full.sh                     # Tier 2 — old monolith, unchanged
├── parity.sh                   # Tier 3 — sim vs github side-by-side (30s)
├── assert.sh                   # wrap-and-grep helper
└── smoke.sh                    # run Tier 1 via assert.sh, CI-friendly
```

Each Tier 1 demo header comment has:
```bash
# AUDIENCE: developer | security | skeptic | buyer
# RUNTIME_SEC: 60
# REQUIRES: cargo, fusermount3, jq, sqlite3
# ASSERTS: "SG-02 fired" "append-only" "DEMO COMPLETE"
```

`assert.sh`:
```bash
#!/usr/bin/env bash
# Usage: scripts/demos/assert.sh scripts/demos/01-edit-and-push.sh
# Runs the demo, captures output, greps each ASSERTS marker. Exits 0 iff all match.
```

The old `scripts/demo.sh` becomes a 2-line shim:
```bash
#!/usr/bin/env bash
exec bash "$(dirname "$0")/demos/full.sh" "$@"
```

### Recording strategy

- Each Tier 1 demo re-recorded → `docs/demos/recordings/{01,02,03,04}.typescript` + `.transcript.txt` (ANSI-stripped excerpts).
- Tier 2 `full.sh` recording stays at `docs/demo.typescript` (unchanged).
- Tier 3 `parity.sh` recording → `docs/demos/recordings/parity.typescript`.

### CI additions

- New job `demos-smoke` in `.github/workflows/ci.yml`: checkout + cargo build --release --bins + `bash scripts/demos/smoke.sh`. ~90s budget.
- `continue-on-error: false` — these are load-bearing. Drift should break CI.

</decisions>

<canonical_refs>
## Canonical References

- `.planning/PROJECT.md` — scope + security guardrails (unchanged).
- `.planning/ROADMAP.md` — this phase appended at the end.
- `.planning/phases/S-stretch-write-path-and-remote-helper/REVIEW.md` — H-04 (create id divergence) is still open; this phase doesn't touch it.
- `crates/reposix-core/src/http.rs` — HttpClient is the ONLY legal HTTP client constructor; both backends use it.
- `crates/reposix-core/src/{issue,path,taint,error}.rs` — shared types.
- `benchmarks/RESULTS.md` — the 92.3% number that `04-token-economy.sh` wraps.
- [GitHub REST API — list issues](https://docs.github.com/en/rest/issues/issues#list-repository-issues)
- [GitHub REST API — get issue](https://docs.github.com/en/rest/issues/issues#get-an-issue)
- [`octocat/Hello-World` issues](https://github.com/octocat/Hello-World/issues) — stable fixture repo.
- [`async-trait` crate](https://docs.rs/async-trait/) — dyn-compatible async trait methods.

</canonical_refs>

<specifics>
## Specific Ideas

- Use `gh api` as the "easy mode" for the demo/parity scripts (no token plumbing needed in demo UX). The library `reposix-github` crate uses raw HTTP via `HttpClient` so it's testable in Rust, not dependent on `gh` being installed.
- `parity.sh` runs `reposix-cli list --backend sim --project demo` and `reposix-cli list --backend github --project octocat/Hello-World`, pipes both through `jq '[.[] | {id, title, status}] | sort_by(.id)'`, and `diff -u`s them. The diff is the demo — "here's what differs, nothing structural."
- Add a `reposix list` subcommand to `reposix-cli` that takes `--backend {sim,github}` and `--project <spec>` and dumps the normalized JSON. Cheap to add; massive demo value.

</specifics>

<deferred>
## Deferred Ideas

- Write path against GitHub (create/update/close with real auth). v0.2.
- Jira adapter (would show a second concrete backend, strengthening the trait). v0.2.
- GraphQL API integration (faster pagination). v0.3.
- Adversarial swarm. Still v0.2 — not in this phase.
- TTY-prompt for `git remote add reposix::...`. v0.2.

</deferred>

---

*Phase: 08-demos-and-real-backend*
*Context: 2026-04-13 09:05 PDT via user-requested post-ship scope.*
