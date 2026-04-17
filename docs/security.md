# Security

## The lethal trifecta

Every deployment of reposix is a textbook **lethal trifecta**[^1]:

1. **Private data.** The FUSE mount exposes issue bodies, custom fields, Confluence page HTML, and any other attacker-influenced text the backend returns.
2. **Untrusted input.** Every remote ticket / page is attacker-influenced text — bodies, titles, comments, parent-id graphs.
3. **Exfiltration channel.** `git push` can target any remote the agent chooses, and the FUSE daemon makes outbound HTTP.

[^1]: [Simon Willison, "The lethal trifecta for AI agents"](https://simonwillison.net/2025/Jun/16/the-lethal-trifecta/), revised April 2026. Full red-team analysis of reposix is in [`.planning/research/threat-model-and-critique.md`](https://github.com/reubenjohn/reposix/blob/main/.planning/research/threat-model-and-critique.md).

As of v0.4, reposix talks to **three real backends**: the in-process simulator (default / test), GitHub Issues (read-only via `reposix-github`, Phase 10), and Atlassian Confluence Cloud (read-only via `reposix-confluence`, Phase 11). All three live behind the same `IssueBackend` trait and the same SG-01 allowlist. Real-backend calls only happen when the user has populated `.env` with credentials *and* set a non-default `REPOSIX_ALLOWED_ORIGINS` that includes the target host — the default configuration binds the allowlist to `http://127.0.0.1:*` (simulator loopback) and ships no credentials.

A naïve implementation of "issues as files" would combine all three legs with no mitigations. reposix cuts leg 3 at the architectural level (allowlist), hardens leg 1 (path validation + tainted-by-default typing), and assumes leg 2 is incurable (taint-type discipline and sanitize-on-egress).

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
| SG-07 | FUSE never blocks the kernel >5s | `crates/reposix-fuse/src/fs.rs::{list_issues_with_timeout, get_issue_with_timeout, update_issue_with_timeout, create_issue_with_timeout}` — `tokio::time::timeout` wrapper around each `IssueBackend` call + EIO return path | `crates/reposix-fuse/tests/sim_death_no_hang.rs` (kernel-timed `timeout 7 stat` subprocess) | ✓ step 4 (fast `ls`) |
| SG-08 | Demo recording shows guardrails firing | `scripts/demo.sh` steps 8a/8b/8c + `docs/demo.typescript` grep-verified | `grep -cE 'EPERM\|append-only\|refusing\|allowlist' docs/demo.typescript` ≥ 6 | — (meta) |

## Threat model

The full adversarial red-team analysis is in [`.planning/research/threat-model-and-critique.md`](https://github.com/reubenjohn/reposix/blob/main/.planning/research/threat-model-and-critique.md) (~600 lines). That document was produced by an independent subagent at T+0 of the autonomous build — its findings became the SG-* requirements above, baked in as first-class contracts before any code was written.

Highlights:

- **Egress allowlist is the single highest-leverage mitigation.** A sealed newtype around `reqwest::Client` eliminates leg 3 of the trifecta for every backend — sim, GitHub, and Confluence all resolve their target hosts through the same `HttpClient` with per-request URL recheck. Phase 8 H-01 sealed the inner client so `client.get(url).send()` bypass paths don't compile.
- **`rm -rf /mnt/reposix` is a real threat.** Without the bulk-delete cap, a confused agent (or a prompt-injection payload) could cascade into a `DELETE` storm. SG-02 catches this *at the git-remote-reposix layer* — defense in depth, independent of the FUSE daemon's own checks.
- **Frontmatter is a confused-deputy surface.** Attacker-authored issue bodies containing `permissions: rwxrwxrwx`, `owner: root`, `version: 999999` must not round-trip. SG-03 strips the server-controlled fields unconditionally. The `Tainted<T>` / `Untainted<T>` newtype split (SG-05) enforces this at compile time: `Untainted::new` is `pub(crate)` so no adapter can forge cleanliness.
- **Path traversal via issue title is prevented at the type level.** Filenames derive from `IssueId(u64)`, never from strings. Users cannot change filenames; the FUSE layer rejects any path component containing `/`, `\0`, `.`, or `..` with `EINVAL`. The v0.4 nested layout (see below) preserves this property — the slug algorithm is lossy-by-design and falls back to `page-<padded-id>` on empty / all-dash / `.` / `..` outputs.
- **Server-controlled IDs are validated before URL construction.** Phase 11 code-review WR-02 (Confluence): the numeric `space_id` returned from `?keys=` lookup is regex-checked `^[0-9]{1,20}$` before being spliced into a URL path, so an adversarial backend can't return `"12345/../../admin"` and pivot onto an unintended endpoint.

## The `tree/` overlay (v0.4, Confluence)

v0.4 ships a synthesized `tree/` directory under the mount root for backends that expose `BackendFeature::Hierarchy` (currently only Confluence). `tree/` is **read-only FUSE symlinks** from human-readable slug paths back to the canonical `pages/<padded-id>.md`. Three security properties, all tested:

- **Symlink targets cannot inject paths.** `TreeSnapshot::symlink_target()` constructs `../../pages/<id>.md` from the typed `IssueId(u64)` — never from the attacker-controlled title or slug. The slug only names the link, not the target.
- **Cycles cannot hang the kernel.** An adversarial backend returning a `parentId` cycle is broken by iterative DFS with a visited-set; the cycle-break node becomes an orphan root with a `tracing::warn!`. The 5s open / 3s readdir budgets hold under cycle. See `nested_layout_cycle_does_not_hang` in the FUSE integration tests.
- **The overlay cannot enter git.** The mount auto-emits a read-only `/tree/\n` `.gitignore` (compile-time const bytes, inode `4`, `perm: 0o444`). Title renames and reparenting only move FUSE-synthesized symlinks; `git diff` only surfaces real body edits.

Full design — slug algorithm, collision resolution, `_self.md` convention, known limitations — in [ADR-003](decisions/003-nested-mount-layout.md).

## Normalization of deviance

From the [`docs/research/agentic-engineering-reference.md`](https://github.com/reubenjohn/reposix/blob/main/docs/research/agentic-engineering-reference.md) §5.5 distillation of Simon's interview:

> Every deployment of an unsafe agent that doesn't get exploited increases institutional confidence in it. This is the Challenger O-ring dynamic. The field has been getting away with unsafe patterns because no headline-grabbing exploit has landed yet. One will. Don't rely on "it hasn't happened" as evidence of safety.

This page exists because shipping reposix with green tests and a beautiful demo but without honest security accounting would be exactly such a deployment. The guardrails above are not cosmetic — each one is enforced at the type system or the kernel boundary, and each one fires on camera in the recording.

## What shipped after v0.1

The list below tracks items that were deferred from v0.1 and have since landed. Each links to the release that shipped it.

- **Real-backend credentials — GitHub.** Shipped in [v0.2.0-alpha](https://github.com/reubenjohn/reposix/blob/main/CHANGELOG.md). `reposix list / mount --backend github` reads real GitHub Issues via `reposix-github`. Bearer token from `gh auth token` or `GITHUB_TOKEN`; anonymous fallback (60/hr) if absent. Guarded by SG-01 allowlist — callers must set `REPOSIX_ALLOWED_ORIGINS=...,https://api.github.com`.
- **Real-backend credentials — Confluence.** Shipped in [v0.3.0](https://github.com/reubenjohn/reposix/blob/main/CHANGELOG.md). `reposix list / mount --backend confluence` reads real Atlassian Confluence spaces via `reposix-confluence`. Basic auth via `ATLASSIAN_EMAIL` + `ATLASSIAN_API_KEY` (no Bearer path — Atlassian user tokens are not OAuth). Tenant subdomain validated against DNS-label rules before any request. `ConfluenceCreds` has a manual `Debug` impl that redacts `api_token`. Credentials stay in `.env` (gitignored); a pre-push token-hygiene hook is tracked in [HANDOFF.md OP-7](https://github.com/reubenjohn/reposix/blob/main/HANDOFF.md).
- **Adversarial swarm harness.** Shipped in Phase 9 as [`crates/reposix-swarm`](https://github.com/reubenjohn/reposix/blob/main/crates/reposix-swarm). `reposix-swarm --clients 50 --duration 30s --mode {sim-direct,fuse}` validates the SG-06 append-only audit invariant under load. Locally validated at 132,895 ops / 0% errors.
- **CRLF + non-UTF-8 handling in `git-remote-reposix`.** Shipped in v0.2.0-alpha (Phase S H-01/H-02 fix). `ProtoReader` now reads raw bytes; CRLF round-trips, non-UTF-8 doesn't tear the protocol pipe.
- **FUSE-in-CI integration job.** Shipped. `integration (mounted FS)` runs on every push with `continue-on-error: false`.
- **Real-backend CI contract.** Shipped. `integration-contract` (GitHub) and `integration-contract-confluence` (Atlassian) both run against live APIs on every push when the relevant secrets are configured, and skip cleanly otherwise.
- **FUSE write-path rewire onto `IssueBackend`.** Shipped in Phase 14 (v0.4.1). The FUSE `release` / `create` callbacks now dispatch through `IssueBackend::update_issue` / `create_issue` instead of the deleted `crates/reposix-fuse/src/fetch.rs` sim-REST helpers. The simulator's REST shape now lives in exactly one crate (`reposix-core::backend::sim::SimBackend`). Closes HANDOFF.md "Known open gaps" item 7.
- **`git-remote-reposix` backend-abstraction.** Shipped in Phase 14 (v0.4.1). The remote helper now constructs a `SimBackend` internally from the parsed `RemoteSpec` and dispatches all pushes through the `IssueBackend` trait; `crates/reposix-remote/src/client.rs` is deleted. URL syntax (`reposix::http://host:port/projects/slug`) is unchanged — a future phase can extend to `reposix::confluence://…` once real-backend writes land. Closes HANDOFF.md "Known open gaps" item 8.

## What's still deferred (v0.4+)

These are *known gaps*, not oversights. Each is tracked in [HANDOFF.md](https://github.com/reubenjohn/reposix/blob/main/HANDOFF.md).

- **`X-Reposix-Agent` spoofing / rate-limit bucket attack (simulator).** Phase 2 review M-02 — the simulator still trusts the client-supplied agent header for rate-limit bucketing. This was tolerable at v0.1 (when only the simulator shipped) but is *no longer abstract* now that real backends exist: the simulator is still the default test target, and agents that learn to trust the header here may carry the habit to production backends where bucketing happens server-side. Fix: HMAC-sign the header at session-start, verify on each request. v0.4+ (OP-7 hardening).
- **Rate-limit `DashMap` unboundedness (simulator).** Phase 2 review M-01 — unique agent headers grow the map unboundedly, memory-DoS vector against the simulator. v0.4+ LRU cap.
- **FUSE SIGTERM cleanup.** Phase 2 review M-04 — if the FUSE daemon is killed via SIGTERM (no Drop runs), the kernel mount leaks until `fusermount3 -u` runs. The CLI's `MountProcess` watchdog wraps this — belt-and-braces — but the daemon itself should handle SIGTERM. v0.4+.
- **Workflow-rule enforcement at the simulator.** The simulator accepts any `IssueStatus` transition. A production backend would reject "open → done" in favor of "open → in_progress → done". v0.4+.
- **Write path on `GithubReadOnlyBackend` and `ConfluenceReadOnlyBackend`.** Both adapters currently return `NotSupported` from `create_issue` / `update_issue` / `delete_or_close`. Real-backend writes are intentionally gated until per-origin write-side allowlists, confirm-on-destructive-op UX, and a revocation story are in place. Tracked in HANDOFF.md.
- **500-page truncation in `reposix-confluence`.** `list_issues` silently caps at 500 pages per invocation. Silent truncation is an SG-05 taint escape surface (the agent thinks it has the whole space when it doesn't). OP-7 queues a WARN log + `--no-truncate` CLI flag that errors instead of silently capping.
- **Audit-log PII redaction.** The simulator's audit rows record the full request path + agent id; issue titles that happen to contain secrets leak into the log. v0.4+ redaction pass.
- **Mount-pre-existing-`.gitignore` collision.** The v0.4 layout assumes the mount root is a virgin working tree. A user-authored `.gitignore` at mount time is not handled. See [ADR-003 §.gitignore emission](decisions/003-nested-mount-layout.md).

## Audit and reproducibility

- **Every file operation is logged** to `audit_events` in the simulator DB (see step 8c of the demo). Rows are append-only by trigger; a schema-attack test in `crates/reposix-core/tests/audit_schema.rs` proves `writable_schema=ON` cannot bypass it.
- **The `.planning/VERIFICATION.md`** document walks each of the 17 original requirements → evidence → test name → ship verdict. Produced by an independent `gsd-verifier` subagent, not by the agent that built the code.
- **Every shipped phase has a review report** (`.planning/phases/*/REVIEW.md`) enumerating HIGH/MEDIUM/LOW findings with exact file:line references. HIGH findings are either fixed before the next phase starts or explicitly deferred with rationale in the release notes. Phase 11 (Confluence) shipped with WR-01 (query-string injection) and WR-02 (server-controlled `space_id` path injection) fixed pre-merge; Phase 13 (nested layout / `tree/`) shipped with all threat-register T-13-* items either mitigated or documented in ADR-003 §Known limitations.
