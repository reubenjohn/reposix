---
phase: 17
plan: A
wave: 1
slug: workload-and-cli
type: execute
serial: true
depends_on: []
depends_on_waves: []
blocks_waves: [2]
estimated_wall_clock: 30m
executor_role: gsd-executor
autonomous: true
files_modified:
  - crates/reposix-swarm/Cargo.toml
  - crates/reposix-swarm/src/lib.rs
  - crates/reposix-swarm/src/confluence_direct.rs
  - crates/reposix-swarm/src/main.rs
requirements:
  - SWARM-01
must_haves:
  truths:
    - "reposix-swarm --mode confluence-direct compiles and dispatches to ConfluenceDirectWorkload"
    - "ConfluenceDirectWorkload exercises ConfluenceBackend directly (no FUSE, no retries of its own)"
    - "Each swarm client has its own ConfluenceBackend (and therefore its own rate_limit_gate)"
    - "Workload is strictly read-only in Phase 17 (list + get only; no create/update/delete)"
  artifacts:
    - path: "crates/reposix-swarm/src/confluence_direct.rs"
      provides: "ConfluenceDirectWorkload struct + Workload trait impl"
      contains: "struct ConfluenceDirectWorkload"
    - path: "crates/reposix-swarm/src/lib.rs"
      provides: "pub module export"
      contains: "pub mod confluence_direct"
    - path: "crates/reposix-swarm/src/main.rs"
      provides: "Mode::ConfluenceDirect variant + CLI args + dispatch arm"
      contains: "ConfluenceDirect"
    - path: "crates/reposix-swarm/Cargo.toml"
      provides: "reposix-confluence runtime dependency"
      contains: "reposix-confluence"
  key_links:
    - from: "crates/reposix-swarm/src/main.rs"
      to: "crates/reposix-swarm/src/confluence_direct.rs"
      via: "Mode::ConfluenceDirect dispatch arm in main()"
      pattern: "ConfluenceDirectWorkload::new"
    - from: "crates/reposix-swarm/src/confluence_direct.rs"
      to: "crates/reposix-confluence/src/lib.rs"
      via: "ConfluenceBackend::new_with_base_url + list_issues/get_issue"
      pattern: "ConfluenceBackend::new_with_base_url"
---

## Goal

Wave 1 delivers the core workload and CLI dispatch for Phase 17: add a
`ConfluenceDirectWorkload` that mirrors `SimDirectWorkload` and wire it into
`reposix-swarm` behind `--mode confluence-direct`. Read-only per the locked
decision in CONTEXT.md (writes land in Phase 21 / OP-7). No custom retry or
back-off logic — `ConfluenceBackend`'s internal `rate_limit_gate` owns that.

**Purpose:** proves the Phase 14 `IssueBackend` trait truly generalizes under
concurrent load against a real HTTP backend (SWARM-01). Consumers of this wave
(Wave 2) rely on the Mode variant and the workload's constructor signature.

**Output:** a compiling swarm binary with a new mode, a new `.rs` source file,
and one new Cargo runtime dependency.

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/phases/17-swarm-confluence-direct-mode-add-mode-confluence-direct-to-r/CONTEXT.md
@.planning/phases/17-swarm-confluence-direct-mode-add-mode-confluence-direct-to-r/17-RESEARCH.md
@.planning/phases/17-swarm-confluence-direct-mode-add-mode-confluence-direct-to-r/17-VALIDATION.md
@CLAUDE.md
@crates/reposix-swarm/src/sim_direct.rs
@crates/reposix-swarm/src/main.rs
@crates/reposix-swarm/src/workload.rs
@crates/reposix-swarm/src/lib.rs
@crates/reposix-swarm/Cargo.toml
@crates/reposix-confluence/src/lib.rs

<interfaces>
<!-- Key contracts from reposix-confluence this workload consumes. -->
<!-- Verified directly in crates/reposix-confluence/src/lib.rs lines 115-474. -->

