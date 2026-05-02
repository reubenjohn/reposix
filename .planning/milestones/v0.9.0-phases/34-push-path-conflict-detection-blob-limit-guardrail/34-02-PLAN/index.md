---
phase: 34
plan: 02
title: "Push-time conflict detection + frontmatter allowlist + push audit ops"
wave: 2
depends_on: ["34-01"]
requirements: [ARCH-08, ARCH-10]
files_modified:
  - crates/reposix-cache/src/audit.rs
  - crates/reposix-cache/src/cache.rs
  - crates/reposix-remote/src/main.rs
  - crates/reposix-remote/tests/push_conflict.rs
autonomous: true
mode: standard
---

# Phase 34 Plan 02 — Push conflict detection + frontmatter allowlist + audit ops

<objective>
Make `handle_export` reject stale-base pushes with the canned `error refs/heads/main fetch first`
status (so git renders its standard `git pull --rebase` hint) plus a detailed
stderr diagnostic. Confirm the existing `sanitize()` step on `execute_action`
strips server-controlled frontmatter fields (`id`, `created_at`, `version`,
`updated_at`) — pin with a regression test. Wire four new audit ops into the
cache: `helper_push_started`, `helper_push_accepted`, `helper_push_rejected_conflict`,
`helper_push_sanitized_field`. Implements ARCH-08 + ARCH-10.
</objective>

<must_haves>
- Conflict detection runs AFTER `parse_export_stream` returns and AFTER the
  `state.backend.list_issues(state.project)` call; it runs BEFORE `plan()`
  produces `PlannedAction`s.
- Comparison rule: for each path in `parsed.tree`, parse the new blob's
  frontmatter to extract its `version` field. Look up the prior issue from
  `prior_by_id` (built from `list_issues` result). If `parsed_version != prior.version`
  → conflict.
- New issues (not in `prior_by_id`) are NOT subject to the conflict check —
  Create path is unaffected (no base version to conflict with).
- On conflict: emit `error refs/heads/main fetch first` on stdout (canned —
  triggers git's "perhaps a `git pull` would help" hint), emit detailed
  diagnostic on stderr via `diag()`, write `helper_push_rejected_conflict`
  audit row, set `state.push_failed = true`. Do NOT call `plan()` and do NOT
  invoke any REST writes.
- On clean push (post-execute success): write `helper_push_accepted` audit row.
- At entry to `handle_export`: write `helper_push_started` audit row.
- Cache lookup is best-effort: if `ensure_cache(state)` fails, log a warning
  and continue without audit rows (push path stays usable for deployments
  where cache is misconfigured). Conflict-detection logic still runs.
- Frontmatter sanitize regression test: an inbound blob with `id: 999999` and
  `version: 999999` produces a backend write whose `version` is the
  server-incremented value, not 999999. Existing `sanitize()` already does
  this; the test pins it.
- New audit op `helper_push_sanitized_field`: best-effort signal — when
  `sanitize()` would have overwritten a non-default field. v0.9.0 emits this
  audit row from `execute_action::Update` when `prior_version != 0` (already
  the case for any existing issue). Keeps the audit-vocabulary surface
  available for Phase 35 introspection.
- `cache_schema.sql` CHECK list already extended in Plan 01 (depends_on:
  ["34-01"]) — Plan 02 only adds the audit helpers + Cache wrapper methods.
</must_haves>

<canonical_refs>
- `.planning/phases/34-push-path-conflict-detection-blob-limit-guardrail/34-CONTEXT.md` §Reject-path atomicity (locked).
- `.planning/REQUIREMENTS.md` ARCH-08 + ARCH-10.
- `.planning/ROADMAP.md` Phase 34 success criteria 1, 2, 3, 6.
- `crates/reposix-remote/src/main.rs:268-350` — `handle_export` (insertion site).
- `crates/reposix-remote/src/main.rs:352-404` — `execute_action` (sanitize path).
- `crates/reposix-remote/src/diff.rs:99-204` — `plan()` (runs AFTER conflict check).
- `crates/reposix-core/src/taint.rs:103` — `sanitize` (already strips id/created_at/updated_at/version).
- `crates/reposix-core/src/frontmatter.rs` — frontmatter parser (returns `Issue` with `version`).
- `crates/reposix-cache/src/audit.rs` — pattern for new audit helpers.
- `crates/reposix-cache/src/cache.rs:115-163` — Cache wrapper methods pattern.
</canonical_refs>

---

## Chapters

| Chapter | Summary |
|---|---|
| [T01 — Add four push-event audit helpers + Cache methods](./t01.md) | Appends four `log_helper_push_*` functions to `audit.rs` and four wrapper methods to `cache.rs`, each with a unit test. |
| [T02 — Insert conflict-detection block at top of `handle_export`](./t02.md) | Rewrites `handle_export` in `main.rs` to run ARCH-08 stale-base version comparison before calling `plan()`, with canned reject status and stderr diagnostic. |
| [T03 — Wire `helper_push_sanitized_field` from `execute_action::Update`](./t03.md) | Emits the `helper_push_sanitized_field` audit row from the `Update` arm of `execute_action` before the `sanitize()` call. |
| [T04 — Integration test: stale-base push reject + clean push accept + sanitize regression](./t04.md) | New `push_conflict.rs` integration test file with three concrete test cases covering the reject, accept, and sanitize paths. |
| [T05 — Workspace gate: clippy + tests + audit-vocabulary check](./t05.md) | Final workspace-wide verification: `cargo check`, `cargo clippy`, `cargo test`, and audit-vocabulary surface count. |
