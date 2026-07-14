← [back to index](./index.md)

# Phase 83 Plan 01 — Context: Canonical Refs + Threat Model

<canonical_refs>
**Spec sources:**
- `.planning/REQUIREMENTS.md` DVCS-BUS-WRITE-01 / DVCS-BUS-WRITE-02 /
  DVCS-BUS-WRITE-03 / DVCS-BUS-WRITE-04 / DVCS-BUS-WRITE-05 (lines
  74-78) — verbatim acceptance.
- `.planning/ROADMAP.md` § Phase 83 (lines 145-165) — phase goal +
  8 success criteria.
- `.planning/research/v0.13.0-dvcs/architecture-sketch.md` § "3. Bus
  remote with cheap-precheck + SoT-first-write" (lines 83-146) —
  bus algorithm steps 4-9 (P83 scope).
- `.planning/research/v0.13.0-dvcs/decisions.md` Q2.3 (both refs on
  success), Q3.6 (no helper-side retry).
- `.planning/phases/83-bus-write-fan-out/83-RESEARCH.md` — full
  research bundle (especially § Architecture Patterns Pattern 1 +
  Pattern 2 + Pattern 3, § Common Pitfalls 1-7, § Catalog Row
  Design, § Mirror-Lag Audit Row Shape, § Audit Completeness
  Contract).
- `.planning/phases/83-bus-write-fan-out/83-PLAN-OVERVIEW.md` §
  "Decisions ratified at plan time" (D-01..D-10).

**`apply_writes` lift substrate (T02):**
- `crates/reposix-remote/src/main.rs::handle_export` lines 343-606
  — the verbatim source body lifted into `write_loop::apply_writes`.
  Three replacements per S1: state.cache.as_ref → cache;
  state.backend.as_ref → backend; state.push_failed = true; return
  Ok(()) → return Ok(WriteOutcome::<variant>).
- `crates/reposix-remote/src/precheck.rs::precheck_export_against_changed_set`
  (lines 90-311) — narrow-deps signature donor pattern (P81).
- `crates/reposix-remote/src/diff.rs::plan` + `PlanError` — pure
  function reused verbatim by `apply_writes`.
- `crates/reposix-remote/src/main.rs::execute_action` (line 619+)
  — pure function reused verbatim by `apply_writes`.
- `crates/reposix-remote/src/main.rs::diag` (line 90) +
  `pub(crate) fn ensure_cache` (line 256) — already widened to
  `pub(crate)` in P82; reused as-is.
- `crates/reposix-remote/src/main.rs::fail_push` (line 283) — local
  helper. NOT widened; bus_handler's `bus_fail_push` clone (P82
  shipped) is the bus-side companion.

**Cache audit op (T03):**
- `crates/reposix-cache/src/audit.rs` lines 200-280
  (`log_helper_push_started`, `log_helper_push_accepted`,
  `log_helper_push_rejected_conflict`) — sibling pattern donors;
  the new `log_helper_push_partial_fail_mirror_lag` follows the
  same shape.
- `crates/reposix-cache/src/cache.rs` lines 232-302 (`Cache::`
  wrappers for the helper-push audit fns) — sibling pattern donor.
- `crates/reposix-cache/src/mirror_refs.rs::Cache::log_mirror_sync_written`
  (line 274) — alternative `Cache::` wrapper donor (this one lives
  in `mirror_refs.rs` because it composes ref names; the new
  partial-fail wrapper goes in `cache.rs` because it does not).
- `crates/reposix-cache/fixtures/cache_schema.sql` lines 11-48 —
  schema CHECK list (the comment on lines 22-27 cites P79
  `attach_walk` + P80 `mirror_sync_written`; P83 extension cites
  the new op).
- `crates/reposix-cache/src/audit.rs` `#[cfg(test)] mod tests` —
  unit test home for the new helper.

**Bus handler write fan-out (T04):**
- `crates/reposix-remote/src/bus_handler.rs` lines 172-174 (current
  `emit_deferred_shipped_error` invocation) — replacement site.
- `crates/reposix-remote/src/bus_handler.rs::resolve_mirror_remote_name`
  (line 180) — already shipped P82; the `mirror_remote_name`
  produced by this function is consumed by `push_mirror` in T04.
