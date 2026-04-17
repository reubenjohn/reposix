# Project Retrospective

*A living document updated after each milestone. Lessons feed forward into future planning.*

---

## Milestone: v0.8.0 — JIRA Cloud Integration

**Shipped:** 2026-04-16
**Phases:** 3 (27–29) | **Plans:** 9 | **Sessions:** 1 autonomous session

### What Was Built
- `IssueBackend` → `BackendConnector` hard rename across all 5 crates + ADR-004 + `Issue.extensions` field (Phase 27)
- Full `reposix-jira` crate: JQL pagination, status mapping, subtask hierarchy, JIRA extensions, CLI dispatch, contract tests, ADR-005, reference docs (Phase 28)
- Complete JIRA write path: `create_issue` (POST + ADF + issuetype OnceLock cache), `update_issue` (PUT), `delete_or_close` (Transitions API + DELETE fallback) — 31 unit tests + 5-arm contract suite (Phase 29)

### What Worked
- Adapting the Confluence ADF module (`adf_paragraph_wrap`, `adf_to_markdown`) into Phase 29 with minimal rework — copy-adapted from reposix-confluence
- OnceLock for issuetype cache: clean session-scoped caching without mutexes
- FIFO mock ordering in wiremock contract test (corrected from plan's LIFO assumption) — caught during execution, not design
- Phased approach: rename first (Phase 27), read-only second (Phase 28), write path last (Phase 29) — each phase independently testable
- Wiremock-first testing: every write method tested against mocked HTTP before any real tenant

### What Was Inefficient
- gsd-tools `summary-extract` pulled accomplishments from wrong phase summaries during milestone archival — MILESTONES.md needed manual correction
- VALIDATION.md left in `status: draft` / `nyquist_compliant: false` at phase ship — should be updated to `complete` during Phase wrap-up
- `audit-open` command has a runtime bug (`output is not defined`) — blocked pre-close artifact check; had to proceed without it

### Patterns Established
- `OnceLock<Vec<String>>` on backend structs for session-scoped API discovery caches
- `make_untainted` / `make_untainted_for_contract` test helpers as fixtures for write path tests
- `assert_write_contract<B>()` pattern for create→update→delete→assert-gone round-trip — reusable across backends
- FIFO mock ordering in wiremock (not LIFO) — plan templates should note this explicitly
- Trait rename via hard cut (no backward-compat alias) at pre-1.0 — no aliasing debt

### Key Lessons
1. **Plan the rename phase before the feature phase.** Phase 27 (rename) before Phase 28 (new crate) meant the new crate was written against the correct API from day one — no rename debt carried into new code.
2. **Copy-adapt, don't rewrite.** The ADF module in Phase 29 was adapted from `reposix-confluence/src/adf.rs` with minor changes — saved ~2h of implementation time vs writing from scratch.
3. **wiremock FIFO vs LIFO ordering:** Plans that sequence mock responses should specify FIFO (`.mount()` in sequence with `expect(1)` barriers) rather than LIFO stack ordering. This mismatch caused a test rework in 29-03.

### Cost Observations
- Model: claude-sonnet-4-6 (all phases)
- Sessions: 1 autonomous
- Notable: Phases 27–29 executed in a single autonomous session with no human intervention mid-execution

---

## Cross-Milestone Trends

### Process Evolution

| Milestone | Phases | Key Change |
|-----------|--------|------------|
| v0.1.0 | 4+S | Initial MVD: simulator + FUSE + CLI + write path |
| v0.3.0 | 1 | First external adapter (Confluence read-only) |
| v0.6.0 | 5 | Write path + full sitemap (OP-1, OP-2, OP-3) |
| v0.7.0 | 6 | Hardening + Confluence expansion (OP-7..11) |
| v0.8.0 | 3 | JIRA Cloud integration: rename + read + write |

### Cumulative Quality

| Milestone | Tests (workspace) | Notes |
|-----------|-------------------|-------|
| v0.1.0 | 168 | Simulator + FUSE + guardrails |
| v0.3.0 | 193 | + Confluence wiremock |
| v0.5.0 | 278 | + IssueBackend decoupling + _INDEX.md |
| v0.6.0 | 317+ | + Confluence write path |
| v0.7.0 | ~400+ | + Hardening + comments + whiteboards |
| v0.8.0 | All green | + JIRA read + write (31 new unit + 5 contract) |

### Top Lessons (Verified Across Milestones)

1. **Simulator-first pays off every time.** Every new backend (GitHub, Confluence, JIRA) was fully tested against wiremock before touching a real API. Zero credential exposure during development.
2. **Copy-adapt before rewrite.** ADF module (Confluence → JIRA), workload patterns (SimDirect → ConfluenceDirect), contract test helpers — each time, adapting existing code was 3-5× faster than reimplementing.
3. **Phase ordering matters.** Foundation phases (type renames, trait changes) before feature phases prevents debt accumulation in new code.
