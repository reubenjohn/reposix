---
phase: 91-attach-sync-real-backend-wiring
plan: 02
subsystem: api
tags: [git-remote-helper, fast-import, fast-export, push-planner, canonical-path, ql-001]

# Dependency graph
requires:
  - phase: 91-01
    provides: catalog-first mint of agent-ux/ql-001-canonical-path-shape (NOT-VERIFIED)
provides:
  - "reposix_core::path::{record_filename, record_path, issue_id_from_path} — single canonical issues/<id>.md source of truth"
  - "fast-import peek-one-byte-LF fix (BUG-3) — no-op push no longer drops the lowest-id record"
  - "diff::plan issues/*.md filter + deletes-win reconciliation (BUG-2)"
  - "refresh unpadded producer + D91-10 stale-file sweep"
  - "box-independent QL-001 regression suite (RED-if-bug-returns) at the plan()/parse_export_stream layer"
  - "ql-001-canonical-path verifier flipped to 8 real asserts (PASS); real-git-push-e2e waiver retired + pre-pr restored"
affects: [91-03, 91-05, SC-6-litmus, real-git-push-e2e-CI]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Canonical path helper in reposix-core, routed by all producers/consumers (no hand-rolled parsers)"
    - "LITERAL-string fixtures (never helper-derived) so a regressed helper cannot mask a returning bug"
    - "BufRead fill_buf()+consume(1) peek-one-byte instead of read_line for optional-trailing-LF handling"

key-files:
  created: []
  modified:
    - crates/reposix-core/src/path.rs
    - crates/reposix-remote/src/diff.rs
    - crates/reposix-remote/src/fast_import.rs
    - crates/reposix-remote/src/main.rs
    - crates/reposix-remote/src/precheck.rs
    - crates/reposix-cache/src/builder.rs
    - crates/reposix-cli/src/refresh.rs
    - quality/gates/agent-ux/ql-001-canonical-path.sh
    - quality/catalogs/agent-ux.json

key-decisions:
  - "D91-01: canonical shape is issues/<id>.md unpadded, spelled once in reposix-core"
  - "D91-02: box-independent cargo/grep proof locally; real-git-push-e2e is CI-only (git 2.25 here exits 75)"
  - "D91-10: refresh wipes differently-spelled record files before rewrite; non-record files untouched"
  - "D91-11/MF-2: real-git-push-e2e stays a legacy row (no minted_at) with the waiver removed"

patterns-established:
  - "Fixtures use LITERAL issues/<id>.md, never record_path() — magic-fixture masking hazard (raise-list §3)"

requirements-completed: [QL-001]

# Metrics
duration: ~150min
completed: 2026-07-04
---

# Phase 91 Plan 02: QL-001 Canonical Push-Path Fix Summary

**The push diff planner now round-trips a genuinely git-produced `issues/<id>.md` tree: the 4-way path-shape mismatch (BUG-1), the non-issue-blob reject + M/D reconciliation gap (BUG-2), and the commit-message-swallows-first-M-line stream-parser bug (BUG-3) are fixed through one shared reposix-core helper, guarded by a RED-if-bug-returns regression suite.**

## Performance

- **Duration:** ~150 min
- **Tasks:** 4/4 (all auto)
- **Files modified:** 17 (7 source, 10 fixture/verifier/catalog)
- **Commits:** `1c03da0`, `4bebfa3`, `3bc10bf`, `c9e2b8f`

## Accomplishments

1. **Canonical helper (Task 1).** `reposix_core::path::record_filename/record_path/issue_id_from_path` (padding-agnostic, built on `validate_record_filename`). Routed builder.rs, refresh.rs, fast_import emit, diff.rs prior-key, precheck.rs; deleted the QL-157 `main.rs` duplicate.
2. **Parser + planner fixes (Task 2).** fast_import peek-one-byte-LF; diff::plan `issues/*.md` filter + deletes-win; refresh unpadded + D91-10 stale-sweep with a pre-seeded-stale-file test.
3. **Honest fixtures + regressions (Task 3).** Re-keyed ~13 masking fixtures to LITERAL `issues/<id>.md`; added 5 box-independent QL-001 regressions covering criteria 1-4 + deletes-win.
4. **Verifier + waiver (Task 4).** ql-001 verifier now runs 8 real asserts (PASS); real-git-push-e2e waiver retired, pre-pr restored, transport_claim:false; QL-001 intake → RESOLVED.

## SF-2 RED-if-bug-returns proof (verbatim)

Method: reverted the three fix hunks in-place (prior-key → `format!("{:04}.md")`, removed the `issues/*.md` filter + deletes-win, restored `read_line`), kept the re-keyed + new tests, ran `cargo test -p reposix-remote --bin git-remote-reposix`. All 7 canonical regressions went RED, then restored the fix → GREEN.

```
test diff::tests::full_seeded_tree_push_emits_zero_deletes ... FAILED
test diff::tests::canonical_single_edit_is_one_update ... FAILED
test diff::tests::reposix_metadata_paths_are_ignored_not_rejected ... FAILED
test diff::tests::delete_wins_over_add_for_same_path ... FAILED
test diff::tests::unchanged_push_emits_no_patches ... FAILED
test diff::tests::extra_trailing_newline_is_a_noop ... FAILED
test fast_import::tests::commit_message_without_trailing_lf_does_not_swallow_first_m_line ... FAILED
test result: FAILED. 69 passed; 7 failed; 0 ignored; 0 measured; 0 filtered out
```

Selected assertion detail (verbatim):

