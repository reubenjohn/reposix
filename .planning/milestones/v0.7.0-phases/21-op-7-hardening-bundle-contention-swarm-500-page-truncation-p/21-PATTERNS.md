# Phase 21: OP-7 Hardening Bundle — Pattern Map

**Mapped:** 2026-04-15
**Files analyzed:** 8 new/modified files
**Analogs found:** 8 / 8

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `crates/reposix-swarm/src/contention.rs` | workload | request-response (If-Match storm) | `crates/reposix-swarm/src/sim_direct.rs` | exact |
| `crates/reposix-swarm/tests/contention_e2e.rs` | test | request-response | `crates/reposix-swarm/tests/mini_e2e.rs` | exact |
| `crates/reposix-swarm/tests/chaos_audit.rs` | test | event-driven (kill-9 + WAL) | `crates/reposix-swarm/tests/mini_e2e.rs` | role-match |
| `crates/reposix-confluence/src/lib.rs` | service | CRUD / batch pagination | self (modify) | self |
| `crates/reposix-swarm/src/main.rs` | binary entrypoint | request-response | self (modify Mode enum) | self |
| `crates/reposix-cli/src/main.rs` | CLI | request-response | self (add `--no-truncate`) | self |
| `.github/workflows/ci.yml` | config | batch | self (modify matrix) | self |
| `scripts/hooks/test-pre-push.sh` (audit only) | utility/test | batch | self | self |

---

## Pattern Assignments

### `crates/reposix-swarm/src/contention.rs` (workload, request-response)

**Analog:** `crates/reposix-swarm/src/sim_direct.rs`

**Imports pattern** (lines 1–28):
```rust
use std::sync::Arc;
use std::time::Instant;

use async_trait::async_trait;
use parking_lot::Mutex;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use reposix_core::backend::sim::SimBackend;
use reposix_core::{sanitize, Issue, IssueBackend, IssueId, IssueStatus, ServerMetadata, Tainted};
use chrono::Utc;

use crate::metrics::{ErrorKind, MetricsAccumulator, OpKind};
use crate::workload::Workload;
```

**Struct declaration pattern** (lines 30–39):
```rust
pub struct SimDirectWorkload {
    backend: SimBackend,
    project: String,
    rng: Mutex<StdRng>,
    ids: Mutex<Vec<IssueId>>,
}
```
For `ContentionWorkload` replace `ids: Mutex<Vec<IssueId>>` with a fixed `target_id: IssueId` (no random selection — all clients hammer the same issue).

**Constructor pattern** (lines 41–59):
```rust
pub fn new(origin: String, project: String, seed: u64) -> anyhow::Result<Self> {
    let suffix = format!("swarm-{seed}");
    let backend = SimBackend::with_agent_suffix(origin, Some(&suffix))?;
    Ok(Self {
        backend,
        project,
        rng: Mutex::new(StdRng::seed_from_u64(seed)),
        ids: Mutex::new(Vec::new()),
    })
}
```
`ContentionWorkload::new` adds `target_id: IssueId` as a fourth argument; `ids` cache is dropped.

**Core step pattern — the key divergence** (lines 109–151 of sim_direct.rs):

The existing workload passes `None` as `expected_version` (wildcard etag, never 409s):
```rust
// sim_direct.rs line 139 — wildcard etag:
match self.backend.update_issue(&self.project, id, untainted, None).await {
```

`ContentionWorkload::step` MUST pass `Some(version)` to provoke 409s:
```rust
// Pattern for ContentionWorkload::step:
// 1. GET to read current version
let issue = self.backend.get_issue(&self.project, self.target_id).await?;
let current_version = issue.version;
// 2. Build patch with same-client mutation
let now = Utc::now();
let patched = Issue { id: self.target_id, version: current_version, ... };
let untainted = sanitize(Tainted::new(patched), ServerMetadata { ... });
let start = Instant::now();
// 3. PATCH with explicit version — triggers 409 on stale version
match self.backend.update_issue(&self.project, self.target_id, untainted, Some(current_version)).await {
    Ok(_) => metrics.record(OpKind::Patch, elapsed_us(start)),   // win
    Err(err) => {
        metrics.record(OpKind::Patch, elapsed_us(start));
        metrics.record_error(ErrorKind::classify(&err));           // 409 = Conflict — expected
    }
}
```

