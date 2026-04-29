# Phase 73: Connector contract gaps — bind 4 wiremock/decision rows (Context)

**Gathered:** 2026-04-29 (autonomous-run prep)
**Status:** Ready for execution
**Mode:** `--auto` (sequential gsd-executor on main; depth-1)
**Milestone:** v0.12.1
**Estimated effort:** 2-3 hours wall-clock (2 new Rust tests + 1 rebind + 1 prose-or-retire)

<domain>
## Phase Boundary

Close 4 `MISSING_TEST` rows asserting connector contract behavior. Two are pure rebinds to existing tests (`agent_flow_real.rs::dark_factory_real_*`); two require NEW Rust tests (wiremock-based byte-exact `Authorization` header assertion + JIRA `list_records` excludes attachments/comments); one row's source prose is STALE (claims JIRA real adapter "not implemented" but it shipped in v0.11.x — fix prose, then bind OR retire).

These rows live in v0.11.0-shipped docs but were never wired to behavioral tests, so the docs-alignment dimension surfaced them. Fixing them closes the JIRA-shape and connector-authoring-guide clusters from `quality/reports/doc-alignment/backfill-20260428T085523Z/PUNCH-LIST.md`.

**Explicitly NOT in scope:**
- Re-measuring JIRA token-economy benchmark numbers (deferred to perf-dim P67 or v0.13.0).
- Adding new attachments/comments support to JIRA adapter (decisions/005 explicitly defers this; the test asserts the deferral, not the missing feature).
- Touching the GitHub or Confluence backend feature surface (only test additions).

</domain>

<decisions>
## Implementation Decisions

### D-01: Two new Rust tests (auth-header-exact + jira-attachments-excluded)
Both use `wiremock` 0.6 (already a workspace dep — check `crates/reposix-confluence/Cargo.toml`). Test files live next to existing per-crate test files (`crates/reposix-confluence/tests/`, `crates/reposix-jira/tests/`).

### D-02: Auth-header test asserts BYTE-EXACT prefix
For Confluence: `Authorization: Basic <base64(email:token)>` — assert prefix `Basic ` AND the base64 of the test creds. For GitHub: `Authorization: Bearer <token>` OR `Authorization: token <token>` (depends on GH adapter — verify against `crates/reposix-github/src/`). The test seeds wiremock with `expect_match` on the header value and verifies the request was sent with that exact value.

### D-03: JIRA attachments/comments test seeds full issue then asserts markdown body
Seed wiremock JIRA `/rest/api/3/search` response with one issue containing `fields.attachment = [{...}]` AND `fields.comment.comments = [{...}]`. Call `JiraBackend::list_records`. Assert the resulting `Record.body` (markdown) contains NEITHER `attachment` nor `comment` fields (grep the rendered body). This tests the rendering boundary, not the JSON parsing layer.

### D-04: Real-backend smoke fixture is a pure rebind
`docs/connectors/guide/real-backend-smoke-fixture` already has tests (the 3 `dark_factory_real_*` `#[ignore]` tests in `crates/reposix-cli/tests/agent_flow_real.rs`). The bind invocation just points the catalog row at those tests; no test code changes. Use `--test crates/reposix-cli/tests/agent_flow_real.rs::dark_factory_real_confluence` (one of the three; bind to one specific test fn name per current schema).

### D-05: STALE JIRA prose — pick path (a) or (b) on ROI
- **Path (a) — keep the row, fix the prose:** Update `docs/benchmarks/token-economy.md:23-28` table row from "Jira (real adapter) | — | — | — | — | N/A (adapter not yet implemented)" to "Jira (real adapter) | (pending re-measurement) | (pending) | (pending) | (pending) | TBD (adapter shipped v0.11.x; bench rerun deferred to perf-dim P67)". Then bind to a verifier asserting `crates/reposix-jira/Cargo.toml` exists.
- **Path (b) — retire the row:** `propose-retire docs/benchmarks/token-economy/jira-real-adapter-not-implemented` with rationale "superseded by JIRA real adapter shipping in v0.11.x Phase 29; bench numbers tracked separately under perf-dim". Owner confirms retire in a follow-up TTY run.
- **Decision rule:** path (a) if total work < 30 min (prose edit + verifier + bind); path (b) otherwise.

### D-06: Wiremock fixtures are MINIMAL (one-shot per test)
Don't build a wiremock fixture-fragment library. One small JSON literal per test, inline. The existing `tests/agent_flow_real.rs` patterns are too heavyweight for these contract-level assertions. Easy to read, easy to maintain.

### D-07: Verifier subagent dispatch — Path A
Same as P72. Verdict at `quality/reports/verdicts/p73/VERDICT.md`.

### D-08: Eager-resolution of in-flight surprises
If wiremock setup reveals a deeper bug (e.g. the auth header is actually wrong in the live GitHub adapter), fix it ONLY IF < 1 hour. Otherwise append to SURPRISES-INTAKE.md.

### D-09: Cargo memory budget
- `cargo test -p reposix-confluence -p reposix-jira` is the heaviest invocation (limited to those two crates). One cargo at a time.
- Workspace-wide cargo only at end of phase if needed for verdict.

### D-10: CLAUDE.md update
P73 H3 subsection ≤30 lines under "v0.12.1 — in flight". Note the path-(a)-vs-(b) decision and what landed.

</decisions>

<canonical_refs>
## Canonical References

- `crates/reposix-confluence/tests/` — pattern for confluence wiremock tests.
- `crates/reposix-jira/src/lib.rs` — `JiraBackend::list_records` impl; the rendering boundary the attachments/comments test asserts.
- `crates/reposix-cli/tests/agent_flow_real.rs` — the existing `dark_factory_real_*` tests.
- `docs/decisions/005-jira-issue-mapping.md:79-87` — the prose claim being bound (attachments/comments excluded).
- `docs/guides/write-your-own-connector.md:158` — the prose claim being bound (auth-header + smoke-fixture).
- `docs/benchmarks/token-economy.md:23-28` — the STALE prose row being fixed-or-retired.
- `quality/PROTOCOL.md` — Principle A + B.

</canonical_refs>

<specifics>
## Specific Ideas

- The 4 catalog rows are listed verbatim in `.planning/milestones/v0.12.1-phases/ROADMAP.md` § Phase 73.
- For the GH auth header: `crates/reposix-github/src/` — find the `reqwest::Client` builder and confirm token format before writing the test.
- For wiremock: use `wiremock::matchers::header_exact` for byte-exact assertions. Don't use `header_regex` (that masks real bugs).
- After binding, run `target/release/reposix-quality doc-alignment refresh docs/guides/write-your-own-connector.md docs/decisions/005-jira-issue-mapping.md docs/benchmarks/token-economy.md` to flip rows.

</specifics>

<deferred>
## Deferred Ideas

- Re-running JIRA token-economy benchmark with the real adapter (P67 perf-dim).
- Adding wiremock-based GitHub rate-limit-gate test (write-your-own-connector.md:158 "Rate-limit gate arms on 429 / Retry-After"): worth a row in v0.13.0 if not already covered.
- Connector authoring guide refactor: the guide is currently 158 lines of mixed contract assertions and tutorial; could split into reference + tutorial. P77 GOOD-TO-HAVE candidate.

</deferred>

---

*Phase: 73-connector-contract-gaps*
*Context gathered: 2026-04-29*
*Source: HANDOVER-v0.12.1.md § 3b + autonomous-run prep.*
