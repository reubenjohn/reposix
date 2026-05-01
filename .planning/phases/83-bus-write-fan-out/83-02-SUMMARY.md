# 83-02 Plan Summary — Fault injection + audit completeness + phase close

Plan 2 of 2 in phase 83. Four sequential tasks, five atomic commits. Phase verifier graded GREEN at end.

## Tasks shipped (4/4)

**T01 — Catalog-first.** Four `agent-ux` rows minted with `status: FAIL` BEFORE Rust:
- `agent-ux/bus-write-fault-injection-mirror-fail`
- `agent-ux/bus-write-fault-injection-sot-mid-stream`
- `agent-ux/bus-write-fault-injection-post-precheck-409`
- `agent-ux/bus-write-audit-completeness`

**T02 — Mirror-fail integration test (Q-D `#[cfg(unix)]`).** `tests/bus_write_mirror_fail.rs` exercises documented failure case (a): wiremock SoT succeeds, file:// mirror push fails (failing-update-hook returns exit 1). Asserts SoT-state-correct, mirror-lag audit row written, `refs/mirrors/<sot>-head` updated, `synced-at` NOT updated, `ok` returned to git. **CRITICAL fixture bug fixed inline (Rule 1 + SURPRISES-INTAKE entry):** `make_failing_mirror_fixture` set `hooks/update` chmod 0o755 but didn't override `core.hooksPath` — host's `~/.gitconfig` `core.hooksPath = ~/.git-hooks` overrode the per-repo hook. Fix: explicit `git config core.hooksPath <bare>/hooks` on the bare repo.

**T03 — SoT-fail tests.** `tests/bus_write_sot_fail.rs` (mid-stream 5xx) + `tests/bus_write_post_precheck_409.rs` (post-precheck 409). Cases (b) and (c). Each asserts: no SoT writes succeeded after fail, no mirror push, helper exits non-zero with clear error.

**T04 — Audit-completeness test + flip + close.** `tests/bus_write_audit_completeness.rs` asserts dual-table audit per OP-3:
- `audit_events_cache` direct query: `helper_backend_instantiated >= 1`, `helper_push_started == 1`, `helper_push_accepted == 1`, `mirror_sync_written == 1`, `helper_push_partial_fail_mirror_lag == 0`
- `audit_events` (wiremock request log proxy for SimBackend wire path): exactly 1 PATCH for the 1 update_record action; 0 unexpected POST/DELETE.

Catalog flip: 5 rows (4 P83-02 + 1 held-FAIL P83-01) → PASS. CLAUDE.md fault-injection addendum. `git push origin main`.

## Commits

| SHA | Subject |
|---|---|
| `c5175cd` | quality(catalog): mint 4 P83-02 fault-injection rows + 4 TINY verifiers (catalog-first) |
| `4bbd38c` | test(remote): mirror-fail integration test with `#[cfg(unix)]` failing-update-hook fixture (DVCS-BUS-WRITE-06 a) |
| `6f3742e` | test(remote): SoT-fail tests — mid-stream 5xx + post-precheck 409 (DVCS-BUS-WRITE-06 b+c) |
| `d8699b6` | test(remote): bus_write_audit_completeness.rs dual-table audit assertion |
| `bd903bb` | quality(agent-ux): flip 5 P83 rows FAIL→PASS + CLAUDE.md addendum (phase 83 complete) |
| `fc46415` (CI fix-forward) | fix(cli): broaden dark_factory_conflict_teaching_string_present to write_loop.rs |
| `cf81824` (CI fix-forward) | fix(remote): per-test cache_dir isolation in push_conflict tests |

## In-phase deviations (eager-resolution per OP-8)

1. **`make_failing_mirror_fixture` core.hooksPath bug** — fix-forward in T02; SURPRISES-INTAKE filed (RESOLVED inline per OP-8 honesty trail).
2. **Plan-prose minor correction** — T03 catalog row asserted "exactly 1 list_changed_since" for post-precheck-409; actual count is 2 (bus_handler PRECHECK B + write_loop L1 precheck). Catalog row's expected.asserts updated.
3. **Audit-events queryability** — bus_write tests use wiremock instead of spawning real `reposix-sim`; sim's `audit_events` SQLite table not directly queryable. Asserted at the wire boundary (wiremock request log = byte-equivalent to sim's audit middleware writes).

## Post-close CI fix-forwards

- **fc46415:** `dark_factory_conflict_teaching_string_present` test (`crates/reposix-cli/tests/agent_flow.rs`) asserted `git pull --rebase` lives in `main.rs`; P83-01 T02 lifted it to `write_loop.rs`. Broadened test to scan main.rs + write_loop.rs + bus_handler.rs.
- **cf81824:** `push_conflict.rs` tests (3 of 3) failed coverage step under `cargo llvm-cov --workspace` (passed regular `cargo test`). Root cause: shared host-level `REPOSIX_CACHE_DIR` (default ~/.cache/reposix) carried state across tests. Fix: per-test tempdir + `.env("REPOSIX_CACHE_DIR", ...)` on subprocess (mirror_refs.rs / bus_write_happy.rs precedent).

## Acceptance

- All 8 P83 catalog rows PASS at phase close.
- All 6 DVCS-BUS-WRITE-* requirements shipped, observable test coverage.
- Phase verifier GREEN at `quality/reports/verdicts/p83/VERDICT.md` (commit SHAs verified: 3857f9a, 76cf527, 836dc6f, 6978369, b2d18cc, 4b7be9d, c5175cd, 4bbd38c, 6f3742e, d8699b6, bd903bb, fc46415).
- CLAUDE.md updated in-phase (QG-07).
- 1 SURPRISES-INTAKE entry filed (fixture-fix; RESOLVED).
- Pre-push 26 PASS / 0 FAIL.
