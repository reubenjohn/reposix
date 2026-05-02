# Phase 32 — Research: `stateless-connect` capability in `git-remote-reposix` (read path)

**Date:** 2026-04-24
**Requirements:** ARCH-04, ARCH-05
**Source artifacts consulted:**
- `.planning/phases/32-.../32-CONTEXT.md` (locked decisions)
- `.planning/research/v0.9-fuse-to-git-native/architecture-pivot-summary.md` §3
- `.planning/research/v0.9-fuse-to-git-native/partial-clone-remote-helper-findings.md`
- `.planning/research/v0.9-fuse-to-git-native/poc/git-remote-poc.py`
- `.planning/research/v0.9-fuse-to-git-native/poc/poc-helper-trace.log`
- `.planning/phases/31-.../31-VERIFICATION.md` (Phase 31 cache API)
- `crates/reposix-remote/src/main.rs`, `src/protocol.rs`

---

## Chapters

1. **[Protocol & Gotchas](./ch1-protocol.md)** — What we are building (Tunnel pattern), wire protocol bytes (helper handshake, protocol-v2 RPC loop, pkt-line frame format), and the three locked regression-test gotchas.

2. **[Rust Port Plan](./ch2-rust-port.md)** — Module layout (`pktline.rs`, `stateless_connect.rs`, `main.rs` dispatch), binary stdin discipline (gotcha 3 detail), and cache bridge wiring (`Cache::build_from`, blob pre-materialization).

3. **[Audit Logging & Tests](./ch3-audit-tests.md)** — Audit logging surface (OP-3 rows), and the binary integration test harness (smoke, full-clone, sparse-batching, push regression).

4. **[Decisions, Manifest & Close](./ch4-decisions.md)** — Port-specific idiomatic-Rust decisions vs POC, POC bugs to NOT port, file manifest, sizing estimate, threat-model touch points, open questions, and RESEARCH COMPLETE statement.
