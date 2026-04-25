---
phase: 32
slug: stateless-connect-capability-in-git-remote-reposix-read-path
status: approved
nyquist_compliant: true
wave_0_complete: true
created: 2026-04-24
---

# Phase 32 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | `cargo test` (Rust stable 1.82+), `assert_cmd` + `tempfile` for integration, `wiremock` for backend mocks |
| **Config file** | `crates/reposix-remote/Cargo.toml` (existing); no new harness |
| **Quick run command** | `cargo test -p reposix-remote --lib` |
| **Full suite command** | `cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings` |
| **Gated integration** | `cargo test -p reposix-remote --test stateless_connect -- --ignored` (requires git >= 2.27) |
| **Estimated runtime** | ~45 s for full suite (workspace), ~4 s for quick |

---

## Sampling Rate

- **After every task commit:** `cargo check -p reposix-remote` (<10s).
- **After every plan wave:** `cargo test -p reposix-remote && cargo clippy -p reposix-remote --all-targets -- -D warnings`.
- **Before phase verifier:** Full workspace suite green.
- **Max feedback latency:** 45 s (full workspace test).

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---|---|---|---|---|---|---|---|---|---|
| 32-01-01 | 01 | 1 | ARCH-04 | — | pkt-line frame round-trips (flush/delim/response-end/data) | unit | `cargo test -p reposix-remote --lib pktline` | W0 | pending |
| 32-01-02 | 01 | 1 | ARCH-05 gotcha 1 | — | initial advertisement ends with flush only, no `0002` | unit | `cargo test -p reposix-remote initial_advertisement_ends_with_flush_only` | W0 | pending |
| 32-01-03 | 01 | 1 | ARCH-05 gotcha 2 | — | RPC response appends `0002` | unit | `cargo test -p reposix-remote rpc_response_appends_response_end` | W0 | pending |
| 32-01-04 | 01 | 1 | ARCH-05 gotcha 3 | — | binary stdin round-trips NUL + non-UTF-8 bytes | unit | `cargo test -p reposix-remote stdin_is_binary_throughout` | W0 | pending |
| 32-01-05 | 01 | 1 | ARCH-04 | — | capabilities advertises `stateless-connect`, `object-format=sha1`, refspec | unit | `cargo test -p reposix-remote capability_advertisement_lists_stateless_connect` | W0 | pending |
| 32-01-06 | 01 | 1 | ARCH-04 | — | refspec namespace is `refs/reposix/*` | unit | `cargo test -p reposix-remote refspec_namespace_is_reposix` | W0 | pending |
| 32-02-01 | 02 | 2 | ARCH-04 | Audit | `handle_stateless_connect` opens cache + runs `build_from` + writes `helper_connect` audit row | unit | `cargo test -p reposix-remote helper_connect_writes_audit_row` | ❌ W0 | pending |
| 32-02-02 | 02 | 2 | ARCH-04 | Audit | `helper_fetch` audit row records want_count, request_bytes, response_bytes | unit | `cargo test -p reposix-remote helper_fetch_records_want_count` | ❌ W0 | pending |
| 32-02-03 | 02 | 2 | ARCH-04 | — | helper spawns `upload-pack --stateless-rpc` against cache.repo_path() | unit | `cargo test -p reposix-remote invokes_upload_pack_on_cache_path` | ❌ W0 | pending |
| 32-02-04 | 02 | 2 | ARCH-04 | — | `State.last_fetch_want_count` incremented per RPC (wired but not enforced) | unit | `cargo test -p reposix-remote want_counter_is_wired` | ❌ W0 | pending |
| 32-03-01 | 03 | 3 | ARCH-04 | — | `git clone --filter=blob:none` against sim: tree present, blobs missing | integration (gated) | `cargo test -p reposix-remote --test stateless_connect -- --ignored partial_clone_against_sim_is_lazy` | ❌ W0 | pending |
| 32-03-02 | 03 | 3 | ARCH-04 | Audit | `git cat-file -p` lazy-fetches exactly once per OID (idempotent) | integration (gated) | `cargo test -p reposix-remote --test stateless_connect -- --ignored lazy_fetch_idempotent` | ❌ W0 | pending |
| 32-03-03 | 03 | 3 | ARCH-04 | — | Existing export push path green (regression) | existing | `cargo test -p reposix-remote --test protocol` | OK | pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [x] `crates/reposix-remote/tests/protocol.rs` — existing regression suite; must not break.
- [ ] `crates/reposix-remote/tests/stateless_connect.rs` — new unit + integration tests (created by Plans 01, 02, 03).
- [x] `cargo test` already installed (workspace Rust toolchain in `rust-toolchain.toml`).

*Wave 0 setup: plans add test files alongside feature code; no separate Wave 0 installation step needed.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Real `git clone --filter=blob:none` end-to-end | ARCH-04 | Dev-host git is 2.25.1 (no v2 filter); must run on git >= 2.27 (alpine container or CI with apt-installed `git` 2.34+) | `docker run --rm -v $PWD:/w alpine:latest sh -c 'apk add --quiet git && cd /w && cargo test -p reposix-remote --test stateless_connect -- --ignored'` |
| Capture Rust-port trace log | OP-1 | One-time snapshot for `rust-port-trace.log` ground-truth artifact | After Plan 03: run the gated clone test with `RUST_LOG=debug tracing` enabled, capture stderr to `.planning/research/v0.9-fuse-to-git-native/rust-port-trace.log`, commit. |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 45 s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** approved 2026-04-24 (phase-runner)
