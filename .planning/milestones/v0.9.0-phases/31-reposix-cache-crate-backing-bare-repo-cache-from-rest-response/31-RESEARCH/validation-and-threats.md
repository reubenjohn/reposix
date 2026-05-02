← [back to index](./index.md)

# Validation Architecture and Threat Model Delta

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | `cargo test` (Rust standard) + `trybuild` 1.0.116 for compile-fail |
| Config file | `crates/reposix-cache/Cargo.toml` (no separate config — Rust convention) |
| Quick run command | `cargo test -p reposix-cache` |
| Full suite command | `cargo test --workspace` (includes regression check on all other crates) |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| ARCH-01 | `Cache::build_from(backend)` produces a valid bare repo with a tree containing all N issue paths | integration | `cargo test -p reposix-cache --test tree_contains_all_issues` | ❌ Wave A |
| ARCH-01 | After `build_from`, `.git/objects/` contains tree + commit but NO blob objects | integration | `cargo test -p reposix-cache --test blobs_are_lazy` | ❌ Wave A |
| ARCH-01 | `Cache::read_blob(oid)` materializes exactly one blob and returns the expected bytes | integration | `cargo test -p reposix-cache --test materialize_one` | ❌ Wave A or B |
| ARCH-02 | Each `read_blob` call writes one `op="materialize"` audit row; N reads → N rows | integration | `cargo test -p reposix-cache --test materialize_one -- --include-ignored audit_count` | ❌ Wave B |
| ARCH-02 | `Cache::read_blob` returns `Tainted<Vec<u8>>` | unit | `cargo test -p reposix-cache --test materialize_one -- type_signature` | ❌ Wave A |
| ARCH-02 | `egress::send(cache.read_blob(oid).unwrap())` does NOT compile | compile-fail | `cargo test -p reposix-cache --test compile_fail` | ❌ Wave C |
| ARCH-02 | `UPDATE audit_events_cache SET ts=0` and `DELETE FROM audit_events_cache` both fail with SQLITE_CONSTRAINT | integration | `cargo test -p reposix-cache --test audit_is_append_only` | ❌ Wave B |
| ARCH-03 | Pointing the cache at an origin not in `REPOSIX_ALLOWED_ORIGINS` returns `Error::Egress(InvalidOrigin)` | integration | `cargo test -p reposix-cache --test egress_denied_logs` | ❌ Wave B |
| ARCH-03 | A denied egress writes an `op="egress_denied"` audit row | integration | (same test as above, asserts row count) | ❌ Wave B |
| ARCH-03 | No `reqwest::Client::new` / `ClientBuilder::new` call in `crates/reposix-cache/src/`. Workspace clippy lint already configured. | clippy | `cargo clippy -p reposix-cache --all-targets -- -D warnings` | ✅ existing `clippy.toml` |
| ARCH-03 | Cache uses `reposix_core::http::client()` exclusively (verifiable by absence of any `reqwest::` import) | grep / clippy | `! grep -rn "reqwest::Client" crates/reposix-cache/src/` | ❌ add to verify-work checklist |

### Sampling Rate

- **Per task commit:** `cargo check -p reposix-cache && cargo clippy -p reposix-cache --all-targets -- -D warnings` (≤ 30 s typically)
- **Per wave merge:** `cargo test -p reposix-cache` (all integration + compile-fail tests)
- **Phase gate:** `cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings` (no regression in 318+ existing tests)

### Wave 0 Gaps

- [ ] `crates/reposix-cache/Cargo.toml` — new crate manifest (Wave A)
- [ ] `crates/reposix-cache/src/lib.rs` + module skeleton — Wave A
- [ ] `crates/reposix-cache/fixtures/cache_schema.sql` — Wave B
- [ ] `crates/reposix-cache/tests/compile_fail.rs` + `tests/compile-fail/tainted_blob_into_egress.{rs,stderr}` — Wave C
- [ ] Workspace `Cargo.toml` `members` array gains `crates/reposix-cache` — Wave A
- [ ] Workspace `Cargo.toml` `[workspace.dependencies]` gains `gix = "=0.82.0"` and `dirs = "6"` — Wave A

(The framework itself — `cargo test`, `trybuild` — is already in use across the workspace; no install step needed.)

