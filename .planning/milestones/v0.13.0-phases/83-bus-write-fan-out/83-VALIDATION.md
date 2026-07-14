---
phase: 83
artifact: validation
nyquist_validation: true
source: ".planning/phases/83-bus-write-fan-out/83-RESEARCH.md § Validation Architecture (lines 858–899)"
purpose: "Nyquist-validation check artifact for Phase 83 — promoted to standalone file per PLAN-CHECK.md H2 (workflow.nyquist_validation expects a phase-local 83-VALIDATION.md sibling)."
---

# Phase 83 — Validation Architecture (Nyquist check artifact)

> Promoted verbatim from `83-RESEARCH.md § Validation Architecture`
> per PLAN-CHECK.md HIGH H2. Workflow gates with
> `nyquist_validation: true` look for a phase-local
> `<phase>-VALIDATION.md` sibling; this file is that sibling. Source
> of truth remains the research document — keep both in sync if the
> validation contract evolves during P83 execution.

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
| DVCS-BUS-WRITE-01 | SoT-first write + dual audit | integration | `cargo test -p reposix-remote --test bus_write_happy happy_path_writes_both_refs_and_acks_ok` | NO — T05 / P83a |
| DVCS-BUS-WRITE-02 | Mirror-fail returns ok with lag | integration | `cargo test -p reposix-remote --test bus_write_mirror_fail bus_write_mirror_fail_returns_ok_with_lag_audit_row` | NO — P83b T02 |
| DVCS-BUS-WRITE-03 | Mirror-success updates synced-at + ok | integration | `cargo test -p reposix-remote --test bus_write_happy happy_path_writes_both_refs_and_acks_ok` (combined) | NO — T05 / P83a |
| DVCS-BUS-WRITE-04 | No helper-side retry | mechanical | `bash quality/gates/agent-ux/bus-write-no-helper-retry.sh` (greps source) | NO — T01 / P83a |
| DVCS-BUS-WRITE-05 | No-mirror-remote regression | integration | `cargo test -p reposix-remote --test bus_write_no_mirror_remote bus_write_no_mirror_remote_emits_q35_hint` | NO — T05 / P83a |
| DVCS-BUS-WRITE-06 | Three fault scenarios + audit completeness | integration | `cargo test -p reposix-remote --test 'bus_write_*'` | NO — P83b |

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
