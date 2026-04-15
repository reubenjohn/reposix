---
phase: 21
plan: B
type: execute
wave: 2
depends_on: [21-A]
files_modified:
  - crates/reposix-swarm/src/contention.rs
  - crates/reposix-swarm/src/lib.rs
  - crates/reposix-swarm/src/main.rs
  - crates/reposix-swarm/tests/contention_e2e.rs
autonomous: true
requirements:
  - HARD-01
user_setup: []
tags: [hardening, swarm, contention, if-match]

must_haves:
  truths:
    - "User can run `reposix-swarm --mode contention --target http://127.0.0.1:<port> --project demo --target-issue <id> --clients 50 --duration 30` and get a markdown summary with Patch win_count > 0 and Conflict count > 0"
    - "The integration test proves win_count equals the final version increment count in the audit log (no torn writes)"
    - "No `Other`-class errors appear in the summary (If-Match path is the only error source)"
  artifacts:
    - path: "crates/reposix-swarm/src/contention.rs"
      provides: "ContentionWorkload — N-client If-Match storm against same issue"
      min_lines: 80
      exports: ["ContentionWorkload"]
    - path: "crates/reposix-swarm/src/lib.rs"
      provides: "pub mod contention declaration"
      contains: "pub mod contention"
    - path: "crates/reposix-swarm/src/main.rs"
      provides: "Mode::Contention CLI dispatch + --target-issue arg"
      contains: "Mode::Contention"
    - path: "crates/reposix-swarm/tests/contention_e2e.rs"
      provides: "integration test: 50 clients, 5s (not 30s in CI), deterministic 409s"
      contains: "#[tokio::test"
  key_links:
    - from: "crates/reposix-swarm/src/main.rs"
      to: "crates/reposix-swarm/src/contention.rs"
      via: "use reposix_swarm::contention::ContentionWorkload"
      pattern: "ContentionWorkload::new"
    - from: "crates/reposix-swarm/src/contention.rs"
      to: "crates/reposix-sim/src/routes/issues.rs"
      via: "update_issue with Some(version) triggers If-Match 409 path"
      pattern: "update_issue.*Some\\("
    - from: "crates/reposix-swarm/tests/contention_e2e.rs"
      to: "crates/reposix-swarm/src/contention.rs"
      via: "direct constructor call; spawns sim via run_with_listener"
      pattern: "ContentionWorkload::new"
---

<objective>
Add `Mode::Contention` to `reposix-swarm`. The new `ContentionWorkload` has N clients all hammering the same issue with explicit `If-Match: "<version>"` PATCHes, proving the sim's 409 path is deterministic and that every winning write produces exactly one monotonic version bump (no torn writes, no silent drops).

Purpose: HARD-01 (REQUIREMENTS.md v0.7.0). `SimDirectWorkload` today uses wildcard etags (`None`), which never provokes 409s — so the existing swarm harness does NOT actually exercise the If-Match conflict path under concurrent load. Without this plan, `If-Match` is "shipped but unmeasured".

Output: A new workload module + CLI mode + an integration test that asserts `conflict_count > 0`, `win_count > 0`, and `win_count == final_version`.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/phases/21-op-7-hardening-bundle-contention-swarm-500-page-truncation-p/CONTEXT.md
@.planning/phases/21-op-7-hardening-bundle-contention-swarm-500-page-truncation-p/21-RESEARCH.md
@.planning/phases/21-op-7-hardening-bundle-contention-swarm-500-page-truncation-p/21-PATTERNS.md
@crates/reposix-swarm/src/sim_direct.rs
@crates/reposix-swarm/src/workload.rs
@crates/reposix-swarm/src/metrics.rs
@crates/reposix-swarm/src/main.rs
@crates/reposix-swarm/tests/mini_e2e.rs

<interfaces>
<!-- Key contracts the executor uses. Extracted from codebase. -->

From crates/reposix-swarm/src/workload.rs:
```rust
#[async_trait]
pub trait Workload: Send + Sync {
    async fn step(&self, metrics: &Arc<MetricsAccumulator>) -> anyhow::Result<()>;
}
```

From crates/reposix-swarm/src/metrics.rs:
```rust
pub enum OpKind { List, Get, Patch, /* ... */ }
pub enum ErrorKind { Conflict, Transport, Other, /* ... */ }
pub struct MetricsAccumulator { /* pub fn record(OpKind, u64); record_error(ErrorKind); */ }
```