## Threat Model Delta

This phase adds an on-disk cache populated from REST responses. The cache is NOT a new exfiltration surface — and the planner needs to be able to articulate why.

### Why the cache is not a new exfiltration surface

| Threat | Pre-cache | With cache | Net change |
|--------|-----------|------------|------------|
| Tainted bytes flow into outbound HTTP | FUSE callback returns issue body bytes; nothing prevents another component from re-POSTing them. Mitigated by frontmatter sanitize + allowlist. | `Cache::read_blob` returns `Tainted<Vec<u8>>`. The trybuild compile-fail fixture in this phase mechanically blocks `egress::send(tainted)` at the type level. | **Reduced.** The cache narrows where tainted bytes can be born (only `read_blob`), and the type discipline catches misuse at compile time. |
| Cache file readable by other local users | N/A (no cache pre-v0.9.0) | Cache lives at `$XDG_CACHE_HOME/reposix/...`. Default umask gives 0700 on the directory; SQLite file inherits default 0644 unless we mode-set. | **NEW THREAT.** Mitigation: open `cache.db` with `mode=0o600` (lifted pattern from `cache_db.rs:71`). Verify in test. |
| Cache poisoning by a malicious backend | Pre-cache: malicious backend response renders to FUSE bytes returned to agent. | With cache: same malicious bytes get committed into the bare repo. `git diff` would surface them — auditable. | **No worse.** The cache is just a stable place for bytes that were already attacker-influenced. The git history makes them MORE auditable, not less. |
| Egress to non-allowlisted origin | `reposix_core::http::client()` rejects with `Error::InvalidOrigin`. | Same — cache calls through the same factory. PLUS: the rejection is now audited (`op=egress_denied` row), which is a NEW capability. | **Improved.** Was silent-rejected error → now audited. |
| Audit-log tampering | `audit_events` in simulator's DB has BEFORE UPDATE/DELETE triggers + DEFENSIVE flag. | `audit_events_cache` in this crate's DB will have the SAME hardening (lifted pattern). | **No change.** Same hardening, two tables. |
| Cache file as a covert channel | N/A | An attacker who can write to the cache directory can plant arbitrary commits. But: same attacker can plant anything in any directory they control. The cache is no different from `~/.cache/anything-else`. | **No new attack.** OS-level file permissions are the only defence, and they're sufficient (0700 dir). |

### CLAUDE.md Threat-Model Table — what changes after this phase

The current CLAUDE.md threat-model table says:

> | Exfiltration | `git push` can target arbitrary remotes; the FUSE daemon makes outbound HTTP. |

After this phase: outbound HTTP is no longer FUSE's job. It's the cache's. The CLAUDE.md table needs an update in Phase 36 (per ARCH-14) — but this phase introduces the change. Recommendation: include a CHANGELOG `[Unreleased]` entry noting "outbound HTTP surface migrated from FUSE daemon to `reposix-cache`; allowlist enforcement unchanged."

### What this phase does NOT cover (explicit deferrals)

- **Push path.** This phase is read-only against the backend (no `create_issue` / `update_issue` / `delete_or_close`). Phase 34 covers push — the place where Tainted-vs-Untainted matters operationally.
- **Threat-model document update.** `research/threat-model-and-critique.md` revision is in architecture-pivot-summary §7 open question 3 and is deferred per CONTEXT.md.
- **Concurrent writers.** Single-writer per cache is the v0.9.0 contract. Phase 33 will introduce the lock.

### Single mandatory hardening checklist for this phase

- [ ] `cache.db` opens with `mode=0o600`
- [ ] `audit_events_cache` has `BEFORE UPDATE` and `BEFORE DELETE` triggers raising ABORT
- [ ] Cache's SQLite handle has `SQLITE_DBCONFIG_DEFENSIVE` set (mirror `reposix_core::audit::open_audit_db`)
- [ ] Egress denial writes `op=egress_denied` audit row BEFORE returning the typed error
- [ ] `Tainted<Vec<u8>>` is the public return type of `read_blob` (asserted by trybuild compile-fail)
- [ ] Zero `reqwest::Client::*` constructors in the new crate (asserted by clippy `disallowed_methods`)
