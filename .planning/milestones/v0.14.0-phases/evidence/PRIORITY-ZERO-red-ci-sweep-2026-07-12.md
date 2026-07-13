# PRIORITY-ZERO: red main-CI diagnosis + TokenWorld durable-fixture repair

- **Date:** 2026-07-12 (CI run stamped 2026-07-13 UTC)
- **Failing run:** `29220925797` on main (HEAD `3f3d824`), conclusion FAILURE
- **Failing job:** `integration (contract, real confluence)` (job `86725997774`),
  step "Run reposix-confluence contract test against real Atlassian", exit **101**
- **Failing test:** `contract::contract_confluence_live_hierarchy`
  (`contract_confluence_live` in the same binary PASSED)

## Confirmed cause (NOT the orphan-pages hypothesis)

The handover's prime suspect — "2 orphan p93 smoke pages tripping a page-count
assertion" — was **wrong**. The test never counts pages. The real cause:

Both **durable fixture pages were trashed**, not missing. During this session's heavy
TokenWorld churn, the protected durable pair (parent `7766017`, child `7798785`) was
moved to Confluence trash (adapter `delete_or_close` issues `DELETE /wiki/api/v2/pages/{id}`
which trashes — `crates/reposix-confluence/src/lib.rs:418-426`). A **trashed page still
resolves** via `GET /wiki/api/v2/pages/{id}` (returns `status:"trashed"`, `parentId:null`),
so `backend.get_record` returns `Ok` for both. The test's "if either is missing, self-seed
a fresh pair" fallback therefore never triggered — it took the read-only fast-path assert
branch (`contract.rs:846`) and panicked on the null parentId.

**Quoted panic line (contract.rs:847):**

```
thread 'contract_confluence_live_hierarchy' panicked at crates/reposix-confluence/tests/contract.rs:847:9:
assertion `left == right` failed: durable fixture child 7798785 must have parent_id == Some(7766017)
(docs/reference/testing-targets.md § Protected durable fixtures); this test NEVER deletes either id
  left: None
 right: Some(RecordId(7766017))
```

## Contract-test invariant

`crates/reposix-confluence/tests/contract.rs:846-855` — when both durable ids resolve,
`child.parent_id` MUST equal `Some(DURABLE_PARENT_ID)`
(`DURABLE_PARENT_ID = 7766017`, `DURABLE_CHILD_ID = 7798785`, defined at
`contract.rs:750-751`). Doc anchor: `docs/reference/testing-targets.md:85-118`
(§ "Protected durable fixtures — NEVER delete").

## Live TokenWorld state (space id 360450)

Before repair — `current` listing had exactly ONE page:
`9109514  parent=None  "p93 smoke B (kind=test 1783910889)"` (orphan debris).
Both durable pages were `status:"trashed"` with `parentId:null`.

| id | role | before | after repair |
|----|------|--------|--------------|
| 7766017 | durable parent | trashed, parent null | **current**, space-root (parent null — correct) |
| 7798785 | durable child  | trashed, parent null | **current**, parentId **7766017**, parentType page |
| 9109514 | orphan `p93 smoke B` (kind=test) | current | **deleted** (HTTP 204) |

Final `current` listing = exactly the 2 protected durable pages, invariant satisfied.

## Repair actions (TokenWorld only — sanctioned mutate-freely target)

Performed via the new committed helper `scripts/confluence_tokenworld.py`
(inspect / list / restore / reparent / delete; refuses to delete the two protected ids):

1. `restore 7766017` — un-trashed (v1 `PUT ...?status=trashed` status flip) → current.
2. `restore 7798785` — un-trashed → current.
3. `reparent 7798785 7766017` — restore lost the parent link; v2 `PUT` re-set parentId → 7766017.
4. `delete 9109514` — swept the orphan `p93 smoke B` (HTTP 204).

Pages **kept:** 7766017, 7798785. Pages **deleted:** 9109514 (`p93 smoke B`, kind=test).
No other page was touched; no protected id was ever deleted.

## Noticing (filed for follow-up, not fixed here — no product-code changes in a P0)

**Test-robustness gap (MEDIUM):** `contract_confluence_live_hierarchy`'s "if either
durable id is missing, self-seed" fallback treats only a `get_record` *error* as
"missing". A **trashed** durable page resolves `Ok` (status trashed, parentId null), so
the fallback is bypassed and the read-only assert hard-panics instead of self-seeding —
turning routine trash churn of the fixture into a red CI. Suggested fix: in the fast
path, gate the read-only assert on `status == current` (or treat trashed as missing so
the self-seed path runs). This would make the contract test self-healing against fixture
trashing, the exact failure mode that produced this P0. → belongs in
`.planning/.../SURPRISES-INTAKE.md`.

## Verification

Re-ran the failed job against the same code with the now-clean backend:
`gh run rerun 29220925797 --failed`, waited via `scripts/ci-wait.sh 29220925797`.

**Re-run conclusion: GREEN.** Run `29220925797` now `status=completed`,
`conclusion=success`; the `integration (contract, real confluence)` job
(`86725997774`) re-ran to `conclusion=success`. `ci-wait.sh` exited 0.
Backend-state hypothesis confirmed: identical code, clean/repaired backend → pass.
PRIORITY ZERO RESOLVED — main CI is green.
