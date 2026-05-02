# Phase 32 Research — Decisions, Manifest & Close

← [back to index](./index.md)

## 9. Port-specific decisions — idiomatic Rust over POC semantics

| POC (Python) | Rust port |
|---|---|
| `subprocess.run(["git", "upload-pack", ...], capture_output=True, env=env)` | `std::process::Command::new("git").args(...).env("GIT_PROTOCOL","version=2").output()?` — synchronous is fine; this is inside the single-threaded runtime and blocks anyway. Avoid `tokio::process::Command` unless async buys something. |
| `sys.stdin.buffer.read(1)` byte-by-byte | `BufReader::read_until(b'\n', ...)` and `read_exact` — no per-byte reads. |
| `proc.communicate()` footgun | `Command::output()` in Rust does the right thing. |
| `STDOUT.write(b"0002")` | `proto.send_raw(b"0002")` via existing method. |
| `log()` to stderr + optional file | `tracing::debug!` (existing subscriber writes to stderr via `with_writer(std::io::stderr)`). No env-var log file — audit log is the persistence layer. |

---

## 10. POC bugs to NOT port

From `push-path-stateless-connect-findings.md`:

1. **Empty-delta refspec bug** (fixed in POC and existing helper):
   `refs/heads/*:refs/heads/*` → must be `refs/heads/*:refs/reposix/*`.
   The existing main.rs already uses `refs/reposix/*`. Regression test
   asserts this.

2. **`line.startswith("commit ")`** — not relevant to stateless-connect
   (fast-export parsing bug; Phase 32 doesn't touch `fast_import.rs`).

3. **Python `proc.communicate()` after `stdin.close()`** — not
   applicable to Rust's `Command::output()`.

---

## 11. File manifest (what gets touched)

- **New:** `crates/reposix-remote/src/pktline.rs`
- **New:** `crates/reposix-remote/src/stateless_connect.rs`
- **Edit:** `crates/reposix-remote/src/main.rs` — add capability lines,
  dispatch arm, `State.backend_name`, `State.cache`,
  `State.last_fetch_want_count`.
- **Edit:** `crates/reposix-remote/src/protocol.rs` — add
  `read_exact_bytes` method (or expose inner BufReader).
- **Edit:** `crates/reposix-remote/Cargo.toml` — add
  `reposix-cache = { path = "../reposix-cache" }` dependency.
- **Edit:** `crates/reposix-cache/src/audit.rs` — add
  `log_helper_connect`, `log_helper_advertise`, `log_helper_fetch`
  helpers (one-liner wrappers around existing `log_event`).
- **New:** `crates/reposix-remote/tests/stateless_connect.rs` — unit
  tests for the three gotchas + capability advertisement.
- **New (gated):** integration test at the same file or sibling,
  `#[cfg_attr(not(feature = "integration-git"), ignore)]`.
- **New:** `.planning/research/v0.9-fuse-to-git-native/rust-port-trace.log`
  — captured from an actual run of the Rust helper (OP-1 feedback-loop
  artifact).

---

## 12. Sizing estimate

- `pktline.rs`: ~120 LOC + ~60 LOC tests.
- `stateless_connect.rs`: ~200 LOC + ~80 LOC tests.
- `main.rs` edits: ~30 LOC added.
- `protocol.rs` edits: ~15 LOC added.
- `audit.rs` edits: ~30 LOC added.
- Integration test: ~120 LOC.

Total new code: ~ 650 LOC. Fits CONTEXT.md "~200 lines" estimate for
the core tunnel, plus pktline lib + tests.

---

## 13. Threat-model touch points (per CLAUDE.md "Threat model")

- **Outbound HTTP allowlist.** Not newly triggered in this phase — the
  helper doesn't open HTTP in the read path; the cache backend does,
  and it already enforces `REPOSIX_ALLOWED_ORIGINS`.
- **No shell escape from FUSE writes.** Not applicable (we're deleting
  FUSE); `upload-pack` input bytes flow through a `Command` with no
  shell.
- **Tainted-by-default.** All response bytes from `upload-pack` are
  tainted (they're derived from REST responses). The helper writes
  them to stdout which flows to git, which stores them in `.git/objects`
  — the working tree ends up materializing tainted content. This is
  the intended design: the mount point IS a git checkout of tainted
  data.
- **Audit log append-only.** Phase 31 triggers already enforce. No
  change needed.

---

## 14. Open questions (deferred, not blocking)

1. Should we pre-warm `Cache::build_from` lazily or eagerly on helper
   startup? **Decision:** eagerly in `handle_stateless_connect` only
   (not on every invocation of the helper — `capabilities`,
   `list`, and `export` don't need the bare repo's refs to be current).
2. Multi-repo concurrency — two `git` processes invoking the helper
   simultaneously. Phase 31's cache uses a SQLite WAL + `gix` which is
   process-safe. Not revisiting here.
3. `upload-pack` binary discovery — assume `git` is on `PATH`. The
   `reposix init` command (Phase 35) will document the requirement.

---

## RESEARCH COMPLETE

Deliverables: three gotchas locked with named tests, cache-bridge flow
documented, file manifest sized, port-specific idiomatic-Rust decisions
recorded, POC bugs identified for NO-PORT. Ready for planning.
