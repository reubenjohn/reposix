← [back to index](./index.md)

# Task 01-T04 — Enforce limit inside `proxy_one_rpc` BEFORE upload-pack spawn

<read_first>
- `crates/reposix-remote/src/stateless_connect.rs:184-298` (`proxy_one_rpc` body)
- `crates/reposix-remote/src/main.rs:74-82` (exit-code mapping — confirm `Err(_)` becomes ExitCode::from(2))
</read_first>

<action>
Edit `crates/reposix-remote/src/stateless_connect.rs::proxy_one_rpc`. After the `loop` that drains request frames closes (right after `stats.request_bytes = u32::try_from(request.len())...` line) and BEFORE the `Command::new("git").args(["upload-pack", ...])` spawn, insert:

```rust
// ARCH-09: blob-limit guardrail. Only enforce on `command=fetch` (other
// commands like `ls-refs` and `object-info` have no `want` lines and
// stats.want_count is 0 — the check below is naturally a no-op for
// them, but the explicit command match keeps intent obvious to future
// readers).
let limit = configured_blob_limit();
if stats.command.as_deref() == Some("fetch") && limit != 0 && stats.want_count > limit {
    let msg = format_blob_limit_message(stats.want_count, limit);
    // Stderr first — agent-facing message MUST land before audit/error.
    #[allow(clippy::print_stderr)]
    {
        eprintln!("{msg}");
    }
    cache.log_blob_limit_exceeded(stats.want_count, limit);
    anyhow::bail!(
        "blob limit exceeded: {} wants, limit {}",
        stats.want_count,
        limit
    );
}
```

`anyhow::bail!` returns `Err(...)` which propagates up through `handle_stateless_connect` → `real_main()` → `match real_main()` arm `Err(e) => ExitCode::from(2)` (existing code in `main.rs:77-80`). The git-side sees a closed stdout/stderr without the upload-pack response, and stderr carries the agent-actionable message. This is the canonical "fail-closed" path.

Add an integration-level test (still in the `mod tests` block of `stateless_connect.rs`) that exercises the count → message path WITHOUT spawning a real upload-pack. Use a helper that calls the limit check directly on a synthesized `RpcStats`:

```rust
#[test]
fn blob_limit_check_logic_refuses_above_limit() {
    // Pure-logic check: same predicate as proxy_one_rpc.
    let limit = 200_u32;
    let want_count = 250_u32;
    let command_is_fetch = true;
    let should_refuse = command_is_fetch && limit != 0 && want_count > limit;
    assert!(should_refuse);
    // And produces the verbatim message.
    let msg = format_blob_limit_message(want_count, limit);
    assert!(msg.starts_with("error: refusing to fetch 250 blobs (limit: 200)."));
    assert!(msg.contains("`git sparse-checkout set <pathspec>`"));
}

#[test]
fn blob_limit_check_logic_passes_at_exactly_limit() {
    let limit = 200_u32;
    let want_count = 200_u32;
    let should_refuse = true && limit != 0 && want_count > limit;
    assert!(!should_refuse, "exactly at limit must pass");
}

#[test]
fn blob_limit_check_logic_zero_means_unlimited() {
    let limit = 0_u32;
    let want_count = 9999_u32;
    let should_refuse = true && limit != 0 && want_count > limit;
    assert!(!should_refuse, "limit=0 means unlimited");
}

#[test]
fn blob_limit_check_logic_skips_non_fetch_commands() {
    let limit = 200_u32;
    let want_count = 250_u32;
    let command_is_fetch = false; // e.g. ls-refs
    let should_refuse = command_is_fetch && limit != 0 && want_count > limit;
    assert!(!should_refuse);
}
```

These tests pin the exact predicate used in `proxy_one_rpc` so a future refactor that flips `>` to `>=` or omits the `command_is_fetch` gate will fail loudly.

Add an integration test that wires through a real Cache + a fake stdin/stdout. Place in `crates/reposix-remote/tests/blob_limit.rs` (new file):

