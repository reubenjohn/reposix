← [back to index](./index.md) · phase 80 plan 01

# Objective & Architecture Overview

<objective>
Land the mirror-lag observability infrastructure for v0.13.0's DVCS
topology: two refs per SoT-host (`refs/mirrors/<sot-host>-head` direct
ref + `refs/mirrors/<sot-host>-synced-at` annotated tag) written into
the cache's bare repo by the existing single-backend `handle_export`
push path on success; readable by vanilla `git fetch` via the helper's
widened `stateless-connect` advertisement; cited in conflict-reject
hints with a "(N minutes ago)" rendering for diagnosis-by-staleness.

This is a **single plan, four sequential tasks** per RESEARCH.md
§ "Plan splitting":

- **T01** — Catalog-first: 3 rows + 3 TINY verifier shells (status FAIL).
- **T02** — Cache crate impl: new `mirror_refs.rs` (writer + reader,
  copy-and-adapt from `sync_tag.rs`); new `audit::log_mirror_sync_written`;
  pub mod + re-exports in `lib.rs`; 4 unit tests.
- **T03** — Helper crate wiring: insert ref writes + audit-row write
  into `handle_export` success branch; widen `stateless-connect`
  advertisement to `refs/mirrors/*`; compose reject-hint stderr from
  `cache.read_mirror_synced_at`.
- **T04** — Integration tests + catalog flip FAIL → PASS + CLAUDE.md
  update + per-phase push + (orchestrator dispatches) verifier.

Sequential (T01 → T02 → T03 → T04). Per CLAUDE.md "Build memory budget"
the executor holds the cargo lock sequentially across T02 → T03 → T04.
T01 is doc-only (catalog row + verifier shell scaffolding).

## Architecture (for executor context — read these BEFORE diving into tasks)

The two refs live in the **cache's bare repo** (`Cache::repo` — a
`gix::Repository` handle on `<cache-root>/reposix/<backend>-<project>.git/`),
NOT in the working tree's `.git/`. The working tree receives them via
the helper's `stateless-connect` `list` advertisement. T03 widens this
from `refs/heads/main` only to also include `refs/mirrors/*`. Vanilla
`git fetch` from the working tree pulls the refs across; `git log
refs/mirrors/<sot>-synced-at -1` shows when the mirror last caught up.

The `<sot-host>` slug is `state.backend_name` (the controlled enum
`"sim" | "github" | "confluence" | "jira"` set by the URL-scheme
dispatcher in `crates/reposix-remote/src/backend_dispatch.rs`). NOT
user input. NOT a free-form host parameter. RESEARCH.md pitfall 5 ratifies
this.

The SoT SHA written into `refs/mirrors/<sot-host>-head` is the cache's
post-write synthesis-commit OID — the OID returned by
`Cache::build_from()` AFTER the SoT REST writes have landed. The
implementation in T03 calls `cache.build_from().await?` AFTER
`execute_action` succeeded for every action and BEFORE the
`proto.send_line("ok refs/heads/main")`. RESEARCH.md A2 sketched
`parsed.commits.last()`; verified at planning time that `ParsedExport`
has NO `commits` field — its fields are `commit_message`, `blobs`,
`tree`, `deletes` (`crates/reposix-remote/src/fast_import.rs:72-81`).
The synthesis-commit OID is the meaningful SHA: it's what the cache's
bare repo presents to vanilla `git fetch` after this push.

Trade-off: `build_from` is "tree sync = full" per
`crates/reposix-cache/src/lib.rs:7-10`. Calling it a second time per
push touches the full tree-list cycle. For single-backend push at
P80 scale (one space, dozens to hundreds of issues) this is acceptable.
P81's L1 migration replaces this with `list_changed_since`. T03
documents the cost as a comment adjacent to the `build_from` call.

## Annotated-tag shape for `<sot>-synced-at`

Per RESEARCH.md Pattern 2 + ROADMAP success criterion 1:

- The tag is annotated, NOT lightweight — it has a tag *object* with a
  message body.
- Message body, first line: `mirror synced at <RFC3339>` — single human-
  readable line for plain `git log` rendering. No JSON envelope (RESEARCH
  flagged this as hostile to cold-reader UX). Future structured fields
  can ride additional `key: value` lines without breaking the first-line
  contract.
- gix API: `Repository::tag(name, target_id, tagger, message, force)` is
  the canonical idiom (RESEARCH.md A1 — verify at workspace pin gix 0.83
  during T02; fallback path is two `RefEdit`s if the API name differs,
  bounded ≤ 30 lines).

## Reject-hint shape

Per RESEARCH.md "Reject-message hint composition":

```
hint: your origin (GH mirror) was last synced from <sot> at <ts> (<N> minutes ago)
hint: run `reposix sync` to update local cache from <sot> directly, then `git rebase`
```

When `read_mirror_synced_at` returns `None` (first push, no refs yet):
omit the hint LINES cleanly — print the existing reject diagnostic
without the synced-at hint. RESEARCH.md pitfall 7 + sibling test
`reject_hint_first_push_omits_synced_at_line` covers this.

## Best-effort vs hard-error semantics

- **Ref writes:** best-effort. `tracing::warn!` on failure; the push
  still acks `ok` to git. Matches the `Cache::log_*` family precedent
  (let-else + WARN). NO new error variant.
- **Audit row write:** UNCONDITIONAL per OP-3. The
  `log_mirror_sync_written` audit-row write fires even if both ref
  writes failed (records the attempt). Best-effort SQL semantics
  match `log_sync_tag_written` precedent (`crates/reposix-cache/src/audit.rs:340-363`)
  — SQL errors WARN-log; the function returns nothing; caller drops
  via `cache.log_mirror_sync_written(...)`.

## Execution constraints

This plan **must run cargo serially** per CLAUDE.md "Build memory
budget". Per-crate fallback (`cargo check -p reposix-cache`,
`cargo check -p reposix-remote`, `cargo nextest run -p <crate>`) used
instead of workspace-wide.

This plan terminates with `git push origin main` (per CLAUDE.md push
cadence) with pre-push GREEN. The catalog rows' initial FAIL status
is acceptable through T01-T03 because the rows are `pre-pr` cadence
(NOT `pre-push`); the runner re-grades to PASS during T04 BEFORE the
push commits.
</objective>
