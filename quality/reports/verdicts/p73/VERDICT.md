# P73 Verdict — connector contract gaps

**Verifier:** unbiased subagent (Path A — Task tool, zero session context)
**Phase contract:** PLAN.md must_haves + CONTEXT.md D-01..D-10 + REQUIREMENTS.md CONNECTOR-GAP-01..04
**Verdict:** GREEN
**Graded:** 2026-04-29 (autonomous-run)

## Dimension scores

| # | Dimension | Status | Evidence |
|---|---|---|---|
| 1 | 4 catalog rows BOUND | COVERED | All 4 rows in `quality/catalogs/doc-alignment.json` carry populated `tests` arrays; status-after.txt reports `claims_missing_test 13 → 9` (-4 exact match). All 4 row ids confirmed: `auth-header-exact-test` (2 tests, multi-test row), `real-backend-smoke-fixture` (1 test), `attachments-comments-excluded` (1 test), `jira-real-adapter-not-implemented` (1 shell verifier). |
| 2 | 2 new Rust tests + 1 rebind, all pass | COVERED | Sequential per-crate runs: `cargo test -p reposix-confluence --test auth_header` → 1 passed; `cargo test -p reposix-github --test auth_header` → 1 passed; `cargo test -p reposix-jira --test list_records_excludes_attachments` → 1 passed. D-09 sequential cargo respected. |
| 3 | D-02 byte-exact header assertion | COVERED | Both `auth_header.rs` files use `wiremock::matchers::header("Authorization", expected_header.as_str())`. Plan-time prose cited `header_exact`; actual wiremock 0.6.5 API is `header(K, V)` returning `HeaderExactMatcher` — same byte-exact semantics, NOT `header_regex`. Surface-level reconciliation documented in SUMMARY.md "Deviations from plan" §1 with source citation `wiremock-0.6.5/src/matchers.rs:355`. |
| 4 | D-03 rendering boundary test | COVERED | `list_records_excludes_attachments_and_comments` seeds wiremock JIRA `/rest/api/3/search/jql` with `fields.attachment` AND `fields.comment.comments` populated, calls `JiraBackend::list_records` (the rendering boundary, NOT `JiraFields` parse layer), then asserts (1) `record.body` excludes `attachment`/`comment` keywords + sentinel strings; (2) `record.extensions` keys exclude both terms; (3) extensions VALUES exclude the seeded sentinel strings. Four-axis assertion is the rendering-boundary contract claimed by D-03. |
| 5 | D-04 real-backend smoke rebind | COVERED | Catalog row `docs/connectors/guide/real-backend-smoke-fixture` cites `crates/reposix-cli/tests/agent_flow_real.rs::dark_factory_real_confluence` — pure rebind, zero new test code. Per-spec choice of one canonical fn (D-04). |
| 6 | D-05 path (a)/(b) decision honored | COVERED | Path (a) chosen per D-05 default (commit `40ae5c1`). Prose at `docs/benchmarks/token-economy.md:28` updated from `N/A (adapter not yet implemented)` to `TBD (adapter shipped v0.11.x; bench rerun deferred to perf-dim P67)` with data columns `(pending)`. Verifier `quality/gates/docs-alignment/jira-adapter-shipped.sh` exists, executable, exits 0 with PASS message. Bind landed on row `jira-real-adapter-not-implemented`. No path (b) propose-retire commit (correct — only one path allowed). |
| 7 | D-06 wiremock minimalism | COVERED | All 3 test files inline minimal JSON literals via `serde_json::json!({...})`. No new fixture-fragment library. The JIRA test has one helper fn `issue_with_attachments_and_comments()` (40 lines) confined to that single test file — appropriate scope. No imports of shared fixture modules. |
| 8 | D-09 cargo memory budget | COVERED | All 3 verifier-side test runs were single-crate `-p` invocations, sequential. Commit messages confirm task-by-task execution; no parallel cargo orchestration observed. SUMMARY explicitly documents D-09 compliance. |
| 9 | Atomic commits | COVERED | 10 P73 commits (range `83bd3e4..b1d5b9e` inclusive of both endpoints = 10 commits; expected ~11 — the gap is Task 6 which was an inline decision checkpoint, no commit, per SUMMARY.md). Each commit cites a specific CONNECTOR-GAP-* id or task scope ("scaffold", "implement Confluence", "rebind", "path (a)", "bind", "walk", "CLAUDE.md H3", "phase SUMMARY"). Atomicity is clean — no commit straddles requirement boundaries. |
| 10 | CLAUDE.md update + banned-words | COVERED | P73 H3 subsection at `CLAUDE.md:344` titled `### P73 — Connector contract gaps`. 14 lines of body (well under 30-line cap). Names all 3 new test files, the rebind, and the path (a) decision. Documents `wiremock::matchers::header(K, V)` as canonical idiom with the plan-time correction. Banned-words check on the new section: zero matches for FUSE/fusermount/inode/daemon/kernel/syscall/replace/partial-clone/promisor (note: CLAUDE.md is not under `docs/` so the banned-words linter does not formally apply, but the new section is clean against the same rule set). |
| 11 | OP-8 honesty check | COVERED | `SURPRISES-INTAKE.md` and `GOOD-TO-HAVES.md` received no new P73 entries. Cross-check: SUMMARY.md "Deviations from plan" enumerates 4 plan-vs-source mismatches (wiremock fn name, `walk` vs `refresh`, `GithubReadOnlyBackend` vs `GithubBackend`, verifier home dir) — all documented as in-task surface-level reconciliations, not skipped findings. Each deviation was < 5 lines, < 1 hour, no new dependency, scope-local — meets the eager-fix criteria in CLAUDE.md OP-8 verbatim. The empty intake is the honest record, NOT signal-suppression. Auth-header tests pass first try ⇒ no live-adapter bug surfaced ⇒ honest claim of "nothing surprising worth carrying forward." |
| 12 | alignment_ratio delta plausibility | COVERED | summary-before.json: `alignment_ratio=0.8939`, `claims_bound=320`, `claims_missing_test=13`. summary-after.json: `alignment_ratio=0.9050`, `claims_bound=324`, `claims_missing_test=9`. Recompute: `claims_total=388`, `claims_retired=30`, non-retired = 358; `324/358 = 0.9050279...` — matches recorded value. `claims_missing_test` -4 exactly matches the 4 P73 rows transitioning MISSING_TEST → BOUND. `claims_bound` +4 matches. All three deltas mutually consistent. |
| 13 | No prohibited actions | COVERED | No `git push` to `origin/main` from this phase (origin/main HEAD is at older commit `docs(state): P65 SHIPPED…`). No new tag created. No `confirm-retire` commits in P73 range. No `propose-retire` commits in P73 range (path (a) chosen, no retire needed). |

