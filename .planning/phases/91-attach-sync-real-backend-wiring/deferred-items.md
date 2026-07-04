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