From reposix_core (already in swarm deps):
```rust
// Trait method used for PATCH with explicit version:
async fn update_issue(
    &self,
    project: &str,
    id: IssueId,
    patch: Untainted<Issue>,
    expected_version: Option<u64>,  // Some(v) triggers If-Match; None = wildcard
) -> Result<Issue>;
```

From crates/reposix-swarm/src/sim_direct.rs (analog to copy):
- Constructor pattern: `SimBackend::with_agent_suffix(origin, Some(&suffix))?`
- Elapsed helper: `fn elapsed_us(start: Instant) -> u64 { u64::try_from(start.elapsed().as_micros()).unwrap_or(u64::MAX) }`

From crates/reposix-swarm/src/main.rs:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
#[clap(rename_all = "kebab_case")]
enum Mode { SimDirect, ConfluenceDirect, Fuse /* ADD Contention */ }
```
</interfaces>
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task B1: Create ContentionWorkload (module + Workload impl) with failing stub test</name>
  <files>
    crates/reposix-swarm/src/contention.rs,
    crates/reposix-swarm/src/lib.rs,
    crates/reposix-swarm/tests/contention_e2e.rs
  </files>
  <read_first>
    - crates/reposix-swarm/src/sim_direct.rs
    - crates/reposix-swarm/src/workload.rs
    - crates/reposix-swarm/src/metrics.rs
    - crates/reposix-swarm/tests/mini_e2e.rs
    - .planning/phases/21-op-7-hardening-bundle-contention-swarm-500-page-truncation-p/21-PATTERNS.md (section "crates/reposix-swarm/src/contention.rs")
  </read_first>
  <behavior>
    - ContentionWorkload::new(origin: String, project: String, target_id: IssueId, seed: u64) -> anyhow::Result<Self>
    - Workload::step: GET target issue → extract version → PATCH with Some(version) → on Ok record OpKind::Patch (win); on Err with 409-class, record OpKind::Patch + ErrorKind::Conflict; on any other Err, return it (propagates to run_swarm)
    - The test in contention_e2e.rs asserts: win_count > 0, conflict_count > 0, other_count == 0, win_count == final issue version (after floor v=1 for initial state, so: final_version - starting_version == win_count)
  </behavior>
  <action>
    Create `crates/reposix-swarm/src/contention.rs` following the analog pattern from `sim_direct.rs`:

    ```rust
    #![forbid(unsafe_code)]
    #![warn(clippy::pedantic)]

    use std::sync::Arc;
    use std::time::Instant;

    use async_trait::async_trait;
    use chrono::Utc;
    use parking_lot::Mutex;
    use rand::rngs::StdRng;
    use rand::SeedableRng;
    use reposix_core::backend::sim::SimBackend;
    use reposix_core::{sanitize, Issue, IssueBackend, IssueId, ServerMetadata, Tainted};

    use crate::metrics::{ErrorKind, MetricsAccumulator, OpKind};
    use crate::workload::Workload;

    /// Workload where N clients hammer the same issue with explicit If-Match
    /// versions. One winner per version; the rest must 409. Proves the
    /// simulator's version-gating path is deterministic under load.
    pub struct ContentionWorkload {
        backend: SimBackend,
        project: String,
        target_id: IssueId,
        #[allow(dead_code)] // reserved for future jitter; not used in step() today
        rng: Mutex<StdRng>,
    }

    impl ContentionWorkload {
        /// # Errors
        /// Returns an error if `SimBackend::with_agent_suffix` fails (bad origin URL, etc.).
        pub fn new(origin: String, project: String, target_id: IssueId, seed: u64) -> anyhow::Result<Self> {
            let suffix = format!("contention-{seed}");
            let backend = SimBackend::with_agent_suffix(origin, Some(&suffix))?;
            Ok(Self { backend, project, target_id, rng: Mutex::new(StdRng::seed_from_u64(seed)) })
        }
    }

    #[async_trait]
    impl Workload for ContentionWorkload {
        async fn step(&self, metrics: &Arc<MetricsAccumulator>) -> anyhow::Result<()> {
            // 1. GET live version (unsynchronised across clients → intentionally racy)
            let get_start = Instant::now();
            let issue = match self.backend.get_issue(&self.project, self.target_id).await {
                Ok(i) => i,
                Err(e) => {
                    metrics.record(OpKind::Get, elapsed_us(get_start));
                    metrics.record_error(ErrorKind::classify(&e));
                    return Ok(()); // non-fatal; try next step
                }
            };
            metrics.record(OpKind::Get, elapsed_us(get_start));
            let current_version = issue.version;

            // 2. Build mutation (append to title → small, deterministic, bytes-in-bytes-out)
            let mut patched = issue.clone();
            patched.title = format!("{} [c{seed}]", patched.title, seed = current_version);
            patched.updated_at = Utc::now();
            let untainted = sanitize(
                Tainted::new(patched),
                ServerMetadata { id: self.target_id, created_at: issue.created_at, version: current_version },
            );

            // 3. PATCH with If-Match: <current_version> — explicit version triggers 409 path
            let patch_start = Instant::now();
            match self
                .backend
                .update_issue(&self.project, self.target_id, untainted, Some(current_version))
                .await
            {
                Ok(_) => {
                    metrics.record(OpKind::Patch, elapsed_us(patch_start)); // win
                }
                Err(err) => {
                    metrics.record(OpKind::Patch, elapsed_us(patch_start));
                    metrics.record_error(ErrorKind::classify(&err)); // 409 = Conflict (expected)
                }
            }
            Ok(())
        }
    }

    fn elapsed_us(start: Instant) -> u64 {
        u64::try_from(start.elapsed().as_micros()).unwrap_or(u64::MAX)
    }
    ```

    Wire the module in `crates/reposix-swarm/src/lib.rs`: add `pub mod contention;` alongside the other `pub mod` lines (keep them alphabetical: confluence_direct, **contention**, driver, fuse_mode, metrics, sim_direct, workload).

    Create `crates/reposix-swarm/tests/contention_e2e.rs` as a RED-phase test (will not compile until B1 code lands, will fail until B2 seeds real data). Copy the `spawn_sim` + `audit_row_count` helpers verbatim from `crates/reposix-swarm/tests/mini_e2e.rs`. Test body:

    ```rust
    #![forbid(unsafe_code)]
    #![warn(clippy::pedantic)]
    #![allow(clippy::missing_panics_doc)]

    use std::time::Duration;

    use reposix_core::http::{client, ClientOpts};
    use reposix_sim::{run_with_listener, SimConfig};
    use reposix_swarm::contention::ContentionWorkload;
    use reposix_swarm::driver::{run_swarm, SwarmConfig};
    use tempfile::NamedTempFile;

    // ... (copy spawn_sim + seed_fixture + audit_row_count from mini_e2e.rs) ...

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn contention_50_clients_5s_deterministic_409() {
        let (base, _db, sim_handle) = spawn_sim(200).await;
        // Use issue id 1 from the fixture (seed_fixture provides it)
        let target_id = reposix_core::IssueId::new(1);

        let cfg = SwarmConfig {
            clients: 50,
            duration: Duration::from_secs(5),
            target: base.clone(),
            // whatever the actual SwarmConfig fields are — mirror mini_e2e.rs exactly
            ..Default::default()
        };

        let markdown = run_swarm(cfg, |i| {
            ContentionWorkload::new(base.clone(), "demo".to_string(), target_id, u64::try_from(i).unwrap_or(0))
        })
        .await
        .expect("run_swarm");

        // Three invariants:
        // (a) At least one winning PATCH — If-Match isn't locking out every client
        // (b) At least one Conflict — clients are actually racing (If-Match is enforced)
        // (c) No Other-class errors — no transport/serialization bugs
        assert!(markdown.contains("| Patch"), "expected Patch op rows in summary");
        assert!(!markdown.contains("| Other"), "Other-class errors present:\n{markdown}");
        // Conflict count assertion is structural — look for "Conflict" row with count > 0.
        // Exact marker depends on MetricsAccumulator markdown format; confirm against
        // mini_e2e.rs assertions when implementing, and adjust the substring.

        sim_handle.abort();
        let _ = sim_handle.await;
    }
    ```

    **Note on Default for SwarmConfig**: if `SwarmConfig` has no `Default` impl, mirror the exact construction used in `mini_e2e.rs`. Do NOT invent fields.

    **Note on `SimBackend::with_agent_suffix`**: confirm the signature by reading the first 10 lines of `sim_direct.rs` lines 41-59 before writing `contention.rs`. If it takes `&str`, adapt. If the `reposix_core::IssueId::new(1)` constructor doesn't exist, use whatever `mini_e2e.rs` uses to obtain an IssueId (it calls `list_issues` and takes the first one — same pattern is acceptable here, but the fixture must have at least 1 issue seeded).

    Run `cargo check -p reposix-swarm --tests --locked`. Expect a clean compile. Run `cargo test -p reposix-swarm --test contention_e2e --locked -- --nocapture`. Expect FAILURE (test panics with a concrete assertion — if the SwarmConfig fields mismatch, fix them; the point is to get to a *failing assertion*, not a compile error).

    Commit: `git add crates/reposix-swarm/src/contention.rs crates/reposix-swarm/src/lib.rs crates/reposix-swarm/tests/contention_e2e.rs && git commit -m "test(21-B): failing stub for ContentionWorkload + contention_e2e"`
  </action>
  <verify>
    <automated>cargo check -p reposix-swarm --tests --locked && cargo test -p reposix-swarm --test contention_e2e --locked -- --nocapture 2>&1 | grep -qE "(FAILED|test result: FAILED)"</automated>
  </verify>
  <acceptance_criteria>
    - File `crates/reposix-swarm/src/contention.rs` exists and is >= 80 lines
    - `grep -q "pub mod contention" crates/reposix-swarm/src/lib.rs` succeeds
    - `grep -q "ContentionWorkload" crates/reposix-swarm/src/contention.rs` succeeds
    - `grep -q "update_issue.*Some" crates/reposix-swarm/src/contention.rs` succeeds (explicit If-Match version)
    - `cargo check -p reposix-swarm --tests --locked` exits 0 (code compiles)
    - `cargo test -p reposix-swarm --test contention_e2e --locked` exits non-zero (RED phase — test must fail, proving the assertion is real)
  </acceptance_criteria>
  <done>
    Workload module compiles; stub test fails for a concrete, assertion-level reason; both files committed.
  </done>
</task>

<task type="auto" tdd="true">
  <name>Task B2: Wire Mode::Contention into the binary + make the e2e test pass</name>
  <files>
    crates/reposix-swarm/src/main.rs,
    crates/reposix-swarm/tests/contention_e2e.rs
  </files>
  <read_first>
    - crates/reposix-swarm/src/main.rs
    - crates/reposix-swarm/src/contention.rs (from B1)
    - crates/reposix-swarm/tests/contention_e2e.rs (from B1)
    - .planning/phases/21-op-7-hardening-bundle-contention-swarm-500-page-truncation-p/21-PATTERNS.md (section "crates/reposix-swarm/src/main.rs")
  </read_first>
  <behavior>
    - `reposix-swarm --mode contention --target-issue <id> --clients 50 --duration 5s --target http://127.0.0.1:<port> --project demo` runs to completion and prints markdown summary.
    - `--target-issue` is required when `--mode contention`, rejected otherwise (clap validation OR a runtime check in main).
    - After B2, `cargo test -p reposix-swarm --test contention_e2e --locked` passes (all three assertions from B1 satisfied).
  </behavior>
  <action>
    In `crates/reposix-swarm/src/main.rs`:

    1. Add `Contention` to the `Mode` enum (between `ConfluenceDirect` and `Fuse` — keep source-order tidy):
       ```rust
       enum Mode {
           SimDirect,
           ConfluenceDirect,
           /// N clients patching the same issue via If-Match — proves 409 determinism.
           Contention,
           Fuse,
       }
       ```

    2. Add a `--target-issue <u64>` CLI arg to the main `Args` struct. Use `Option<u64>` so it's only required when `--mode contention`. Follow the existing `#[arg(long, ...)]` decoration style.

    3. Add a match arm for `Mode::Contention` in the dispatch block. Model it exactly on the `Mode::SimDirect` arm (lines ~104–116 per PATTERNS.md):
       ```rust
       Mode::Contention => {
           let origin = args.target.clone();
           let project = args.project.clone();
           let target_id = args
               .target_issue
               .ok_or_else(|| anyhow::anyhow!("--target-issue is required for --mode contention"))?;
           let target_id = reposix_core::IssueId::new(target_id);
           run_swarm(cfg, move |i| {
               ContentionWorkload::new(
                   origin.clone(),
                   project.clone(),
                   target_id,
                   u64::try_from(i).unwrap_or(0),
               )
           }).await?
       }
       ```

    4. Add `use reposix_swarm::contention::ContentionWorkload;` to the top of main.rs.

    5. Run the e2e test and iterate until green. Likely adjustments:
       - The test's seed fixture may not have an issue — add one via direct backend call in the test setup, OR use the first id from `list_issues`.
       - The Conflict-row markdown substring in the assertion depends on `MetricsAccumulator` format — look at the summary that `run_swarm` prints and pin the exact substring.
       - `seed_fixture()` path: copy from `mini_e2e.rs`; it's a PathBuf to `fixtures/seed.json` (or similar) in the swarm crate.

    6. Run full test: `cargo test -p reposix-swarm --test contention_e2e --locked -- --nocapture`. The `-- --nocapture` lets you see the printed markdown summary. Expect PASS.

    7. Run clippy: `cargo clippy -p reposix-swarm --all-targets -- -D warnings`. Fix any lints (prefer explicit types over allow-ing pedantic lints; if a pedantic lint is genuinely wrong for this code, add `#[allow(clippy::<name>)]` with a one-line rationale per CLAUDE.md).

    8. Commit: `git add crates/reposix-swarm/src/main.rs crates/reposix-swarm/tests/contention_e2e.rs && git commit -m "feat(21-B): Mode::Contention + ContentionWorkload wired end-to-end (HARD-01)"`
  </action>
  <verify>
    <automated>cargo test -p reposix-swarm --test contention_e2e --locked && cargo clippy -p reposix-swarm --all-targets -- -D warnings</automated>
  </verify>
  <acceptance_criteria>
    - `grep -q "Mode::Contention" crates/reposix-swarm/src/main.rs` succeeds
    - `grep -q "target_issue" crates/reposix-swarm/src/main.rs` succeeds
    - `grep -q "use reposix_swarm::contention::ContentionWorkload" crates/reposix-swarm/src/main.rs` succeeds
    - `cargo test -p reposix-swarm --test contention_e2e --locked` exits 0 (GREEN)
    - `cargo clippy -p reposix-swarm --all-targets -- -D warnings` exits 0
    - `cargo run -p reposix-swarm -- --help 2>&1 | grep -q "contention"` (the new mode is discoverable in --help output)
  </acceptance_criteria>
  <done>
    `Mode::Contention` shipped; e2e test green; HARD-01 closes.
  </done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| contention client tasks → sim (HTTP) | each task is a trusted in-process client; sim is the SUT |