```rust
// From crates/reposix-confluence/src/lib.rs

#[derive(Clone)]
pub struct ConfluenceCreds {
    pub email: String,
    pub api_token: String,   // manual Debug redacts this
}

#[derive(Clone)]
pub struct ConfluenceBackend { /* fields private; Clone is cheap */ }

impl ConfluenceBackend {
    // Production (builds https://{tenant}.atlassian.net)
    pub fn new(creds: ConfluenceCreds, tenant: &str) -> Result<Self>;

    // Tests / wiremock / custom base URLs
    pub fn new_with_base_url(creds: ConfluenceCreds, base_url: String) -> Result<Self>;
}

// From reposix_core::backend::IssueBackend — already implemented for ConfluenceBackend
#[async_trait]
trait IssueBackend {
    async fn list_issues(&self, project: &str) -> Result<Vec<Issue>>;
    async fn get_issue(&self, project: &str, id: IssueId) -> Result<Issue>;
    // + write methods (not used in Phase 17)
}
```

```rust
// From crates/reposix-swarm/src/metrics.rs + workload.rs (already exist)

#[async_trait]
pub trait Workload: Send + Sync + 'static {
    async fn step(&self, metrics: &Arc<MetricsAccumulator>) -> anyhow::Result<()>;
}

impl MetricsAccumulator {
    pub fn record(&self, kind: OpKind, elapsed_us: u64);
    pub fn record_error(&self, kind: ErrorKind);
}

pub enum OpKind { List, Get, Patch /* unused in Phase 17 */ }
impl ErrorKind {
    pub fn classify(err: &reposix_core::Error) -> Self;
}
```
</interfaces>
</context>

## Tasks

<tasks>

