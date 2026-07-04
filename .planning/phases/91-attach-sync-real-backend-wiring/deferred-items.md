# P91 deferred items (out-of-scope discoveries)

Per gsd-executor SCOPE BOUNDARY: issues NOT directly caused by the current
task's changes are logged here, not fixed.

## 91-02 (Lane 1 QL-001)

- **[pre-existing] `agent-ux/p87-surprises-absorption` catalog row missing
  `claim_vs_assertion_audit`.** Surfaced as a `FAIL:` schema-validation line
  during `python3 quality/runners/run.py --cadence on-demand`. Confirmed
  pre-existing on `HEAD` (the committed row lacks the field, which is required
  for rows minted on/after 2026-05-08). NOT caused by 91-02 (which only edited
  the `real-git-push-e2e` and `ql-001-canonical-path-shape` rows). Out of
  scope for the QL-001 push-path fix. Candidate for a structure/catalog-honesty
  drain in P95 or a steward window.

## 91-04 (D91-08 hierarchy self-seed)

- **[pre-existing, worsened] `crates/reposix-confluence/tests/contract.rs`
  exceeds the `.rs` file-size-limits progressive-disclosure budget (20000
  chars).** Was already 32583 chars before 91-04's edit (well over budget);
  91-04's D91-08 hybrid rewrite of `contract_confluence_live_hierarchy` (+
  `make_hierarchy_issue`/`open_audit_db`/`DURABLE_PARENT_ID`/`DURABLE_CHILD_ID`
  helpers) added ~5.3k chars, landing at 37844. The `structure/file-size-limits`
  gate is currently WAIVED (until 2026-08-08) so this did not block the
  commit, but the file is now further from compliant. NOT fixed inline —
  splitting a 700+ line multi-arm contract test file (sim / wiremock / live /
  live-hierarchy arms, each with real fixture setup) into composable files is
  a real restructuring effort, not a <1h eager-fix, and orthogonal to D91-08's
  scope (making one test self-seeding). Candidate for a P95 (or waiver-renewal
  window before 2026-08-08) split — e.g. hoist `contract_confluence_live` +
  `contract_confluence_live_hierarchy` into a sibling
  `tests/contract_live.rs`, or extract `assert_contract`/fixture helpers into
  a shared `tests/common/mod.rs` the way `reposix-remote`'s test suite already
  does.

## Wave-5 (91-05) out-of-scope discoveries — 2026-07-04T21:00:02Z
- file-size-limits: pre-existing over-budget files (NOT touched by 91-05): quality/runners/test_audit_field.py (18861/15000), test_realbackend.py (16889/15000), verdict.py (16498/15000). Route to a P90/P92 framework-file split.