- `crates/reposix-remote/src/bus_handler.rs::precheck_mirror_drift`
  (line 245) — donor pattern for the `Command::new("git").args(...)`
  shell-out idiom + cwd-inheritance behavior.
- `crates/reposix-remote/src/protocol.rs` — `Protocol`,
  `ProtoReader`, `send_line`, `send_blank`, `flush` shapes.
- `crates/reposix-remote/src/fast_import.rs::parse_export_stream`
  — stdin parser reused verbatim.
- `crates/reposix-cache/src/mirror_refs.rs::Cache::write_mirror_synced_at`
  + `Cache::write_mirror_head` + `Cache::log_mirror_sync_written`
  + `Cache::refresh_for_mirror_head` (P80) — caller-side ref/audit
  helpers.

**Test fixtures (T05):**
- `crates/reposix-remote/tests/mirror_refs.rs` (P80) — helper-driver
  donor pattern for `bus_write_happy.rs`.
- `crates/reposix-remote/tests/bus_precheck_b.rs` (P82) —
  `make_synced_mirror_fixture` donor for the file:// bare mirror
  with passing update hook; reused by `bus_write_happy.rs`.
- `crates/reposix-remote/tests/bus_precheck_a.rs` (P82) —
  `bus_no_remote_configured_emits_q35_hint` donor pattern for
  `bus_write_no_mirror_remote.rs`.
- `crates/reposix-remote/tests/common.rs` (existing — `sample_issues`,
  `sim_backend`) — append `make_failing_mirror_fixture` (P83-02
  consumes) + `count_audit_cache_rows` (P83-02 consumes).
- `crates/reposix-remote/Cargo.toml` `[dev-dependencies]` —
  `wiremock`, `assert_cmd`, `tempfile`, `rusqlite` already present.

**Quality Gates:**
- `quality/catalogs/agent-ux.json` — existing file with 12 P82 rows
  post-P82 close; the 4 new P83-01 rows join.
- `quality/gates/agent-ux/bus-url-parses-query-param-form.sh` (P82)
  + `bus-precheck-b-sot-drift-emits-fetch-first.sh` (P82) — TINY
  verifier precedents (~30-50 line shape; delegate to `cargo test
  -p reposix-remote --test <name>`).
- `quality/PROTOCOL.md` § "Verifier subagent prompt template" + §
  "Principle A".

**Operating principles:**
- `CLAUDE.md` § "Build memory budget" — strict serial cargo,
  per-crate fallback.
- `CLAUDE.md` § "Push cadence — per-phase" — terminal push
  protocol.
- `CLAUDE.md` § Operating Principles OP-1 (simulator-first), OP-2
  (Tainted-by-default — no new tainted byte source in P83-01),
  OP-3 (audit log unchanged in shape — new op extends the existing
  table), OP-7 (verifier subagent), OP-8 (+2 reservation).
- `CLAUDE.md` § "Threat model" — `<threat_model>` section below
  enumerates the new shell-out boundary's STRIDE register.