<task type="auto" tdd="false">
  <name>Task 17-A-01: Add reposix-confluence runtime dependency + create confluence_direct module stub</name>
  <files>crates/reposix-swarm/Cargo.toml, crates/reposix-swarm/src/lib.rs, crates/reposix-swarm/src/confluence_direct.rs</files>
  <action>
  1. Edit `crates/reposix-swarm/Cargo.toml`: under `[dependencies]`, add
     `reposix-confluence = { path = "../reposix-confluence" }`. Do NOT add it
     to `[dev-dependencies]` in this task — Wave 2 does that.
  2. Edit `crates/reposix-swarm/src/lib.rs`: add `pub mod confluence_direct;`
     alongside the other `pub mod` lines. Keep the crate-level docstring and
     `#![forbid(unsafe_code)]` / `#![warn(clippy::pedantic, missing_docs)]`
     attributes intact.
  3. Create `crates/reposix-swarm/src/confluence_direct.rs`. File header must
     start with a module docstring explaining this is the Confluence mirror of
     `sim_direct.rs` (read-only: list + 3×get; no patch because Phase 17 is
     scoped read-only per CONTEXT.md locked decision). Include crate-level
     lints at the top of the file are NOT needed (lib.rs sets them), but a
     module-level doc-comment IS required so `missing_docs` stays clean.

     Define the struct skeleton and constructor only in this task:

     ```rust
     //! `confluence-direct` workload: each client drives ConfluenceBackend
     //! directly over HTTP. Mirror of `sim_direct.rs` minus the patch step
     //! (Phase 17 is read-only by design — writes ship in Phase 21 / OP-7).
     //!
     //! Rate-limit handling is transparent: ConfluenceBackend's internal
     //! `rate_limit_gate` sleeps on 429 Retry-After; the workload records a
     //! `RateLimited` error only if the backend surfaces one.

     use std::sync::Arc;
     use std::time::Instant;

     use async_trait::async_trait;
     use parking_lot::Mutex;
     use rand::rngs::StdRng;
     use rand::{Rng, SeedableRng};
     use reposix_confluence::{ConfluenceBackend, ConfluenceCreds};
     use reposix_core::{IssueBackend, IssueId};

     use crate::metrics::{ErrorKind, MetricsAccumulator, OpKind};
     use crate::workload::Workload;

     /// A confluence-direct workload instance. Holds a per-client
     /// [`ConfluenceBackend`] (so each swarm client has its own rate-limit
     /// gate) and a per-client RNG.
     pub struct ConfluenceDirectWorkload {
         backend: ConfluenceBackend,
         space: String,
         rng: Mutex<StdRng>,
         /// Cached ids from the most recent `list_issues` call.
         ids: Mutex<Vec<IssueId>>,
     }

     impl ConfluenceDirectWorkload {
         /// Build a new instance.
         ///
         /// `base_url` is the Confluence tenant base (e.g.
         /// `https://tenant.atlassian.net`) or a wiremock URI for tests.
         /// `space` is the Confluence space key used as the `project`
         /// argument to [`IssueBackend::list_issues`].
         ///
         /// # Errors
         /// Propagates [`ConfluenceBackend::new_with_base_url`] failures
         /// (allowlist build errors, invalid base URL).
         pub fn new(
             base_url: String,
             creds: ConfluenceCreds,
             space: String,
             seed: u64,
         ) -> anyhow::Result<Self> {
             let backend = ConfluenceBackend::new_with_base_url(creds, base_url)
                 .map_err(|e| anyhow::anyhow!("ConfluenceBackend init: {e}"))?;
             Ok(Self {
                 backend,
                 space,
                 rng: Mutex::new(StdRng::seed_from_u64(seed)),
                 ids: Mutex::new(Vec::new()),
             })
         }

         fn random_id(&self) -> Option<IssueId> {
             let ids = self.ids.lock();
             if ids.is_empty() {
                 return None;
             }
             let mut r = self.rng.lock();
             let idx = r.gen_range(0..ids.len());
             Some(ids[idx])
         }
     }
     ```

     Task 17-A-02 fills in the `Workload` impl. Stop here for this task.

  4. Run `cargo check -p reposix-swarm` — must pass. `cargo clippy
     -p reposix-swarm --all-targets -- -D warnings` must pass (dead-code
     warnings on the struct fields are suppressed because `backend`/`space`
     are read in the constructor via `Self { ... }`, and `rng`/`ids` are used
     in `random_id`).
  </action>
  <behavior>
    - `cargo check -p reposix-swarm` compiles cleanly
    - `cargo clippy -p reposix-swarm --all-targets -- -D warnings` is clean
    - `crates/reposix-swarm/src/confluence_direct.rs` exists with struct +
      `new()` + `random_id()`; no `Workload` impl yet
    - `reposix-confluence` appears exactly once under `[dependencies]` in
      `crates/reposix-swarm/Cargo.toml`
  </behavior>
  <verify>
    <automated>cargo check -p reposix-swarm &amp;&amp; cargo clippy -p reposix-swarm --all-targets -- -D warnings</automated>
  </verify>
  <done>
    Module skeleton compiles; reposix-confluence wired as runtime dep; lib.rs
    exports the new module; no clippy warnings.
  </done>
</task>

<task type="auto" tdd="false">
  <name>Task 17-A-02: Implement Workload::step (list + 3×get) + add Mode::ConfluenceDirect CLI dispatch</name>
  <files>crates/reposix-swarm/src/confluence_direct.rs, crates/reposix-swarm/src/main.rs</files>
  <action>
  1. In `crates/reposix-swarm/src/confluence_direct.rs`, append the `Workload`
     impl after the `impl ConfluenceDirectWorkload { ... }` block. Mirror
     `sim_direct.rs::step` exactly EXCEPT (a) no patch step, (b) use `space`
     instead of `project`. Read-only is a locked decision (CONTEXT.md);
     executors MUST NOT add a patch or any write op in this phase.

     ```rust
     #[async_trait]
     impl Workload for ConfluenceDirectWorkload {
         async fn step(&self, metrics: &Arc<MetricsAccumulator>) -> anyhow::Result<()> {
             // 1. list
             let start = Instant::now();
             match self.backend.list_issues(&self.space).await {
                 Ok(issues) => {
                     metrics.record(OpKind::List, elapsed_us(start));
                     let mut g = self.ids.lock();
                     g.clear();
                     g.extend(issues.iter().map(|i| i.id));
                 }
                 Err(err) => {
                     metrics.record(OpKind::List, elapsed_us(start));
                     metrics.record_error(ErrorKind::classify(&err));
                 }
             }

             // 2. 3 × get (random ids; break early if list never populated
             //    the cache)
             for _ in 0..3 {
                 let Some(id) = self.random_id() else {
                     break;
                 };
                 let start = Instant::now();
                 match self.backend.get_issue(&self.space, id).await {
                     Ok(_issue) => {
                         metrics.record(OpKind::Get, elapsed_us(start));
                     }
                     Err(err) => {
                         metrics.record(OpKind::Get, elapsed_us(start));
                         metrics.record_error(ErrorKind::classify(&err));
                     }
                 }
             }
             // NOTE: no patch step in Phase 17 (read-only; writes in Phase 21).
             Ok(())
         }
     }

     /// Elapsed-microseconds helper. Saturates at `u64::MAX` for any duration
     /// that somehow overflows (practically impossible for a swarm run).
     fn elapsed_us(start: Instant) -> u64 {
         u64::try_from(start.elapsed().as_micros()).unwrap_or(u64::MAX)
     }
     ```

  2. Edit `crates/reposix-swarm/src/main.rs`:

     a. Extend the `Mode` enum with a `ConfluenceDirect` variant (kebab-case
        rendered as `confluence-direct`):

        ```rust
        #[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
        #[clap(rename_all = "kebab_case")]
        enum Mode {
            /// HTTP to the simulator via `SimBackend`.
            SimDirect,
            /// HTTP to `ConfluenceBackend` directly (read-only in v0.6).
            ConfluenceDirect,
            /// Real syscalls against a FUSE mount point.
            Fuse,
        }
        ```

        Update `Mode::as_str` to return `"confluence-direct"` for the new
        variant.

     b. Add two new CLI flags to `Args`, placed after `project`:

        ```rust
        /// Atlassian account email (required for `confluence-direct`).
        #[arg(long)]
        email: Option<String>,

        /// Atlassian API token (required for `confluence-direct`). Falls back
        /// to the `ATLASSIAN_API_KEY` env var.
        #[arg(long, env = "ATLASSIAN_API_KEY")]
        api_token: Option<String>,
        ```

     c. Add the dispatch arm to the `match args.mode` block:

        ```rust
        Mode::ConfluenceDirect => {
            let email = args
                .email
                .clone()
                .ok_or_else(|| anyhow::anyhow!("--email required for confluence-direct"))?;
            let token = args
                .api_token
                .clone()
                .ok_or_else(|| anyhow::anyhow!(
                    "--api-token or ATLASSIAN_API_KEY env var required for confluence-direct"
                ))?;
            let creds = reposix_confluence::ConfluenceCreds {
                email,
                api_token: token,
            };
            let base = args.target.clone();
            let space = args.project.clone();
            run_swarm(cfg, |i| {
                reposix_swarm::confluence_direct::ConfluenceDirectWorkload::new(
                    base.clone(),
                    creds.clone(),
                    space.clone(),
                    u64::try_from(i).unwrap_or(0),
                )
            })
            .await?
        }
        ```

        The import line at the top of main.rs (`use reposix_swarm::...`) may
        be extended OR the full path may be kept inline — either is
        acceptable; match the style already in main.rs.

  3. Verify:
     - `cargo check -p reposix-swarm` passes.
     - `cargo clippy -p reposix-swarm --all-targets -- -D warnings` is clean.
     - `cargo run -p reposix-swarm -- --mode confluence-direct --help` lists
       the new `--email` and `--api-token` flags.
     - `cargo run -p reposix-swarm -- --mode confluence-direct --target
       http://127.0.0.1:1 --project TEST --duration 1 --clients 1` fails
       cleanly with "--email required" (proves the arg is wired).

  Do NOT run any real-network command. Do NOT write any test in this task —
  Wave 2 owns all tests.
  </action>
  <behavior>
    - Test invariants from RESEARCH.md §"Pattern 2":
      - `step` records exactly one `OpKind::List` per call
      - `step` records 0..=3 `OpKind::Get` (3 if list succeeded and populated
        ids; 0 on first iteration if list failed)
      - `step` NEVER records `OpKind::Patch` (locked decision)
      - Errors from `list_issues`/`get_issue` are classified via
        `ErrorKind::classify`, never swallowed silently
    - CLI invariants:
      - `Mode::ConfluenceDirect.as_str() == "confluence-direct"`
      - `--email` missing for confluence-direct mode → anyhow error with
        message containing "--email"
      - `--api-token` missing AND `ATLASSIAN_API_KEY` unset → anyhow error
  </behavior>
  <verify>
    <automated>cargo check -p reposix-swarm &amp;&amp; cargo clippy -p reposix-swarm --all-targets -- -D warnings &amp;&amp; cargo run -p reposix-swarm -- --mode confluence-direct --help 2&gt;&amp;1 | grep -q -- --email</automated>
  </verify>
  <done>
    Workload impl complete (list + 3×get, no patch). Mode::ConfluenceDirect
    compiles, dispatches, and rejects runs that lack credentials. Clippy clean.
    No test changes in this task (Wave 2 owns tests).
  </done>