**Error recording pattern** (lines 143–149 of sim_direct.rs):
```rust
Err(err) => {
    metrics.record(OpKind::Patch, elapsed_us(start));
    metrics.record_error(ErrorKind::classify(&err));
}
```
`ErrorKind::Conflict` is the expected outcome for losing clients. The summary must NOT fail if Conflict count > 0 — that proves If-Match is working.

**Elapsed helper** (lines 156–159):
```rust
fn elapsed_us(start: Instant) -> u64 {
    u64::try_from(start.elapsed().as_micros()).unwrap_or(u64::MAX)
}
```
Copy verbatim — identical in `sim_direct.rs` and `confluence_direct.rs`.

---

### `crates/reposix-swarm/tests/contention_e2e.rs` (test, request-response)

**Analog:** `crates/reposix-swarm/tests/mini_e2e.rs`

**File-level boilerplate** (lines 1–36):
```rust
#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]
#![allow(clippy::missing_panics_doc)]

use std::time::Duration;
use std::path::PathBuf;

use reposix_core::http::{client, ClientOpts};
use reposix_sim::{run_with_listener, SimConfig};
use reposix_swarm::driver::{run_swarm, SwarmConfig};
// + ContentionWorkload import
use tempfile::NamedTempFile;
```

**`spawn_sim` helper** (lines 54–90 of mini_e2e.rs) — copy verbatim:
```rust
async fn spawn_sim(rate_limit_rps: u32) -> (String, NamedTempFile, tokio::task::JoinHandle<()>) {
    let db = NamedTempFile::new().expect("tempfile");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let addr = listener.local_addr().expect("local_addr");
    let base_url = format!("http://{addr}");
    let cfg = SimConfig { bind: addr, db_path: db.path().to_owned(), seed: true,
        seed_file: Some(seed_fixture()), ephemeral: false, rate_limit_rps };
    let handle = tokio::spawn(async move { let _ = run_with_listener(listener, cfg).await; });
    // poll /healthz (40 × 25ms = 1s)
    let http = client(ClientOpts::default()).expect("http client");
    for _ in 0..40 {
        if http.get(format!("{base_url}/healthz")).await.map(|r| r.status().is_success()).unwrap_or(false) {
            return (base_url, db, handle);
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    panic!("sim failed to come up at {base_url} within 1s");
}
```

**`audit_row_count` helper** (lines 92–98 of mini_e2e.rs) — copy verbatim:
```rust
fn audit_row_count(path: &std::path::Path) -> rusqlite::Result<i64> {
    let conn = rusqlite::Connection::open(path)?;
    conn.query_row("SELECT COUNT(*) FROM audit_events", [], |row| row.get(0))
}
```

**Test function pattern** (lines 100–177 of mini_e2e.rs):
```rust
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn contention_swarm_50_clients_30s_deterministic_409() {
    let (base, db, sim_handle) = spawn_sim(200).await;
    let db_path = db.path().to_owned();
    let cfg = SwarmConfig { clients: 50, duration: Duration::from_secs(30),
        mode: "contention", target: &base };
    let markdown = run_swarm(cfg, |i| {
        ContentionWorkload::new(base.clone(), "demo".to_string(), target_id, u64::try_from(i).unwrap_or(0))
    }).await.expect("run_swarm");
    // Assertions follow mini_e2e pattern but with Conflict-class check:
    assert!(!markdown.contains("| Other"), "Other errors = transport bug");
    // ... (see RESEARCH.md §Pattern 1 for assertion spec)
    sim_handle.abort();
    let _ = sim_handle.await;
}
```
Note: keep `#[tokio::test(flavor = "multi_thread", worker_threads = 4)]` — the contention test needs real OS threads, not cooperative multitasking.

---

### `crates/reposix-swarm/tests/chaos_audit.rs` (test, event-driven)

**Analog:** `crates/reposix-swarm/tests/mini_e2e.rs` (spawn/healthz/audit patterns)

**Key divergence:** chaos test spawns sim as a real child process via `std::process::Command` (NOT `tokio::spawn`) so `child.kill()` sends SIGKILL:

