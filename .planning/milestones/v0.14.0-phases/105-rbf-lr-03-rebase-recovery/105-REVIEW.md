---
phase: 105-rbf-lr-03-rebase-recovery
reviewed: 2026-07-12T00:00:00Z
depth: deep
files_reviewed: 6
files_reviewed_list:
  - crates/reposix-remote/src/fast_import.rs
  - crates/reposix-remote/src/main.rs
  - crates/reposix-cli/src/init.rs
  - crates/reposix-remote/tests/bus_capabilities.rs
  - crates/reposix-remote/tests/protocol.rs
  - crates/reposix-remote/tests/stateless_connect.rs
  - quality/catalogs/agent-ux.json
  - quality/gates/agent-ux/rebase-recovery-reconciles.sh
findings:
  critical: 1
  warning: 2
  info: 2
  total: 5
status: issues_found
---

# Phase 105 (RBF-LR-03 rebase-recovery): Code Review Report

**Reviewed:** 2026-07-12
**Depth:** deep (cross-file: helper stream ↔ advertised refspec ↔ init fetch refspec)
**Verdict:** CHANGES-REQUIRED

## Summary

The two-layer fix is well-engineered and the namespace split is correct: no
remaining helper-side code writes `refs/reposix/origin/*`, the private
`refs/reposix-import/*` namespace cannot collide with user refs or `refs/heads/*`,
`resolve_import_parent` reads the correct (tracking) ref, taint is clean, and
back-compat is a non-issue (the persisted `remote.origin.fetch` refspec is
unchanged; only the per-connection advertised refspec changed).

BUT the `from <parent>` chaining introduced a **correctness regression on record
deletion**: the synthesized commit now overlays M-directives onto the parent
tree with **no `deleteall`/`D`**, so a record removed at the SoT is silently
retained in the fetched tracking tree — and would be resurrected at the SoT on
the next push. Empirically confirmed against real `git fast-import` (below).

Priority answers: (1) namespace split — correct, no residual double-writer, no
collision. (2) no-op guard — sound for add/modify, but see BL-01: it compares a
*pure-snapshot* tree oid against the parent while the real commit emits an
*overlay* tree; these diverge exactly when a record is deleted. (3) taint —
clean, static ref literals only. (4) back-compat — no migration gap.

## Critical Issues

### CR-01 (BLOCKER): `from <parent>` without `deleteall` silently retains deleted records

**File:** `crates/reposix-remote/src/fast_import.rs:187-210`
**Issue:** `emit_import_stream` chains the synthesized commit onto the parent
(`from {p.commit}`, line 197-199) and then emits only `M 100644` directives for
each record in the current snapshot (line 200-210). git fast-import initializes
a `from`-based commit with the **ancestor's tree**, then applies M/D as an
overlay — with no `deleteall` and no `D <path>` lines, a record that was deleted
at the SoT (present in the parent tree, absent from `list_records`) is **never
removed**. The fetched `refs/reposix/origin/main` tree keeps the stale
`issues/<id>.md`; a subsequent `git push` fast-exports that retained file and the
diff planner re-creates the deleted record at the SoT (silent resurrection).

Confirmed empirically with real `git fast-import` (git 2.25.1): after a
"delete issue2" snapshot fetch (`from <parent>` + `M issues/1.md` only, no
deleteall), `git ls-tree` still lists `issues/2.md`.

