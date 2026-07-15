---
phase: 114-t4-confluence-oid-drift-fix-first-reconcile-audit
plan: 01
subsystem: api
tags: [confluence, adf, atlas_doc_format, oid-drift, render-parity, wiremock]

# Dependency graph
requires:
  - phase: 114-RESEARCH
    provides: root-cause diagnosis (list-vs-get body-format mismatch → deterministic Error::OidDrift)
provides:
  - Confluence list_records and get_record request the SAME body representation (both send ?body-format=atlas_doc_format)
  - render-parity contract test (list body == get body for an ADF-native page)
  - defensive next_url re-append immunizing >100-page spaces against cursor body-format drop
affects: [114-02 (FIX-02 reconcile-audit), real-backend t4 gate SC1/SC2, TokenWorld checkouts]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "adapter render-parity: list and get paths must request identical body representations so build_from and read_blob hash byte-identical renders"

key-files:
  created: []
  modified:
    - crates/reposix-confluence/src/client.rs
    - crates/reposix-confluence/src/types.rs
    - crates/reposix-confluence/tests/contract.rs
    - crates/reposix-cli/tests/agent_flow_real.rs
    - .planning/phases/114-t4-confluence-oid-drift-fix-first-reconcile-audit/114-RESEARCH.md

key-decisions:
  - "Fixed at the root (Option 1): one query-param change on the LIST url — NOT a weakening of read_blob's drift check, NOT per-page GETs in the list walk"
  - "Applied the recommended (non-blocking) defensive next_url re-append, making OQ2 moot regardless of Confluence's _links.next behavior"
  - "Contract-test fixture string-encodes atlas_doc_format.value to match the live v2 wire format (avoids the object-encoded-fixture-lie trap)"

patterns-established:
  - "Render parity as a testable invariant: list_records(project)[i].body == get_record(project, id).body for an unmutated ADF-native page"

requirements-completed: [FIX-01]

# Metrics
duration: 6min
completed: 2026-07-15
---

# Phase 114 Plan 01: Confluence list-vs-get render-parity (FIX-01) Summary

**`list_issues_impl` now requests `?body-format=atlas_doc_format` on the LIST url so the Confluence list render and get render decode to byte-identical bytes — eliminating the deterministic `Error::OidDrift` that aborted every ADF-native page checkout.**

## Performance

- **Duration:** ~6 min
- **Started:** 2026-07-15T06:41:58Z
- **Completed:** 2026-07-15T06:48:14Z
- **Tasks:** 2/2 (TDD RED → GREEN)
- **Files modified:** 5 (4 code/doc + 1 research fold-in)

## Accomplishments
- Root-caused and fixed the product defect at its source: the LIST endpoint requested pages with NO `body-format`, so Confluence returned empty bodies; `build_from` hashed the empty-body list render while `read_blob` re-fetched the real body via `get_record` (`?body-format=atlas_doc_format`) → deterministic oid drift on every ADF-native page.
- Added `list_and_get_render_parity` contract test (RED before fix, GREEN after) proving `list_records` body == `get_record` body for an ADF-native fixture page.
- Shipped the recommended defensive `next_url` cursor re-append (separator-aware, no URL-parsing dependency), immunizing spaces >100 pages against a `_links.next` cursor that drops the param.
- Corrected two now-stale "list bodies are empty" doc claims (`ConfPage` doc + `agent_flow_real.rs`), including a lying-doc-claim doc block that justified a design decision on a premise the fix invalidated.

## Task Commits

Each task was committed atomically:

1. **Task 1: render-parity contract test (RED)** — `47fa803` (test)
2. **Task 2: adapter render-parity fix + stale-doc corrections (GREEN)** — `9908fcc` (fix)

**Plan metadata + RESEARCH fold-in:** _(this docs commit)_ (docs)

## Files Created/Modified
- `crates/reposix-confluence/src/client.rs` — PRIMARY FIX: appended `&body-format=atlas_doc_format` to the `list_issues_impl` LIST url; added defensive re-append on the `next_url` cursor closure (separator-aware).
- `crates/reposix-confluence/src/types.rs` — corrected the `ConfPage` doc comment (was claiming the list endpoint returns `body: {}` empty).
- `crates/reposix-confluence/tests/contract.rs` — added `list_and_get_render_parity`.
- `crates/reposix-cli/tests/agent_flow_real.rs` — corrected the stale "list bodies are empty" inline comment AND the doc block that justified "MUST call get_record, not list_records" on the now-false empty-body premise.
- `.planning/.../114-RESEARCH.md` — annotated OQ2 RESOLVED (defensive re-append) and OQ1 DEFERRED (pre-ADF empirical question awaits the real-backend gate).

