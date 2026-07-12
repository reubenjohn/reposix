---
phase: 106
plan: 01
title: Lost-update guard — shared-cache last_fetched_at cursor no longer gates conflict detection
type: bugfix
severity: HIGH
autonomous: true
requirements: [RBF-LR-05]
depends_on: [P105]
provides: [push-side-optimistic-concurrency-per-record]
affects: [crates/reposix-remote/src/precheck.rs]
---

# Phase 106: Lost-update guard (shared-cache cursor staleness)

## Objective

Close the **SILENT LOST UPDATE** data-loss window filed to
`.planning/milestones/v0.14.0-phases/SURPRISES-INTAKE.md`
(2026-07-12 08:10, discovered-by P105, severity HIGH).

Two `reposix init`/`attach` checkouts of the same SoT share ONE bare cache —
keyed by `(backend, project)` per `reposix_cache::path::resolve_cache_path` — and
therefore ONE wall-clock `last_fetched_at` cursor. When clone A pushes an edit,
the SoT-write branch advances that shared cursor to `now`
(`write_loop.rs:309`, `c.write_last_fetched_at(Utc::now())`). Clone B then pushes
a conflicting stale-base edit; its L1 precheck runs
`backend.list_changed_since(last_fetched_at=now)` → an EMPTY changed-set (A's write
is at-or-before `now`) → the pre-guard code, which GATED the per-record version
check on changed-set membership, never version-checked B's record → B's PATCH lands
and silently clobbers A's edit. Empirically reproduced (P105 repro,
`repro-lost-update.sh`, live sim, git 2.25.1).

## Race characterization (before → after)

- **BEFORE:** `precheck_export_against_changed_set` (precheck.rs) bailed on
  `if !changed_set.contains(&id) { continue; }` at the TOP of the push-set loop.
  `changed_set = list_changed_since(last_fetched_at)`. A shared cursor advanced past
  a concurrent write empties that set → the record that DID move is invisible → no
  `get_record`, no version compare, no conflict → stale-base PATCH lands (LOST UPDATE).
- **AFTER:** the `changed_set` membership no longer suppresses the check. For EVERY
  pushed record the cache knows about (has a prior OID = an Update), the precheck
  issues the authoritative `get_record` and compares the agent's local base
  `version` against the backend's CURRENT `version` — the backend is the sole SoT
  arbiter. A stale base (v0 vs backend v2) rejects with the standard
  `error refs/heads/main fetch first`. `changed_set` is retained only as a forensic
  signal (a conflict on a record absent from the delta is WARN-logged as the
  shared-cursor-staleness fingerprint).

## Why not a cache-prior comparison (rejected alternative)

`execute_action` (main.rs) does NOT refresh the shared cache prior on push — it only
writes the audit row and calls `backend.update_record`. So both clones' cache priors
stay equally stale after A's push; comparing the pushed base against the cache prior
version would ALSO miss the conflict. The backend is the only authoritative source,
so the fix must `get_record`.

## Cost / trade-off

One `get_record` per pushed Update (bounded by push size, not project size). For a
typical agent push (1–5 touched records) this is negligible. A large push touching
many records that mostly did NOT move on the backend pays more GETs than the
delta-gated path did — an accepted correctness-over-perf trade for a data-loss bug.
A perf-optimized variant (persist per-record base version in the oid_map to skip the
GET on an unchanged record) is filed to GOOD-TO-HAVES, not implemented here.

## Tasks

1. **[catalog-first]** Mint `code/lost-update-shared-cursor-rejected` in
   `quality/catalogs/code.json` (GREEN contract, status NOT-VERIFIED) + verifier
   `quality/gates/code/lost-update-shared-cursor.sh`. `type=auto`.
2. **[fix + regression, tdd]** Remove the changed-set suppression gate in
   `precheck_export_against_changed_set`; version-check every pushed Update against
   the backend. Add
   `precheck::tests::stale_base_push_rejected_when_shared_cursor_advanced_past_concurrent_write`
   — a real `Cache` (oid_map + advanced cursor via `build_from`) + an
   `AdvancedCursorMock` backend at v2/past-`updated_at`, asserting the stale-base
   (v0) push returns `Conflicts([(1, 0, 2, _)])`. FAILS without the fix (returns
   `Proceed`), PASSES with it. `type=auto tdd=true`.

## Success criteria

1. `cargo test -p reposix-remote stale_base_push_rejected_when_shared_cursor_advanced_past_concurrent_write`
   exits 0 (the new regression passes).
2. The pre-existing `push_conflict.rs` regressions
   (`stale_base_push_emits_fetch_first_and_writes_no_rest`,
   `clean_push_emits_ok_and_mutates_backend`,
   `frontmatter_strips_server_controlled_fields`) still pass (no regression).
3. `quality/gates/code/lost-update-shared-cursor.sh` exits 0 and writes its artifact.
4. `git push origin main` lands before the verifier subagent.

## Verification

```bash
cargo test -p reposix-remote --lib precheck        # unit regression
cargo test -p reposix-remote --test push_conflict  # no regression on siblings
bash quality/gates/code/lost-update-shared-cursor.sh
```