</task>

</tasks>

## Verification

Phase-level sampling for Wave 1:

- `cargo check --workspace` — whole workspace still compiles.
- `cargo clippy --workspace --all-targets -- -D warnings` — zero warnings.
- `cargo test --workspace` — test count MUST remain ≥ 317 (Phase 16 baseline).
  Wave 1 adds no tests, so the count stays exactly at the baseline.
- `cargo run -p reposix-swarm -- --mode confluence-direct --help` prints the
  `--email` and `--api-token` flags.

## Threat Model

Trust boundaries touched by Wave 1:

| Boundary | Description |
|----------|-------------|
| swarm client → Confluence HTTP | Each workload client issues outbound HTTPS (or mock HTTP). SG-01 allowlist enforces origin in `ConfluenceBackend::new_with_base_url` via the shared `HttpClient`. |
| CLI flags → process env | `--api-token` falls back to `ATLASSIAN_API_KEY` env var; token is held in `ConfluenceCreds` which has manual Debug redaction. |

STRIDE register (scoped to Wave 1 code changes):

| Threat ID | Category | Component | Disposition | Mitigation |
|-----------|----------|-----------|-------------|-----------|
| T-17-01 | Information Disclosure | main.rs credential handling | mitigate | Credentials are moved directly into `ConfluenceCreds` (which has manual `Debug` redaction). They MUST NOT be logged via `tracing::info!`/`println!` in the dispatch arm. |
| T-17-02 | Spoofing / SSRF | Mode::ConfluenceDirect dispatch | mitigate | `base_url` from `--target` is passed unchanged to `ConfluenceBackend::new_with_base_url`, which delegates to the core `HttpClient` SG-01 allowlist check. No string concatenation in main.rs. |
| T-17-03 | Denial of Service | ConfluenceDirectWorkload::step | accept | The workload intentionally hammers the backend; rate limiting is the backend's responsibility (transparent `rate_limit_gate`). Acceptance rationale: this is the swarm harness's job. |
| T-17-04 | Tampering | Workload payloads | mitigate-by-scope | Phase 17 is read-only — no write ops means no tampered payloads leaving the client. Re-assess in Phase 21 / OP-7 when writes are added. |