```rust
// Use env!("CARGO_BIN_EXE_reposix-sim") — idiomatic Cargo pattern for binary path
let sim_bin = env!("CARGO_BIN_EXE_reposix-sim");

let db = NamedTempFile::new().expect("tempfile");
let mut child = std::process::Command::new(sim_bin)
    .args(["--db", db.path().to_str().unwrap(), "--bind", "127.0.0.1:7979"])
    .spawn()
    .expect("spawn sim");
```

**Healthz poll pattern** — adapt from mini_e2e.rs but with timeout:
```rust
// poll with 5s cap; fail fast if sim doesn't come up (DB may be corrupt after kill)
let http = client(ClientOpts::default()).expect("http client");
let deadline = Instant::now() + Duration::from_secs(5);
loop {
    if http.get("http://127.0.0.1:7979/healthz").await.map(|r| r.status().is_success()).unwrap_or(false) {
        break;
    }
    assert!(Instant::now() < deadline, "sim failed to come up within 5s after restart");
    tokio::time::sleep(Duration::from_millis(50)).await;
}
```

**Kill pattern:**
```rust
child.kill().expect("kill sim"); // SIGKILL on Unix per Rust stdlib docs
let _ = child.wait(); // reap zombie
```

**WAL integrity assertion** — use `audit_row_count` but add field-integrity query:
```rust
// Assert no torn rows (NULL in non-nullable fields) — NOT that all rows survived
let conn = rusqlite::Connection::open(db.path()).expect("open after kill");
let bad: i64 = conn.query_row(
    "SELECT COUNT(*) FROM audit_events WHERE op IS NULL OR entity_id IS NULL OR ts IS NULL",
    [],
    |row| row.get(0),
).expect("integrity query");
assert_eq!(bad, 0, "torn rows (NULL fields) found after kill-9: {bad}");
```

**Test gate:** tag `#[ignore]` and env-var guard:
```rust
#[tokio::test]
#[ignore = "chaos: requires reposix-sim binary; set REPOSIX_CHAOS_TEST=1"]
async fn chaos_kill9_no_torn_rows() {
    if std::env::var("REPOSIX_CHAOS_TEST").is_err() {
        eprintln!("SKIP: set REPOSIX_CHAOS_TEST=1 to run");
        return;
    }
    // ...
}
```

---

### `crates/reposix-confluence/src/lib.rs` (service, CRUD — modify)

**Self-modification — no external analog needed.**

**Current truncation site** (lines 763–770):
```rust
while let Some(url) = next_url.take() {
    pages += 1;
    if pages > (MAX_ISSUES_PER_LIST / PAGE_SIZE) {
        tracing::warn!(
            pages,
            "reached MAX_ISSUES_PER_LIST cap; stopping pagination"
        );
        break;  // ← becomes Err in strict mode
    }
    // ...
    if out.len() >= MAX_ISSUES_PER_LIST {
        return Ok(out);  // ← also becomes Err in strict mode
    }
}
```

**Add `list_issues_strict` as a concrete method on `ConfluenceBackend`** — NOT on the `IssueBackend` trait (see RESEARCH.md Pitfall 4). Pattern: extract the pagination loop into a private helper that takes a `bool` parameter; `list_issues` calls it with `strict = false`, `list_issues_strict` calls it with `strict = true`.

**Tenant URL redaction** — for lines 781–784 where the full URL is included in error strings:
```rust
// Current (leaks tenant):
return Err(Error::Other(format!(
    "confluence returned {status} for GET {url}: {}",
    String::from_utf8_lossy(&bytes)
)));
// Fix: log only path+query, not full URL:
use url::Url;
let safe = Url::parse(&url).map(|u| u.path().to_string()).unwrap_or_else(|_| "<url parse error>".to_string());
return Err(Error::Other(format!(
    "confluence returned {status} for GET {safe}: {}",
    String::from_utf8_lossy(&bytes)
)));
```

**`ConfluenceCreds` debug-redaction pattern** (lines 123–130) — existing; maintain this for any new credential-like fields:
```rust
impl std::fmt::Debug for ConfluenceCreds {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConfluenceCreds")
            .field("email", &self.email)
            .field("api_token", &"<redacted>")
            .finish()
    }
}
```

---

### `crates/reposix-swarm/src/main.rs` (binary, request-response — modify)

**Self-modification. Analog: existing `Mode` enum + match arm pattern** (lines 25–153).