## The one-line diff (root cause → fix)

```
- "{}/wiki/api/v2/spaces/{}/pages?limit={}"
+ "{}/wiki/api/v2/spaces/{}/pages?limit={}&body-format=atlas_doc_format"
```
Args unchanged (`self.base(), space_id, PAGE_SIZE`); reuses the existing `translate()` + `Tainted::new` wrap; adds ZERO round-trips.

## RED → GREEN evidence
- **RED (pre-fix):** `list_and_get_render_parity` FAILED — the LIST request went out as `GET /wiki/api/v2/spaces/12345/pages?limit=100` (no body-format), missed the `query_param("body-format","atlas_doc_format")` mock, got a 404, and `list_records` returned `Err(Other("confluence returned 404 …"))`. Proves the test catches the defect.
- **GREEN (post-fix):** full `cargo test -p reposix-confluence` — `list_and_get_render_parity ... ok`; the 3 pre-existing `adversarial_*` list-mock tests all `ok` (query_param matcher ignores the extra param); 98 client unit tests pass; contract 6 passed / 2 ignored (live). No regressions.

## Defensive next_url handling (Open Question 2)
Confluence's `_links.next` `body-format` preservation was UNVERIFIED in research (no live multi-page probe). Rather than leave it to chance, I shipped the recommended defensive re-append: the `next_url` closure now checks whether the followed page url already contains `body-format=` and, if not, appends it with a separator-aware join (`&` when the cursor url has a query, `?` otherwise — a robustness improvement over the plan's plain `&`, correct even for a bare-path cursor). No new dependency; reuses the existing relative-vs-absolute string idiom. TokenWorld (<100 pages) never triggers this path, but larger spaces are now immune. OQ2 is thereby moot regardless of the live answer.

## Real-backend gate (SC1 + SC2)
**NOT-VERIFIED (env-gated).** This wave is wiremock-only; SC1 (`reposix init confluence::TokenWorld` checkout with zero OidDrift on page 7766017) and SC2 (`agent-ux/t4-conflict-rebase-ancestry-real-backend` GREEN) require live `ATLASSIAN_*` creds + `REPOSIX_ALLOWED_ORIGINS` that were not exercised here. The coordinator/developer must run them with a live `.env` before phase close. The fix and the wiremock render-parity proof are the autonomous-testable floor; the live gate is the bar.

## Residual gap (honest)
Render parity is achieved for ADF-native pages. Pre-ADF (storage-only) pages are handled by `translate`'s storage fallback, but whether TokenWorld actually contains one is empirically unresolved without the live gate (Open Question 1, DEFERRED). If the gate trips `OidDrift` on a DIFFERENT page id post-fix, that page is likely pre-ADF — file a GOOD-TO-HAVE, do not claim universal Confluence compatibility.

## 114-RESEARCH.md fold-in
Applied PARTIALLY and truthfully. OQ2 → RESOLVED (defensive re-append shipped). OQ1 → DEFERRED (pre-ADF question requires the env-gated real-backend gate this wiremock wave cannot run). The `## Open Questions` heading was deliberately NOT marked `(RESOLVED)` because OQ1 remains genuinely open — a blanket marker would be a false claim.

## Deviations from Plan
None affecting scope. Two adaptations worth naming:
1. **[Rule 3 — Tooling]** `cargo nextest` is not installed on this VM (`error: no such command: nextest`); ran the plan's verification via `cargo test -p reposix-confluence` (equivalent coverage, respects the ONE-cargo-invocation rule). Filed as a GOOD-TO-HAVE (see below).
2. **[Rule 2 — Correctness/doc-honesty]** Beyond the plan's targeted inline comment, I also corrected the adjacent doc block in `agent_flow_real.rs` (`## Why this MUST call get_record`) whose "list_records requests NO body-format … every listed record carries an EMPTY body" premise the fix invalidates. Leaving it would be a lying doc claim (Owner mandate OD-3 #2). Same function, no behavior change.

## Known Stubs
None.

## Self-Check: PASSED
- `crates/reposix-confluence/tests/contract.rs::list_and_get_render_parity` — present (grep MATCH), runs GREEN.
- `crates/reposix-confluence/src/client.rs` — LIST url carries `body-format=atlas_doc_format` (line 202).
- `crates/reposix-cache/src/builder.rs` — `written_oid != oid` count == 1 (drift check untouched, Pitfall 1 held).
- Commits `47fa803` (test) and `9908fcc` (fix) exist in git log.