## Success Criteria

Wave 1 is done when:

- [ ] `crates/reposix-swarm/src/confluence_direct.rs` exists with
      `ConfluenceDirectWorkload` struct, `new()`, `random_id()`, and `Workload`
      impl. No patch step. Module docstring present.
- [ ] `crates/reposix-swarm/src/lib.rs` exports the module via
      `pub mod confluence_direct;`.
- [ ] `crates/reposix-swarm/src/main.rs` has `Mode::ConfluenceDirect`,
      `--email`, `--api-token` (with `ATLASSIAN_API_KEY` env fallback), and a
      dispatch arm that calls `ConfluenceDirectWorkload::new` via `run_swarm`.
- [ ] `crates/reposix-swarm/Cargo.toml` lists `reposix-confluence` under
      `[dependencies]` (not yet under `[dev-dependencies]` — Wave 2).
- [ ] `cargo check --workspace` passes.
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` is clean.
- [ ] `cargo test --workspace` still passes with test count ≥ 317.
- [ ] `--mode confluence-direct --help` lists the new flags.

<output>
After completion, create
`.planning/phases/17-swarm-confluence-direct-mode-add-mode-confluence-direct-to-r/17-A-SUMMARY.md`
documenting the artifacts created, the Mode variant added, and the literal
fact that no tests were added in Wave 1 (Wave 2 adds them).
</output>