This is a regression: the pre-fix parentless-root commit built the tree purely
from the snapshot, so deletions propagated correctly on a fresh checkout. Trigger
is narrow (record *removed* from the SoT, not merely closed — reachable via
Confluence page delete and the sim's `delete_or_close`), but the impact is data
integrity: deleted records reappear.

Note the internal inconsistency this exposes: `snapshot_tree_oid`
(line 56-96) computes the **pure snapshot** tree, and the no-op guard compares it
to `p.tree` (line 150). That premise (emitted tree == pure snapshot) only holds
when no record was removed — which is precisely the missing-`deleteall`
assumption. The author clearly intended a pure rebuild; the emit path just
doesn't implement it.

**Fix:** emit `deleteall` immediately after the `from` line and before the `M`
directives, so every fetch rebuilds the tree from scratch (making the emitted
tree byte-identical to `snapshot_tree_oid`, and restoring deletion propagation):

```rust
if let Some(p) = parent {
    writeln!(w, "from {}", p.commit)?;
    writeln!(w, "deleteall")?;   // rebuild tree from snapshot; propagate SoT deletions
}
```

Add a unit test: seed {1,2} via emit, then emit {1} with the seed as parent
against real `git fast-import`, assert `issues/2.md` is absent from the resulting
tree. And extend the gate with a deletion drift scenario (see WR-01).

## Warnings

### WR-01 (WARNING): gate + unit tests never exercise record deletion

**File:** `quality/gates/agent-ux/rebase-recovery-reconciles.sh:238-366`
**Issue:** Both scenarios drift the SoT via *edits* (append to `issues/1.md`,
REST PATCH body). Neither scenario, nor any unit test in `fast_import.rs`,
deletes a record from the SoT and asserts it disappears from the fetched tree.
That is exactly the CR-01 blind spot — the gate passes green while the deletion
regression is live. `git_fast_import_roundtrip_with_parent_fast_forwards`
(fast_import.rs:521) only tests add/modify.
**Fix:** add a Scenario C (delete a record at the SoT, run the documented
recovery, assert the record's file is gone from the working tree and the SoT is
not re-populated) once CR-01's `deleteall` lands.

### WR-02 (WARNING): no-op growth guard is never proven to fire against a real repo

**File:** `crates/reposix-remote/src/fast_import.rs:148-156`, `483-512`
**Issue:** The unit test `unchanged_tree_emits_no_commit` fires the guard by
feeding `snapshot_tree_oid`'s own output back as `parent.tree` — it proves the
`==` branch, not that `snapshot_tree_oid` actually equals git's real tree oid for
an unchanged snapshot. In production `p.tree` comes from `git rev-parse
<ref>^{tree}`. If gix's tree serialization ever diverged from git's, the guard
would **never** fire and the ref would grow unboundedly on every no-op fetch —
with no test to catch it (the failure is silent; wrong direction is safe re: data
loss, but the guard's entire purpose is defeated). The gate also never performs
two consecutive no-op fetches and asserts the ref did not advance.
**Fix:** add a test that seeds via `emit_import_stream(None)` against real `git
fast-import`, reads `rev-parse ^{tree}`, then emits the *same* snapshot with that
real tree oid as `parent.tree` and asserts a reset-only (no `commit`) stream —
proving `snapshot_tree_oid` agrees with git. Optionally assert ref-non-advance
across a repeated no-op fetch in the gate.

## Info

### IN-01 (LOW): catalog assert #7 has no matching gate assertion

**File:** `quality/catalogs/agent-ux.json` (expected.asserts[6])
**Issue:** The row asserts "emit_import_stream emits `from <parent>` … and writes
commit/reset to refs/reposix-import/main, never refs/reposix/origin/main." That
claim is covered by `fast_import.rs` unit tests (`assert_no_collapsed_namespace`),
not by any `asserts_passed` string the *shell gate* emits (the CLOBBER guard
checks the private ref *exists*, but not the `from <parent>` emission). Depending
on how strictly `agent-ux/test-name-vs-asserts` maps `expected.asserts` →
`asserts_passed`, this entry may lack a 1:1 mapping. Not a correctness issue.
**Fix:** either add a gate assertion that greps the emitted stream for
`commit refs/reposix-import/main` + `from`, or reword the assert to reference the
unit-test coverage explicitly.

### IN-02 (LOW): stateless-connect (git ≥ 2.34) fix path remains unverified

**File:** `quality/gates/agent-ux/rebase-recovery-reconciles.sh:368-382`
**Issue:** The gate honestly records that only git 2.25.1 is present, so the
`import` path is forced and the protocol-v2 `stateless-connect` fetch path (the
one modern git actually uses in production) is skipped, not faked — good honesty.
But it means RBF-LR-03's real-world fetch path is unverified here; it needs a
modern-git CI run to close PLAN §5. Flagging so it is not forgotten at
milestone-close.
**Fix:** track the §5 modern-git run as the documented follow-up (already TODO'd
in-transcript); do not let it silently pass as "covered."

---

_Reviewed: 2026-07-12_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: deep_