| sim `patch_issue` handler → audit_events WAL | server-controlled write path; already sanitised |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-21-B-01 | Denial of Service | sim under 50-client load | accept | Test uses 5s duration (not 30s) to stay within CI budget; rate_limit_rps=200 prevents runaway |
| T-21-B-02 | Tampering | issue body tainted input echoed through sanitize() | mitigate | Reuse existing `sanitize(Tainted::new(...), ServerMetadata {...})` path — server fields (id, created_at) cannot be overwritten by the client; this is tested in sim already |
| T-21-B-03 | Information Disclosure | markdown summary printed to stdout | accept | Summary contains counts + latencies only; no tenant or secret data. SimBackend is local-only (127.0.0.1 allowlist). |
| T-21-B-04 | Spoofing | client seed collision | accept | Each client uses seed `u64::try_from(i)` from `run_swarm`; deterministic and cannot spoof server identity |
</threat_model>

<verification>
- `cargo test -p reposix-swarm --locked` (full crate) green
- `cargo clippy -p reposix-swarm --all-targets -- -D warnings` clean
- `cargo test --workspace --locked` green (no regressions in sim_direct / confluence_direct / fuse workloads)
- e2e asserts win_count > 0, conflict_count > 0, other == 0
</verification>

<success_criteria>
HARD-01 closes: the swarm harness now has a mode that deterministically provokes 409s, and the integration test encodes the "no torn writes" invariant (wins equal version increments). Future regressions in the sim's If-Match path will fail this test on every CI run.
</success_criteria>

<output>
After completion, create `.planning/phases/21-op-7-hardening-bundle-contention-swarm-500-page-truncation-p/21-B-SUMMARY.md` with: new files added, Mode::Contention invocation example, test count delta, clippy clean confirmation.
</output>
