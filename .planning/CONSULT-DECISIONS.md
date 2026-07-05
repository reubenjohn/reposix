# CONSULT-DECISIONS — Portion-1 L1 coordinator ledger

Decision ledger for the v0.13.0 close-out drive (P92→P97), no-fable regime.
`[SELF]` = decided under the escalation-valve bar (below the E1–E4 threshold),
recorded not escalated. `[CONSULT]` = fable-consult was invoked (E-tier).

Format: `## <ID> — <one-line> [SELF|CONSULT] <date>` then rationale + evidence.

---

## D-P92-01 — Do NOT split P92 into P92a/P92b [SELF] 2026-07-04

**Situation (DP-4-adjacent sizing call):** Charter pre-authorized a P92a/P92b split
IF day-1 recon sized RBF-B-01 (rebase-ancestry, debugger-flagged) at >16h.

**Decision:** No split. Run P92 as a single phase.

**Evidence (recon agent a96e2c74, 2026-07-04):** The heavy mechanism fixes already
landed on `main` BEFORE P92 started —
- `cb630e5` scrubs `GIT_DIR`/`GIT_WORK_TREE`/`GIT_INDEX_FILE`/`GIT_COMMON_DIR`/
  `GIT_OBJECT_DIRECTORY`/`GIT_NAMESPACE` before the bare-cache `git config` shell-outs
  (`crates/reposix-cache/src/cache.rs:649-673`) — this was the root cause of the
  cache-open failure → fresh-root / no-audit path.
- `a0c84a3` chains `.with_audit()` on the Confluence + JIRA connectors
  (`crates/reposix-cache/src/backend_dispatch.rs:303,322`).

RBF-B-01 residual = author a T4 two-writer/pull-rebase ancestry regression test
(prove-before-fix: the test IS the deliverable; debugger only if RED against current
main) + TokenWorld smoke. Sized S–M (~4-10h). OP-3 residual = upgrade
`bus_write_audit_completeness.rs` to query BOTH SQLite tables directly + behavioral
no-retry verifier replacing the source-grep. Sized S (~2-6h). Combined well under
the 16h split trigger.

**Debugger risk:** LOW — root cause already diagnosed/fixed; escalate only if the new
ancestry test grades RED against current main.