**Mode enum extension pattern** (lines 23–32):
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
#[clap(rename_all = "kebab_case")]
enum Mode {
    SimDirect,
    ConfluenceDirect,
    Fuse,
    // ADD: Contention,
    // ADD: Chaos,
}
```

**Match arm factory pattern** (lines 104–116):
```rust
Mode::SimDirect => {
    let origin = args.target.clone();
    let project = args.project.clone();
    run_swarm(cfg, |i| {
        SimDirectWorkload::new(
            origin.clone(),
            project.clone(),
            u64::try_from(i).unwrap_or(0),
        )
    }).await?
}
```
Copy this pattern for `Mode::Contention` — add a `target_id` arg (`--target-issue`) and pass it to `ContentionWorkload::new`.

---

### `crates/reposix-cli/src/main.rs` (CLI — modify)

**Self-modification. Analog: existing `Cmd::Mount` struct field pattern** (lines 65–94+).

**Adding a flag to an existing subcommand** — follow the existing `#[arg(long, ...)]` decoration pattern on the `list`-equivalent subcommand:
```rust
// Existing pattern from Mount subcommand (lines 65+):
Mount {
    mount_point: PathBuf,
    #[arg(long, default_value = "http://127.0.0.1:7878")]
    origin: String,
    ...
}
// Apply the same style to add --no-truncate to the List subcommand:
List {
    ...
    /// Error instead of silently capping at 500 pages (Confluence only).
    #[arg(long)]
    no_truncate: bool,
}
```

---

### `.github/workflows/ci.yml` (config — modify)

**Self-modification. Analog: existing `integration` job** (lines 55–78).

**Current integration job structure** (lines 55–78):
```yaml
integration:
  name: integration (mounted FS)
  runs-on: ubuntu-latest
  needs: [test]
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
    - uses: Swatinem/rust-cache@v2
    - name: Install runtime FUSE binaries
      run: sudo apt-get update && sudo apt-get install -y fuse3
    - name: Verify FUSE available
      run: |
        ls -l /dev/fuse
        which fusermount3
    - name: Build release binaries
      run: cargo build --release --workspace --bins
    - name: Run integration tests (requires FUSE)
      run: cargo test -p reposix-fuse --release --locked -- --ignored --test-threads=1
```

**macOS matrix extension pattern** (add `strategy.matrix.os`, conditionalise steps):
```yaml
integration:
  name: integration (mounted FS)
  strategy:
    matrix:
      os: [ubuntu-latest, macos-14]   # pin macos-14 (Sonoma) per RESEARCH pitfall 3
  runs-on: ${{ matrix.os }}
  needs: [test]
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
    - uses: Swatinem/rust-cache@v2
    - name: Install FUSE (Linux)
      if: runner.os == 'Linux'
      run: sudo apt-get update && sudo apt-get install -y fuse3
    - name: Install macFUSE
      if: runner.os == 'macOS'
      uses: gythialy/macfuse@v1
    - name: Verify FUSE available (Linux)
      if: runner.os == 'Linux'
      run: ls -l /dev/fuse && which fusermount3
    - name: Build release binaries
      run: cargo build --release --workspace --bins
    - name: Run integration tests (requires FUSE)
      env:
        REPOSIX_UNMOUNT_CMD: ${{ runner.os == 'macOS' && 'umount -f' || 'fusermount3 -u' }}
      run: cargo test -p reposix-fuse --release --locked -- --ignored --test-threads=1
```

**Hooks CI step** (small addition to an existing job, e.g. `test` or a new `hooks` job):
```yaml
- name: Test pre-push credential hook
  run: bash scripts/hooks/test-pre-push.sh
```
Follow the naming style of existing steps: lowercase, no trailing period, action verb first.

**Existing env block** (lines 10–15) — carry forward unchanged:
```yaml
env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1
  CARGO_INCREMENTAL: 0
  RUSTFLAGS: "-D warnings"
```

---

## Shared Patterns

### Workload trait implementation
**Source:** `crates/reposix-swarm/src/workload.rs` (entire file, 35 lines)
**Apply to:** `contention.rs`
```rust
#[async_trait]
pub trait Workload: Send + Sync {
    async fn step(&self, metrics: &Arc<MetricsAccumulator>) -> anyhow::Result<()>;
}
```
Every workload struct must implement this trait. `Send + Sync` is required because `Arc<dyn Workload>` is shared across tokio tasks.