```
# BUG-1 create/delete storm hits the cap:
full_seeded_tree_push_emits_zero_deletes:
  canonical full-tree push must plan clean: BulkDeleteRefused { count: 6, limit: 5, tag: "[allow-bulk-delete]" }

# BUG-1 no-op push misclassified as 3 Creates + 3 Deletes:
unchanged_push_emits_no_patches:
  assertion `left == right` failed: unchanged push should emit ZERO actions;
  got: [Create(...id 1...), Create(...id 2...), Create(...id 3...),
        Delete { id: RecordId(1) }, Delete { id: RecordId(2) }, Delete { id: RecordId(3) }]

# BUG-2 non-issue blob rejects the push:
reposix_metadata_paths_are_ignored_not_rejected:
  non-issue metadata path must not reject the push (BUG-2):
  InvalidBlob { path: ".reposix/fetched_at.txt", source: InvalidRecord("missing frontmatter open fence") }

# BUG-3 first M-line after commit message swallowed:
commit_message_without_trailing_lf_does_not_swallow_first_m_line:
  BUG-3: first M-line after commit message must survive; tree={}
```

After restoring the fix: `cargo test -p reposix-remote --bin git-remote-reposix` → `76 passed; 0 failed`.

## Criterion-6 grep proof (canonical single spelling)

```
$ grep -rn 'format!("{:04}\.md"|format!("{:011}\.md"' crates/ | grep -v target | grep -v reposix-core/
(no output — zero record-path construction survives outside reposix-core)
```

## Per-crate test results (with fix)

- `reposix-core --lib path::` — 37 passed (5 new helper tests).
- `reposix-remote` (full: bin + integration) — 76 bin + all integration suites passed; 0 failed.
- `reposix-cli refresh` — `refresh_removes_stale_padded_duplicate_and_regenerates_canonical` + `git_refresh_commit_creates_commit` passed; `refresh_integration` 3 passed.
- `reposix-cache` — full suite passed (31 lib + integration, incl. builder tree tests).
- `cargo clippy -p {reposix-core,reposix-remote,reposix-cli,reposix-cache} --all-targets -- -D warnings` — clean. `cargo fmt --all` — clean.

## Verifier state

- `bash quality/gates/agent-ux/ql-001-canonical-path.sh` → exit 0 (PASS, 8 asserts).
- `bash quality/gates/agent-ux/real-git-push-e2e.sh` → exit 75 on this box (git 2.25.1) — honest environment gap (D91-02), NOT a failure. CI (git ≥2.34, pre-pr job) is the full-stack green.

## Deviations from Plan

### Auto-fixed / expanded

**1. [Rule 3 - Blocking] Reverted runner-induced catalog mutations before committing Task 4.**
- **Found during:** Task 4 pre-commit.
- **Issue:** Running `python3 quality/runners/run.py --cadence on-demand` (a plan `<verify>` step) wrote FAIL verdicts + fresh `last_verified` (>= 2026-05-08 cutoff) back into 4 untouched rows (p87/p88 + 2), which then tripped `_audit_field.validate_row`'s claim_vs_assertion_audit requirement and blocked the commit.
- **Fix:** `git checkout HEAD -- agent-ux.json`, re-applied ONLY the two intended row edits (real-git-push-e2e, ql-001). No `--no-verify`.
- **Commit:** c9e2b8f.

**2. [expanded fixture inventory] Re-keyed more fixtures than the plan enumerated.**
- push_conflict.rs:231,321 and mirror_refs.rs:149,254,344,394,460 (`one_file_export("0042.md",…)`) were NOT in the plan's §interfaces list but MUST be re-keyed — with the prior-key now `issues/<id>.md`, any surviving bare-shape fixture would break the suite. Re-keyed all to `issues/42.md`.

### None otherwise — plan executed as written; git 2.25 exit-75 was expected (D91-02).

## NOTICED (ownership charter §2)

1. **The runner mutates the committed catalog as a side effect of running a full cadence** (`save_catalog` on status change). Any executor who runs `run.py --cadence {on-demand,pre-pr,…}` to "verify" will dirty untouched rows and can trip the load gate on pre-existing audit-field gaps. Worth a guardrail (a `--dry-run`/`--no-write` flag) — filed conceptually below; the immediate blast was contained by reverting.
2. **Pre-existing catalog-honesty gap:** `agent-ux/p87-surprises-absorption` and `agent-ux/p88-good-to-haves-drained` lack `claim_vs_assertion_audit` and load-pass ONLY because their HEAD `last_verified` (2026-05-01) predates the 2026-05-08 cutoff. The moment their verifier is re-run they will hard-block catalog load. Logged to `deferred-items.md` (P95/steward candidate). NOT fixed (out of QL-001 scope, and I lack context to author truthful audit prose for those verifiers).
3. **builder.rs has TWO filename sites** (`:90` inner-tree build, `:312` unchanged-item recompute), not one — both now route through `record_filename`. The plan/research cited only `:90`.
4. **`slug_or_fallback` (path.rs) is genuinely cosmetic-only for this fix** — refresh.rs is id-based (`record_filename`), not slug-based; Confluence page slugs go through a different materializer path. No Confluence page-naming convention was touched (respected `<out_of_scope>`).
5. **`protocol.rs::crlf_blob_body_round_trips_byte_for_byte` is NOT a QL-001 RED gate** — its empty-prior fixture makes a Create fire on both buggy and fixed planners, so it masks BUG-1 by construction. Left re-keyed for consistency but the real RED proof lives in the dedicated diff.rs regressions.
6. **fast_import emit side (`emit_import_stream`) is the deprecated git<2.34 import fallback** — routing it through `record_path` is correct but this path is exercised only on old git; the primary production read path is the cache builder (already canonical).

## Intake filed

- `SURPRISES-INTAKE.md` QL-001 BLOCKER → **RESOLVED** with the 4 commit SHAs + per-criterion evidence map.
- `deferred-items.md` → pre-existing p87/p88 catalog audit-field gap (out of scope).

## Self-Check: PASSED