## Findings

**Strengths:**
- Catalog-first discipline upheld: BEFORE snapshot + 3 stub test files committed (`83bd3e4`) BEFORE any test implementation. Subsequent commits replace `unimplemented!()` bodies with real assertions, and the bind-time hash target was therefore stable from Wave 1.
- The JIRA rendering-boundary test is impressively defense-in-depth: it asserts at four axes (body keywords, body sentinel strings, extensions keys, extensions values) — meaningfully harder for a future regression to slip past than a single `assert!(!body.contains(...))`.
- Plan-vs-source reconciliations were caught in-task with explicit citations (wiremock fn name → matchers.rs:355; doc-alignment verb → CLI --help; type name → lib.rs:125). This is the exact OP-8 eager-fix discipline working as designed.
- The path (a) verifier home divergence (placed at `quality/gates/docs-alignment/jira-adapter-shipped.sh` not `quality/gates/docs-alignment/verifiers/jira-adapter-shipped.sh`) was self-caught and documented in SUMMARY "Deviation #4". The chosen home matches sibling-dimension convention (e.g. `quality/gates/structure/`), so the decision is defensible — just not the planned location. Worth noting but not RED.

**Minor observations (not findings — informational):**
- Multi-test row `auth-header-exact-test` carries 2 tests but the SUMMARY's "Total commits: 11" framing doesn't match `git log` (10 commits in `83bd3e4..b1d5b9e` inclusive). The discrepancy is a reading artifact: Task 6 (decision checkpoint) had no commit, so 9 task-commits + 1 SUMMARY = 10 git commits. SUMMARY.md acknowledges this implicitly ("(inline)" annotation on Task 6). Not a defect.
- The `multi_test_rows` summary stat moved from 0 → 1 between BEFORE and AFTER snapshots, confirming the auth-header multi-test bind landed structurally correct.
- `docs/benchmarks/token-economy.md:28` row id slug retains the historical `jira-real-adapter-not-implemented` name (now factually inaccurate). SUMMARY.md notes this is a cosmetic GOOD-TO-HAVES candidate (P77). Not in P73 scope; correctly not folded in.

## Recommendations to orchestrator

1. **Advance to P74** (narrative + UX cleanup + prose-fix). All 13 dimension gates COVERED; no AMBER, no RED.
2. The path (a) verifier-home pattern (`quality/gates/<dim>/<name>.sh`, not `verifiers/<name>.sh`) is now established convention — future docs-alignment plans should follow it without needing to reconcile.
3. The wiremock `header(K, V)` idiom is now the canonical byte-exact-auth-header pattern; future connector additions should follow the structure in `crates/reposix-confluence/tests/auth_header.rs` (drive through `BackendConnector` trait, not private helpers).
4. P73 surfaces zero items for P76 (SURPRISES-INTAKE.md unchanged) and zero items for P77 (GOOD-TO-HAVES.md unchanged) from this phase. The slug-rename observation (`jira-real-adapter-not-implemented` row id is factually stale) is a candidate for P77 if owner wants cosmetic cleanup, but defensible to leave alone.

---

**Verdict:** GREEN. All 13 dimensions COVERED. Phase 73 closes correctly.
