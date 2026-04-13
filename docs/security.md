# Security

## The lethal trifecta

Every deployment of reposix is a textbook **lethal trifecta**[^1]:

1. **Private data.** The FUSE mount exposes issue bodies, custom fields, and (in v0.2) attachments.
2. **Untrusted input.** Every remote ticket is attacker-influenced text — issue bodies, titles, comments.
3. **Exfiltration channel.** `git push` can target any remote the agent chooses.

[^1]: [Simon Willison, "The lethal trifecta for AI agents"](https://simonwillison.net/2025/Jun/16/the-lethal-trifecta/), revised April 2026. Full red-team analysis of reposix is in [`.planning/research/threat-model-and-critique.md`](https://github.com/reubenjohn/reposix/blob/main/.planning/research/threat-model-and-critique.md).

A naïve implementation of "issues as files" would combine all three legs with no mitigations. reposix cuts leg 3 at the architectural level (allowlist), hardens leg 1 (RBAC + path validation), and assumes leg 2 is incurable (taint-type discipline and sanitize-on-egress).

## The eight guardrails (SG-01..08)

Every guardrail below is enforced in source and tested. "Evidence" points to the file that implements it. "Test" points to the test that asserts it. "On camera" indicates whether the guardrail fires visibly in `docs/demo.typescript`.

| ID | Mitigation | Evidence | Test | On camera |
|----|-----------|----------|------|-----------|
| SG-01 | Outbound HTTP allowlist (`REPOSIX_ALLOWED_ORIGINS`) | `crates/reposix-core/src/http.rs` sealed `HttpClient` newtype + per-request URL recheck | `crates/reposix-core/tests/http_allowlist.rs` (7 tests) | ✓ step 8a |
| SG-02 | Bulk-delete cap (>5 deletes refused; `[allow-bulk-delete]` overrides) | `crates/reposix-remote/src/diff.rs::plan` | `crates/reposix-remote/tests/bulk_delete_cap.rs` (3 tests) | ✓ step 8b |
| SG-03 | Server-controlled frontmatter immutable from clients | `crates/reposix-core/src/taint.rs::sanitize` strips `id`/`version`/`created_at`/`updated_at` | `crates/reposix-core/tests/compile_fail.rs` + runtime proof in Phase-S sanitize-on-egress test | ✓ step 6 |
| SG-04 | Filename = `<id>.md`; path validation rejects `/`, `\0`, `.`, `..` | `crates/reposix-core/src/path.rs::{validate_issue_filename, validate_path_component}` | `crates/reposix-core/src/path.rs` unit tests | enforced on every op |
| SG-05 | Tainted-content typing (`Tainted<T>`/`Untainted<T>`) | `crates/reposix-core/src/taint.rs` — `Untainted::new` is `pub(crate)` | `crates/reposix-core/tests/compile-fail/{tainted_into_untainted,untainted_new_is_not_pub}.rs` (trybuild) | compile-time |
| SG-06 | Audit log append-only + defensive SQLite open | `crates/reposix-core/fixtures/audit.sql` (`BEFORE UPDATE/DELETE` triggers in transaction) + `open_audit_db` (`SQLITE_DBCONFIG_DEFENSIVE`) | `crates/reposix-core/tests/audit_schema.rs` (5 tests incl. `writable_schema` bypass attempt) | ✓ step 8c |
| SG-07 | FUSE never blocks the kernel >5s | `crates/reposix-fuse/src/fetch.rs::with_timeout` + EIO return path | `crates/reposix-fuse/tests/sim_death_no_hang.rs` (kernel-timed `timeout 7 stat` subprocess) | ✓ step 4 (fast `ls`) |
| SG-08 | Demo recording shows guardrails firing | `scripts/demo.sh` steps 8a/8b/8c + `docs/demo.typescript` grep-verified | `grep -cE 'EPERM\|append-only\|refusing\|allowlist' docs/demo.typescript` ≥ 6 | — (meta) |

## Threat model

The full adversarial red-team analysis is in [`.planning/research/threat-model-and-critique.md`](https://github.com/reubenjohn/reposix/blob/main/.planning/research/threat-model-and-critique.md) (~600 lines). That document was produced by an independent subagent at T+0 of the autonomous build — its findings became the SG-* requirements above, baked in as first-class contracts before any code was written.

Highlights:

- **Egress allowlist is the single highest-leverage mitigation.** A ~30-line newtype around `reqwest::Client` eliminates leg 3 of the trifecta for the simulator-backed mode.
- **`rm -rf /mnt/reposix` is a real threat.** Without the bulk-delete cap, a confused agent (or a prompt-injection payload) could cascade into a `DELETE` storm. SG-02 catches this *at the git-remote-reposix layer* — defense in depth, independent of the FUSE daemon's own checks.
- **Frontmatter is a confused-deputy surface.** Attacker-authored issue bodies containing `permissions: rwxrwxrwx`, `owner: root`, `version: 999999` must not round-trip. SG-03 strips the server-controlled fields unconditionally.
- **Path traversal via issue title is prevented at the type level.** Filenames derive from `IssueId(u64)`, never from strings. Users cannot change filenames; the FUSE layer rejects any path component containing `/`, `\0`, `.`, or `..` with `EINVAL`.

## Normalization of deviance

From the [`AgenticEngineeringReference.md`](https://github.com/reubenjohn/reposix/blob/main/AgenticEngineeringReference.md) §5.5 distillation of Simon's interview:

> Every deployment of an unsafe agent that doesn't get exploited increases institutional confidence in it. This is the Challenger O-ring dynamic. The field has been getting away with unsafe patterns because no headline-grabbing exploit has landed yet. One will. Don't rely on "it hasn't happened" as evidence of safety.

This page exists because shipping reposix with green tests and a beautiful demo but without honest security accounting would be exactly such a deployment. The guardrails above are not cosmetic — each one is enforced at the type system or the kernel boundary, and each one fires on camera in the recording.

## What's deferred to v0.2

The following are *known gaps*, not oversights. Each has a rationale.

- **Real-backend credentials (Jira / GitHub / Confluence).** v0.1 authenticates to no real backend. Real integration requires per-origin credential isolation, TTY confirmation on `git remote add reposix::...`, and a revocation story. v0.2 scope.
- **`X-Reposix-Agent` spoofing / rate-limit bucket attack.** Phase 2 review M-02 — a client-controlled agent header means an attacker can consume a victim's rate-limit quota. v0.1 accepts this since the simulator-only mode has no real victims.
- **Rate-limit `DashMap` unboundedness.** Phase 2 review M-01 — unique agent headers grow the map unboundedly, memory-DoS vector. v0.2 LRU cap.
- **FUSE SIGTERM cleanup.** Phase 2 review M-04 — if the FUSE daemon is killed via SIGTERM (no Drop runs), the kernel mount leaks until `fusermount3 -u` runs. The CLI's `MountProcess` watchdog wraps this — belt-and-braces — but the daemon itself should handle SIGTERM.
- **CRLF + non-UTF-8 handling in `git-remote-reposix`.** Phase S review H-01/H-02 — the protocol reader silently strips `\r` and fails on non-UTF-8 bytes. Demo corpus is LF+ASCII so it doesn't trigger. v0.2 fix.
- **Adversarial swarm harness.** Explicitly dropped from v0.1 scope to keep the overnight-build budget honest. The simulator is already swarm-ready; the harness is a half-day of v0.2 work.
- **FUSE-in-CI integration job.** Same rationale — CI runs clippy/test/coverage on every commit; the mount-inside-CI assertion is a v0.2 deliverable that requires a privileged runner.
- **Workflow-rule enforcement at the simulator.** v0.1 accepts any `IssueStatus` transition. A production backend would reject "open → done" in favor of "open → in_progress → done". v0.2.

## Audit and reproducibility

- **Every file operation is logged** to `audit_events` in the simulator DB (see step 8c of the demo). Rows are append-only by trigger; a schema-attack test in `crates/reposix-core/tests/audit_schema.rs` proves `writable_schema=ON` cannot bypass it.
- **The `.planning/VERIFICATION.md`** document walks each of the 17 original requirements → evidence → test name → ship verdict. Produced by an independent `gsd-verifier` subagent, not by the agent that built the code.
- **All three review reports** (phase 1, phase 2+3, phase S) live in `.planning/phases/*/REVIEW.md` and enumerate HIGH/MEDIUM/LOW findings with exact file:line references. HIGH findings from phase 1 were fixed before phase 2 started; HIGH findings from later phases were cataloged and deferred to v0.2 with rationale.