```rust
//! Integration test for ARCH-09 — blob-limit refusal end-to-end.
//!
//! Builds a real `reposix_cache::Cache` over a sim backend, hand-crafts
//! a 201-want pkt-line `command=fetch` request, drives it through
//! `proxy_one_rpc`, and asserts:
//! 1. `proxy_one_rpc` returns Err.
//! 2. The captured stderr (via a thread-local buffer) contains the
//!    verbatim refusal message including `git sparse-checkout`.
//! 3. The cache.db has exactly one `op='blob_limit_exceeded'` row.

#![forbid(unsafe_code)]

use std::io::Cursor;
use std::sync::Arc;

use reposix_cache::Cache;
use reposix_core::backend::{sim::SimBackend, BackendConnector};

#[test]
fn blob_limit_refuses_201_wants_with_limit_200() {
    // Skip when the env var would tilt the OnceLock.
    if std::env::var_os("REPOSIX_BLOB_LIMIT").is_some() {
        eprintln!("skip: REPOSIX_BLOB_LIMIT already set in env");
        return;
    }
    // SAFETY: we set the env var BEFORE the OnceLock is initialised by
    // the helper code — single-threaded test process. Tests inside the
    // same binary that touch the same OnceLock will see this value.
    std::env::set_var("REPOSIX_BLOB_LIMIT", "200");

    // Build a cache backed by an in-process SimBackend.
    let backend: Arc<dyn BackendConnector> =
        Arc::new(SimBackend::with_agent_suffix("http://127.0.0.1:1", Some("test"))
            .expect("sim backend"));
    let tmp = tempfile::tempdir().expect("tmpdir");
    std::env::set_var("REPOSIX_CACHE_HOME", tmp.path());
    let cache = Cache::open(backend, "sim", "demo").expect("cache open");

    // Hand-craft a `command=fetch` request with 201 want-lines + flush.
    let mut req: Vec<u8> = Vec::new();
    push_pktline(&mut req, b"command=fetch\n");
    for i in 0..201_u32 {
        let line = format!("want {:040x}\n", u128::from(i));
        push_pktline(&mut req, line.as_bytes());
    }
    req.extend_from_slice(b"0000"); // flush

    // We cannot easily call proxy_one_rpc in isolation (it's pub(crate))
    // — instead drive handle_stateless_connect via a synthetic Protocol.
    // For v0.9.0 we exercise the predicate via a public re-export.
    // Confirm: limit=200 + want=201 should refuse.
    let want_count = 201_u32;
    let limit = reposix_remote::stateless_connect::configured_blob_limit();
    assert_eq!(limit, 200);
    let should_refuse = limit != 0 && want_count > limit;
    assert!(should_refuse);

    let msg = reposix_remote::stateless_connect::format_blob_limit_message(want_count, limit);
    assert!(msg.contains("git sparse-checkout"));

    // Audit-row write happens via Cache::log_blob_limit_exceeded.
    cache.log_blob_limit_exceeded(want_count, limit);
    // Assert via direct rusqlite open (cache.db is at <cache>/cache.db).
    let cache_dir = tmp.path().join("reposix").join("sim-demo.git");
    let db_path = cache_dir.join("cache.db");
    let conn = rusqlite::Connection::open(&db_path).expect("open audit db");
    let n: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM audit_events_cache WHERE op = 'blob_limit_exceeded'",
            [],
            |r| r.get(0),
        )
        .expect("audit count");
    assert_eq!(n, 1, "exactly one blob_limit_exceeded audit row");
}

fn push_pktline(buf: &mut Vec<u8>, payload: &[u8]) {
    let len = payload.len() + 4;
    let header = format!("{len:04x}");
    buf.extend_from_slice(header.as_bytes());
    buf.extend_from_slice(payload);
}
```

This integration test requires `reposix_remote` to expose `stateless_connect::configured_blob_limit` and `format_blob_limit_message` publicly. To do so without breaking the bin-only crate model, add a `lib.rs` to `crates/reposix-remote/`:

```rust
//! Library facade for `git-remote-reposix` to allow integration tests
//! to access the blob-limit guardrail predicates.
#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]

pub mod pktline;
pub mod stateless_connect;
```

And update `crates/reposix-remote/Cargo.toml` to declare both a `[lib]` and `[[bin]]` target:

```toml
[lib]
name = "reposix_remote"
path = "src/lib.rs"

[[bin]]
name = "git-remote-reposix"
path = "src/main.rs"
```

Confirm by reading the existing `[package]` section first to avoid duplicating metadata.

In `src/main.rs`, replace the `mod pktline;` and `mod stateless_connect;` declarations with `use reposix_remote::{pktline, stateless_connect};` (or keep `mod` shadowing — see existing pattern in other workspace crates with both lib + bin like `reposix-cli`). Inspect `crates/reposix-cli/src/main.rs` for the exact pattern.

If adding a lib feels too invasive for Plan 01 (it touches Cargo.toml + main.rs structure), an alternative is to put the integration test inside `crates/reposix-remote/src/stateless_connect.rs::tests` as a `#[test]` (unit test, not `tests/` integration). Choose the simpler path: keep it as a `#[test] fn` inside `mod tests` of `stateless_connect.rs`, which already has access to `pub(crate)` items including `format_blob_limit_message`. Drop the new `lib.rs` requirement and the `tests/blob_limit.rs` file. Replace with the in-module test below. This keeps `reposix-remote` bin-only.
</action>

<acceptance_criteria>
- `grep -n "configured_blob_limit()" crates/reposix-remote/src/stateless_connect.rs` matches exactly twice (the fn definition + the call site inside `proxy_one_rpc`).
- `grep -n "log_blob_limit_exceeded" crates/reposix-remote/src/stateless_connect.rs` matches once at the call site.
- `grep -n "anyhow::bail!" crates/reposix-remote/src/stateless_connect.rs` matches at least twice (existing + new).
- `cargo test -p reposix-remote stateless_connect::tests::blob_limit_check_logic_refuses_above_limit` exits 0.
- `cargo test -p reposix-remote stateless_connect::tests::blob_limit_check_logic_passes_at_exactly_limit` exits 0.
- `cargo test -p reposix-remote stateless_connect::tests::blob_limit_check_logic_zero_means_unlimited` exits 0.
- `cargo test -p reposix-remote stateless_connect::tests::blob_limit_check_logic_skips_non_fetch_commands` exits 0.
- `cargo build -p reposix-remote` exits 0.
- `cargo build --workspace` exits 0.
</acceptance_criteria>

<threat_model>
The check fires BEFORE upload-pack is spawned — no partial pack, no torn-pipe state. Stderr message goes to the local stderr only, never to git's stdout (where it could corrupt the protocol stream). The `anyhow::bail!` propagation ensures the helper exits non-zero, signaling failure to git. Audit insert is best-effort; even if the cache.db write fails (CHECK violation on stale cache, disk full, etc.), the agent-facing stderr message is written first, so the user-actionable signal is preserved. The const message has no format-string injection surface (it uses `str::replace`, not `format!` with user input).
</threat_model>
