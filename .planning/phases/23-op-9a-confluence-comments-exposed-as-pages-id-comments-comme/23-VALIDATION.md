---
phase: 23
slug: op-9a-confluence-comments
status: approved
nyquist_compliant: true
wave_0_complete: true
created: 2026-04-16
revised: 2026-04-16
---

# Phase 23 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Covers `reposix-confluence` (Plan 01), `reposix-cli` (Plan 02), and
> `reposix-fuse` (Plan 03).

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `cargo test` (built-in) + `wiremock` 0.6 (HTTP stubs, already in workspace) + `tokio::test` (async) |
| **Config file** | `Cargo.toml` per-crate (no external runner config) |
| **Quick run command** | `cargo test -p <crate> <test-filter> -- --nocapture` |
| **Full suite command** | `cargo test --workspace 2>&1 \| tail -20` |
| **Estimated runtime** | ~30–60 s for full workspace (release-profile tests excluded) |
| **Extras** | `cargo clippy --workspace --all-targets -- -D warnings` + `cargo fmt --all --check` (gauntlet) |

All test frameworks and fixtures (`wiremock`, `MockServer`, `ResponseTemplate`,
`ConfluenceBackend::new_with_base_url`, `creds()` helper, labels-overlay test
patterns) already exist in the workspace — **no Wave 0 framework install
required**. The only Wave 0-ish work is net-new test *files* inside
`crates/reposix-confluence/src/lib.rs` and `crates/reposix-fuse/src/comments.rs`,
but those are created by Plan 01 Task 1 and Plan 03 Task 1 themselves (TDD-style
with `tdd="true"` task flag).

---

## Sampling Rate

- **After every task commit:** Run the task's `<automated>` command (scoped test filter).
- **After every plan wave:** Run `cargo test -p <crate>` (scoped to the crate just modified).
- **Before `/gsd-verify-work`:** Full workspace gauntlet green — `cargo test --workspace` + `cargo clippy --workspace --all-targets -- -D warnings` + `cargo fmt --all --check`.
- **Max feedback latency:** ≤ 60 s per full workspace run (target ≤ 15 s for scoped per-task runs).

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 23-01-01 | 01 | 1 | CONF-01, CONF-02 | T-23-01-01 / T-23-01-02 / T-23-01-03 / T-23-01-04 / T-23-01-06 | SSRF-safe pagination (relative-prepend); `redact_url` in error messages; 500-cap `tracing::warn!`; comment bodies never in log spans; rate-limit gate per GET | unit (wiremock) | `cargo test -p reposix-confluence list_comments -- --nocapture 2>&1 \| tail -40` | ✅ | ⬜ pending |
| 23-01-02 | 01 | 1 | SPACES-01 | T-23-01-02 / T-23-01-06 | `redact_url` in 5xx error messages; pagination cursors prepended to `self.base()` (SSRF-safe); rate-limit gate applies | unit (wiremock) | `cargo test -p reposix-confluence list_spaces -- --nocapture 2>&1 \| tail -30` | ✅ | ⬜ pending |
| 23-02-01 | 02 | 2 | SPACES-01 | T-23-02-01 / T-23-02-03 | `read_confluence_env` names-only error (no secret values); sim/github backends fast-fail with explicit error; tainted space names `println!`ed literally (ANSI escape risk accepted, documented) | unit (tokio) | `cargo check -p reposix-cli 2>&1 \| tail -20 && cargo test -p reposix-cli spaces:: 2>&1 \| tail -20` | ✅ | ⬜ pending |
| 23-02-02 | 02 | 2 | SPACES-01 | T-23-02-01 / T-23-02-04 | Clap subcommand registered; env-missing case returns names-only error mentioning `ATLASSIAN_API_KEY`; `redact_url` inherited via Plan 01's `list_spaces` | integration (cargo run) | `cargo build -p reposix-cli 2>&1 \| tail -20 && cargo run -q -p reposix-cli -- spaces --help 2>&1 \| head -20` | ✅ | ⬜ pending |
| 23-03-01 | 03 | 2 | CONF-01, CONF-02, CONF-03 | T-23-03-01 / T-23-03-02 / T-23-03-03 / T-23-03-06 | Body after `---\n\n` closing fence (no YAML re-entry); `is_numeric_ascii` rejects path-traversal ids; only numeric/kind fields in tracing spans; disjoint inode ranges asserted in the existing `fixed_inodes_are_disjoint_from_dynamic_ranges` test | unit | `cargo test -p reposix-fuse comments:: 2>&1 \| tail -40 && cargo test -p reposix-fuse fixed_inodes_are_disjoint_from_dynamic_ranges 2>&1 \| tail -10` | ✅ | ⬜ pending |
| 23-03-02 | 03 | 2 | CONF-01, CONF-02, CONF-03 | T-23-03-04 / T-23-03-05 / T-23-03-07 / T-23-03-08 / T-23-03-09 | `readdir(Bucket)` never emits `.comments` (DoS amplifier defused); `fetch_comments_for_page` wraps `list_comments` in `tokio::time::timeout(5s)` → EIO; `comment_fetcher.is_none()` early-ENOENT guard for sim/github; EROFS on every write/setattr/create/unlink over comment inodes | unit | `cargo check -p reposix-fuse 2>&1 \| tail -20 && cargo test -p reposix-fuse comments_dispatch_tests 2>&1 \| tail -20` | ✅ | ⬜ pending |
| 23-03-03 | 03 | 2 | CONF-01, CONF-02, CONF-03 | T-23-03-05 / T-23-03-09 (workspace integration) | `Mount::open` new 3-arg signature compiles across every caller; `build_comment_fetcher` re-reads env vars (fail-loud if changed); full workspace gauntlet green | integration (workspace) | `cargo build -p reposix-fuse 2>&1 \| tail -10 && cargo test --workspace 2>&1 \| tail -15 && cargo clippy --workspace --all-targets -- -D warnings 2>&1 \| tail -10` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

