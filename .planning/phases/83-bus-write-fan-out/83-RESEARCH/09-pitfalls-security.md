← [back to index](./index.md) · phase 83 research

## Pitfalls and Risks (summary, calling out the new risks beyond the per-pitfall section above)

- **Subprocess cwd assumption — pin with a test (Pitfall 6).** Document explicitly in `bus_handler.rs` module doc.
- **`--force-with-lease` cargo-cult from P84 (Pitfall 2).** Reject in code review.
- **Confluence non-atomicity across actions (Pitfall 3).** Document in P83 plan.
- **`synced-at` write order matters (Pitfall 1).** Always write head before synced-at.
- **Audit-row ordering on partial-fail (Pitfall 4).** Write audit row AFTER outcome known.
- **First-push case (Pitfall 5).** Add explicit happy-path test for empty cache + empty mirror.
- **Stale cache.db CHECK list (Pitfall 7).** Established pattern from P79/P80; no migration.

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | `cargo test` + `cargo nextest run` (existing) |
| Config file | `Cargo.toml` workspace; per-crate `[dev-dependencies]` already carry `wiremock`, `assert_cmd`, `tempfile` |
| Quick run command | `cargo test -p reposix-remote --test bus_write_<name> -- --nocapture` |
| Full suite command | `cargo nextest run -p reposix-remote -p reposix-cache` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|---|---|---|---|---|
| DVCS-BUS-WRITE-01 | SoT-first write + dual audit | integration | `cargo test -p reposix-remote --test bus_write_happy happy_path_writes_both_refs_and_acks_ok` | ❌ T05 / P83a |
| DVCS-BUS-WRITE-02 | Mirror-fail returns ok with lag | integration | `cargo test -p reposix-remote --test bus_write_mirror_fail bus_write_mirror_fail_returns_ok_with_lag_audit_row` | ❌ P83b T02 |
| DVCS-BUS-WRITE-03 | Mirror-success updates synced-at + ok | integration | `cargo test -p reposix-remote --test bus_write_happy happy_path_writes_both_refs_and_acks_ok` (combined) | ❌ T05 / P83a |
| DVCS-BUS-WRITE-04 | No helper-side retry | mechanical | `bash quality/gates/agent-ux/bus-write-no-helper-retry.sh` (greps source) | ❌ T01 / P83a |
| DVCS-BUS-WRITE-05 | No-mirror-remote regression | integration | `cargo test -p reposix-remote --test bus_write_no_mirror_remote bus_write_no_mirror_remote_emits_q35_hint` | ❌ T05 / P83a |
| DVCS-BUS-WRITE-06 | Three fault scenarios + audit completeness | integration | `cargo test -p reposix-remote --test 'bus_write_*'` | ❌ P83b |

### Sampling Rate
- **Per task commit:** `cargo test -p reposix-remote --test bus_write_<active_test>` for the test under change.
- **Per wave merge:** `cargo nextest run -p reposix-remote -p reposix-cache` (per-crate due to CLAUDE.md memory budget).
- **Phase gate:** Full workspace nextest GREEN before `git push origin main` and verifier subagent dispatch.

### Wave 0 Gaps

- [ ] `crates/reposix-remote/src/write_loop.rs` — new file (P83a T02 prelude refactor).
- [ ] `crates/reposix-cache/src/audit.rs` — extend with `log_helper_push_partial_fail_mirror_lag` (P83a T03).
- [ ] `crates/reposix-cache/src/mirror_refs.rs` — extend `impl Cache` with `log_helper_push_partial_fail_mirror_lag` wrapper (P83a T03).
- [ ] `crates/reposix-cache/fixtures/cache_schema.sql` — extend op CHECK list (P83a T03).
- [ ] `crates/reposix-remote/tests/common.rs` — append `make_failing_mirror_fixture` + `count_audit_cache_rows` (P83a T05 + P83b T02-T04).
- [ ] `crates/reposix-remote/tests/bus_write_happy.rs` — new file (P83a T05).
- [ ] `crates/reposix-remote/tests/bus_write_no_mirror_remote.rs` — new file (P83a T05).
- [ ] `crates/reposix-remote/tests/bus_write_mirror_fail.rs` — new file (P83b T02).
- [ ] `crates/reposix-remote/tests/bus_write_sot_fail.rs` — new file (P83b T03).
- [ ] `crates/reposix-remote/tests/bus_write_post_precheck_409.rs` — new file (P83b T03).
- [ ] `crates/reposix-remote/tests/bus_write_audit_completeness.rs` — new file (P83b T04).
- [ ] `quality/catalogs/agent-ux.json` — 8 new rows (4 in P83a T01, 4 in P83b T01).
- [ ] `quality/gates/agent-ux/bus-write-*.sh` — 8 new TINY verifier shells.

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---|---|---|
| V2 Authentication | no | Helper inherits `BackendConnector` auth from existing connectors (already V2-clean per v0.11.x audits). |
| V3 Session Management | no | No sessions; per-invocation. |
| V4 Access Control | yes | Egress allowlist (`REPOSIX_ALLOWED_ORIGINS`) gates SoT REST calls AND `git push <mirror>` shells out to a URL the user already configured locally (so the URL is not user-controllable at this point). |
| V5 Input Validation | yes | Stdin (fast-import stream) is `Tainted<>`; sanitize boundary preserved verbatim from `handle_export`. mirror_remote_name is helper-resolved (not user-controlled) but defensively reject `-`-prefix per § Pattern 2. |
| V6 Cryptography | no | No crypto introduced. |

### Known Threat Patterns for git remote helper + subprocess

| Pattern | STRIDE | Standard Mitigation |
|---|---|---|
| Argument injection via mirror_remote_name | Tampering | Reject `-`-prefixed names before `Command::new("git").args(["push", ...])`. mirror_remote_name comes from helper-resolved git config, not user argv, but defensive check matches T-82-01 idiom. |
| Working-tree cwd hijack via env | Elevation of Privilege | The `bus_handler` cwd is inherited from git's invocation; any test or future code path that spawns the helper from an unexpected cwd would land git pushes in the wrong tree. Mitigation: Pitfall 6 doc-warning + cwd assertion test. |
| Tainted stdin → mirror push amplification | Information Disclosure | Stdin bytes flow through `parse_export_stream` → `Tainted<Record>` → `sanitize` (existing seam) → REST PATCH. Mirror push doesn't see record bodies — it pushes git objects synthesized by the cache's bare repo, which themselves embed sanitized frontmatter. NEW exposure: mirror push subprocess's stderr_tail captured for the audit row. The stderr_tail comes from `git push` and is git-controlled (not record-content controlled), but the audit row is operator-readable and could leak repo-internal info. Trim to 3 lines (already in § Pattern 2). |
| Helper-side retry leaks transient state | Repudiation / Tampering | Q3.6 explicit no-retry + audit-the-attempt-once removes the leak vector. |
| Audit table CHECK constraint bypass | Tampering | Existing append-only triggers + DEFENSIVE flag (per `cache.rs:db.rs::open_cache_db`) cover this; no new attack surface. |