This plan introduces ONE new shell-out construction site
(`git push <mirror_remote_name> main` for the mirror-best-effort
write). The `BackendConnector::list_changed_since` call in P81's
precheck (reused by `apply_writes`) goes through the existing
`client()` factory + `REPOSIX_ALLOWED_ORIGINS` allowlist (no new
HTTP construction site). See `<threat_model>` below for the STRIDE
register.
</canonical_refs>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| helper → `git push <mirror_remote_name> main` shell-out (NEW) | `Command::new("git").args(["push", mirror_remote_name, "main"]).output()`. The `mirror_remote_name` is helper-resolved from `git config --get-regexp '^remote\..+\.url$'` (P82's STEP 0); NOT user-controlled at this point — it was already validated by git when the user ran `git remote add`. Trust direction: helper-controlled call site, value bounded by git's own remote-name validation. Defensive-in-depth mitigation: reject `mirror_remote_name` whose first byte is `-` BEFORE the shell-out (T-83-01). |
| `git push` stderr output → audit row (NEW operator-readable seam) | The mirror-push subprocess's stderr is captured (3-line tail) and stored in `audit_events_cache` row's `reason` field. The stderr is git-controlled (commit SHAs, ref names, hook output) — could leak repo-internal info to operators reading the audit table. Trust direction: git-controlled output, operator-readable storage. Mitigation: trim to 3 lines (T-83-02). |
| Stdin (fast-import stream) → `apply_writes` → SoT REST writes | UNCHANGED — `parse_export_stream` produces `Tainted<Record>`; `execute_action`'s `sanitize(Tainted::new(issue), meta)` boundary is preserved verbatim by the lift. NO new tainted-bytes seam. |
| Cache audit row INSERT (`audit_events_cache`) | UNCHANGED schema; new op `helper_push_partial_fail_mirror_lag` extends the CHECK list. Stale cache.db files WARN-log on INSERT-fail; fresh caches accept. NO new SQL injection surface (rusqlite parameterized query). |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-83-01 | Tampering | `git push` shell-out (`bus_handler::push_mirror`) — argument injection via `mirror_remote_name` (defense-in-depth; the value is helper-resolved, not user-controlled at this boundary) | mitigate | Reject `mirror_remote_name` whose first byte is `-` BEFORE `Command::new("git").args(["push", ...])`. The reject error: *"mirror_remote_name cannot start with `-`: <name>"*. NO `--` separator needed because `mirror_remote_name` is bounded by git's own remote-name validation (the value passed git's `remote.<name>.url` format check at `git remote add` time). Verifier: T05's `bus_write_happy.rs` includes the happy path; the defensive reject is asserted via a unit-style test in `bus_handler.rs`'s `#[cfg(test)] mod tests` (added in T04 if cargo permits in the same task; otherwise inline in T05). Code review checkpoint: `crates/reposix-remote/src/bus_handler.rs` is grepped for the `starts_with('-')` reject at the `push_mirror` site BEFORE merge. |
| T-83-02 | Information Disclosure | `git push` stderr_tail captured for audit row + stderr WARN | mitigate | Trim to 3 lines via `String::from_utf8_lossy(&out.stderr).lines().rev().take(3).collect::<Vec<_>>().join(" / ")`. The 3-line bound is documented in `audit.rs::log_helper_push_partial_fail_mirror_lag` doc comment AND in `bus_handler.rs::push_mirror` doc comment. Verifier: T05's `bus_write_no_mirror_remote.rs` (regression check on the trimming) — N/A, that test exercises the no-remote path. Trim is exercised in P83-02's `bus_write_mirror_fail.rs`; P83-01 documents but does not test the trim. |
| T-83-03 | Repudiation | Partial-fail (SoT-success-mirror-fail) end-state could be misread as full success | mitigate | TWO-fold defense: (a) `helper_push_partial_fail_mirror_lag` audit row records the SoT SHA + exit code + stderr tail — operators see partial-fail in the audit table; (b) the `head ≠ synced-at` invariant on the refs side gives a vanilla-`git`-only operator a way to detect lag without database access (`git log refs/mirrors/<sot>-synced-at -1` shows the timestamp is older than the cache's most recent commit). |
| T-83-04 | Denial of Service | `git push` against private mirrors hangs on SSH-agent prompt | accept | Documented in CLAUDE.md update (T06). Tests use `file://` fixture exclusively. Same disposition as T-82-03 (P82). If SSH prompts become a real production issue, future work could pass `GIT_TERMINAL_PROMPT=0` env var — filed as v0.14.0 GOOD-TO-HAVE candidate, not P83-01 scope. |
| T-83-05 | Tampering | Confluence non-atomicity across actions (PATCH 1 succeeds, PATCH 2 fails — partial SoT state observable) | accept | RESEARCH.md Pitfall 3 + D-09. The recovery story is next-push reads new SoT state via PRECHECK B's `list_changed_since` and either accepts the local change (if version still matches) or rejects with conflict. Documented in `bus_handler.rs` module-doc + CLAUDE.md (T06). Test contract for P83-02 `bus_write_sot_fail.rs`: assert the EXACT partial state — id=1 has new version, id=2 unchanged, no audit row for ids subsequent to id=2, mirror unchanged. |

No new HTTP origin in scope (`apply_writes` reuses the existing
`BackendConnector` allowlist). No new `Tainted<T>` propagation
path — P83-01 preserves the existing `parse_export_stream` →
`Tainted<Record>` → `sanitize` boundary verbatim.
</threat_model>