Every task's `<automated>` verify is present inline in the plan; no
`MISSING — Wave 0` stubs remain. Sampling continuity is preserved: no 3
consecutive tasks without an automated verify.

---

## Wave 0 Requirements

*Existing infrastructure covers all phase requirements.* `wiremock`,
`tokio::test`, `dashmap`, `tracing`, and the `ConfluenceBackend::new_with_base_url`
test constructor (+ `creds()` helper) are already present in the workspace
(inherited from Phases 16, 17, 19, 21). Plan 01 Task 1 and Plan 03 Task 1 are
both `tdd="true"` — they write the new tests alongside the production code.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Live FUSE mount `ls mount/pages/<padded-id>.comments/` + `cat` round-trip against a real Confluence tenant | CONF-01, CONF-02 | Requires a live FUSE mount bound to a live (or wiremock-Confluence-behind-a-live-FUSE) backend. Unit tests in `comments.rs` + dispatch tests in `fs.rs` cover the in-process contract deterministically; full end-to-end requires either `runtime/` mount test harness or a follow-up integration-test phase. | `cargo run -p reposix-fuse -- --backend confluence --project <SPACE_KEY> /tmp/reposix-mnt` then in another shell `ls /tmp/reposix-mnt/pages/00000000001.comments/` + `cat /tmp/reposix-mnt/pages/00000000001.comments/*.md`. Requires `ATLASSIAN_EMAIL`, `ATLASSIAN_API_KEY`, `REPOSIX_CONFLUENCE_TENANT`, and `REPOSIX_ALLOWED_ORIGINS` containing `https://<tenant>.atlassian.net`. |
| Tainted author name with ANSI escapes | T-23-02-03 | `println!` writes raw bytes to the terminal; testing requires a real terminal or TTY-aware harness. Disposition: *accept* for v0.7.0 (documented in Plan 02 threat model). | Create a test Confluence space whose name contains `\x1b[31m`; run `reposix spaces --backend confluence` in a terminal; confirm the escape either renders (expected) or pipe through `cat -v` to neutralise. |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies (every task in Plans 01/02/03 has an inline `<automated>` command).
- [x] Sampling continuity: no 3 consecutive tasks without automated verify.
- [x] Wave 0 covers all MISSING references (none — existing infrastructure is sufficient).
- [x] No watch-mode flags (all verify commands are one-shot `cargo test` / `cargo build`).
- [x] Feedback latency < 60 s (per-task scoped commands are typically ≤ 15 s).
- [x] `nyquist_compliant: true` set in frontmatter.

**Approval:** approved 2026-04-16 (revision after checker feedback 2026-04-16)
