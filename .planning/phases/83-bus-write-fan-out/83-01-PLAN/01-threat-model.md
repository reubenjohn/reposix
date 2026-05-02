← [back to index](./index.md) · phase 83 plan 01

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

---

## Task 83-01-T01 — Catalog-first: mint 4 catalog rows + author 4 verifier shells