### MetricsAccumulator record pattern
**Source:** `crates/reposix-swarm/src/metrics.rs` lines 130–144
**Apply to:** `contention.rs`
```rust
// Record success:
metrics.record(OpKind::Patch, elapsed_us(start));
// Record error (non-fatal — workload continues):
metrics.record_error(ErrorKind::classify(&err));
```
Always record the latency for both success AND error paths. `ErrorKind::Conflict` is expected in the contention workload — it must not fail the `Other`-only assertion.

### Swarm driver `run_swarm` factory pattern
**Source:** `crates/reposix-swarm/src/driver.rs` lines 40–75
**Apply to:** `main.rs` new `Mode::Contention` arm
```rust
run_swarm(cfg, |i| {
    SomeWorkload::new(
        origin.clone(),
        project.clone(),
        u64::try_from(i).unwrap_or(0),
    )
}).await?
```
The factory closure receives `i: usize` (client index); convert to `u64` seed with `u64::try_from(i).unwrap_or(0)`.

### spawn_sim + /healthz poll pattern
**Source:** `crates/reposix-swarm/tests/mini_e2e.rs` lines 54–90
**Apply to:** `contention_e2e.rs` (in-process), `chaos_audit.rs` (child process variant)
```rust
// In-process (contention test): tokio::spawn + run_with_listener
// Child process (chaos test): std::process::Command::new(env!("CARGO_BIN_EXE_reposix-sim"))
// Poll: 40 × 25ms for normal; 100 × 50ms (5s cap) for chaos restart
```

### audit_row_count query
**Source:** `crates/reposix-swarm/tests/mini_e2e.rs` lines 92–98
**Apply to:** `contention_e2e.rs`, `chaos_audit.rs`
```rust
fn audit_row_count(path: &std::path::Path) -> rusqlite::Result<i64> {
    // Open R/W (not RO) — WAL-mode sim keeps rows in WAL until checkpoint
    let conn = rusqlite::Connection::open(path)?;
    conn.query_row("SELECT COUNT(*) FROM audit_events", [], |row| row.get(0))
}
```

### tracing::warn! pattern (no tenant URL)
**Source:** `crates/reposix-confluence/src/lib.rs` lines 565–568
**Apply to:** all new warn sites in lib.rs
```rust
tracing::warn!(
    wait_secs = wait,
    "Confluence rate limit — backing off until retry-after"
    // NOTE: no URL in this warn — tenant-safe by design
);
```
New error/warn sites added during the `--no-truncate` work must NOT include the full request URL. Use `url.path_and_query()` from `reqwest::Url` if a path is needed for debugging.

### `#[forbid(unsafe_code)]` + `#[warn(clippy::pedantic)]` header
**Source:** Every crate's top-level (e.g., `crates/reposix-swarm/src/lib.rs` lines 15–17)
**Apply to:** all new `.rs` files
```rust
#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]
```

### `#[ignore]` + env-var guard for slow/chaos tests
**Source:** `crates/reposix-confluence/tests/contract.rs` — `contract_confluence_live` test (line ~120+, uses `skip_if_no_env!` macro)
**Apply to:** `chaos_audit.rs`
```rust
#[tokio::test]
#[ignore = "chaos: requires reposix-sim binary; set REPOSIX_CHAOS_TEST=1"]
async fn chaos_kill9_no_torn_rows() {
    if std::env::var("REPOSIX_CHAOS_TEST").is_err() {
        eprintln!("SKIP: set REPOSIX_CHAOS_TEST=1 to run chaos tests");
        return;
    }
    // ...
}
```

---

## No Analog Found

All files have analogs in this codebase. No new dependencies are required.

| File | Role | Data Flow | Note |
|------|------|-----------|------|
| — | — | — | All patterns are covered by existing codebase analogs |

---

## Metadata

**Analog search scope:** `crates/reposix-swarm/`, `crates/reposix-confluence/`, `crates/reposix-cli/`, `.github/workflows/`, `scripts/hooks/`
**Files scanned:** 12 source files read in full
**Key If-Match path** verified at: `crates/reposix-sim/src/routes/issues.rs` lines 370–381 (from RESEARCH.md §Code Examples — not re-read; pattern confirmed by research)
**Pattern extraction date:** 2026-04-15
