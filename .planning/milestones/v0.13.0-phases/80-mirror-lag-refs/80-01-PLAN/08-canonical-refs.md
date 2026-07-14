← [back to index](./index.md) · phase 80 plan 01


<canonical_refs>
- `.planning/REQUIREMENTS.md` DVCS-MIRROR-REFS-01..03 (lines 65-67) —
  verbatim acceptance.
- `.planning/ROADMAP.md` § Phase 80 (lines 83-101) — phase goal +
  success criteria.
- `.planning/research/v0.13.0-dvcs/architecture-sketch.md` § "2. Mirror-
  lag observability via plain-git refs" + Q2.1, Q2.2, Q2.3.
- `.planning/research/v0.13.0-dvcs/decisions.md` § "Phase-N+1 (mirror-
  lag refs) decisions" — Q2.1 (refs/mirrors/...), Q2.2 (staleness
  window doc-clarity), Q2.3 (bus updates both refs).
- `.planning/phases/80-mirror-lag-refs/80-RESEARCH.md` — full research
  bundle. Especially:
  - § "Wiring `handle_export` success path — exact line citations" — the
    line-by-line wiring point (lines 470-489).
  - § "Reject-message hint composition" — verbatim hint template.
  - § "Catalog row design (lands FIRST per QG-06)".
  - § "Pitfalls" — 8 pitfalls; load-bearing for T03.
  - § "Assumptions Log" A1, A2, A3 — gix tag API, ParsedExport shape,
    advertisement widening.
- `crates/reposix-cache/src/sync_tag.rs` — DONOR PATTERN. The new
  `mirror_refs.rs` is a copy-and-adapt of this file. Specifically:
  - Lines 24-25: gix `RefEdit` imports.
  - Lines 33: `SYNC_TAG_PREFIX` constant precedent.
  - Lines 95-101: `format_sync_tag_slug` precedent for ref-name formatting.
  - Lines 153-190: `Cache::tag_sync` Pattern 1 (direct ref via RefEdit) —
    this is the verbatim donor for `write_mirror_head`.
  - Lines 178-187: audit-write call pattern (`audit::log_sync_tag_written`).
- `crates/reposix-cache/src/audit.rs` lines 340-363 (`log_sync_tag_written`)
  — DONOR PATTERN for `log_mirror_sync_written`.
- `crates/reposix-cache/src/cache.rs` line 23 — `pub(crate) repo: gix::Repository`
  — the bare-repo handle the new helpers use.
- `crates/reposix-cache/src/cache.rs` lines 232-241 — `Cache::log_helper_push_started`
  + `log_helper_push_accepted` family — the audit-row precedent.
- `crates/reposix-cache/src/cache.rs` lines 443-463 — `Cache::log_attach_walk`
  (P79 precedent for a Result-returning audit helper). The new
  `log_mirror_sync_written` mirrors `log_sync_tag_written` (returns ())
  per RESEARCH.md "Don't Hand-Roll" row 3.
- `crates/reposix-cache/src/builder.rs` line 56 — `pub async fn build_from(&self) -> Result<gix::ObjectId>`
  — the synthesis-commit OID accessor T03's wiring re-calls.
- `crates/reposix-cache/src/builder.rs` line 436 — `pub async fn read_blob(&self, oid: gix::ObjectId) -> Result<Tainted<Vec<u8>>>`
  — UNCHANGED by P80; documented for context only.
- `crates/reposix-cache/src/lib.rs` lines 30-39 — pub mod declarations
  (the new `pub mod mirror_refs;` joins this list alphabetically:
  `mirror_refs` between `meta` and `path`).
- `crates/reposix-cache/src/lib.rs` lines 52-54 — re-export precedent
  (`pub use sync_tag::{...}`).
- `crates/reposix-remote/src/main.rs` lines 280-491 (`handle_export`) —
  the wiring target.
- `crates/reposix-remote/src/main.rs` lines 309-314 (`ensure_cache` →
  `state.cache.as_ref()`) — confirmed cache handle is available in
  `handle_export`'s success branch.
- `crates/reposix-remote/src/main.rs` lines 384-407 (conflict reject
  branch) — wiring point for hint composition.
- `crates/reposix-remote/src/main.rs` line 411-432 (plan-error reject
  branches) — these reject paths do NOT cite mirror refs (mirror-lag is
  not the diagnosis for a malformed-blob or bulk-delete error). Hint
  composition is targeted at the conflict path only.
- `crates/reposix-remote/src/main.rs` lines 470-489 (success branch) —
  insertion point for ref writes. NEW code goes BETWEEN
  `cache.log_helper_push_accepted` (line 471) and `cache.log_token_cost`
  (line 484). The existing `cache.log_token_cost` block remains
  unchanged.
- `crates/reposix-remote/src/main.rs` line 50 — `state.backend_name: String`
  is the `<sot-host>` slug source.
- `crates/reposix-remote/src/stateless_connect.rs` (entire file — T03
  read_first) — to find where ref advertisement composes; investigate
  whether widening to `refs/mirrors/*` requires a filter change or is
  already automatic via tunneling to `git upload-pack`.
- `crates/reposix-remote/src/fast_import.rs` lines 70-81 — `ParsedExport`
  shape (verified at planning time: NO `commits` field; fields are
  `commit_message`, `blobs`, `tree`, `deletes`).
- `quality/catalogs/agent-ux.json` — existing catalog file; 3 new rows
  join. Row shape mirrors the existing
  `agent-ux/reposix-attach-against-vanilla-clone` row (P79 precedent).
- `quality/gates/agent-ux/reposix-attach.sh` — TINY-shape verifier
  precedent (P79). The 3 new mirror-refs verifiers mirror this shape:
  cargo build → start sim → mktemp → scenario → assert.
- `quality/catalogs/README.md` § "Unified schema" — required fields
  per row.
- `quality/PROTOCOL.md` § "Principle A — Subagents propose; tools
  validate and mint" — the rule that the hand-edit annotates a
  documented gap from (NOT applies) per GOOD-TO-HAVES-01.
- `quality/runners/run.py:54-56` — `VALID_CADENCES = ("pre-push",
  "pre-pr", "weekly", "pre-release", "post-release", "on-demand")`.
  `--cadence pre-pr` is valid; the runner re-grades catalog rows to
  PASS during T04.
- `crates/reposix-cli/tests/agent_flow.rs` — integration test pattern
  (`#[ignore]` real-backend, `#[test]` sim-backed; sim subprocess + CLI
  binary against tempdir). T04's tests in
  `crates/reposix-remote/tests/mirror_refs.rs` mirror this pattern.
- `crates/reposix-sim/src/main.rs` — sim CLI args, seed format.
- `CLAUDE.md` § "Build memory budget" — strict serial cargo, per-crate
  fallback.
- `CLAUDE.md` § "Push cadence — per-phase" — terminal push protocol.
- `CLAUDE.md` § Operating Principles OP-1 (simulator-first), OP-3
  (audit log unconditional), OP-7 (verifier subagent), OP-8 (+2 reservation).
- `CLAUDE.md` § "Quality Gates" — catalog-first rule (T01 first commit
  writes catalog rows that define this phase's GREEN contract).

This plan introduces NO new threat-model surface beyond what already
exists in the cache + helper. Refs are local to the cache's bare repo
(no new HTTP construction site); ref names are gix-validated; reject-
message stderr cites bytes from a tag-message body the helper itself
wrote (no taint propagation from the SoT). Audit op `mirror_sync_written`
joins the existing `audit_events_cache` table without DDL change. See
the `<threat_model>` section below for the STRIDE register.

Annotated-tag ref name validation: `gix::refs::FullName::try_from`
rejects `..`, `:`, control bytes per `gix_validate::reference::name`.
The `sot_host` slug is `state.backend_name` (a controlled enum:
`"sim" | "github" | "confluence" | "jira"`) — not free-form user input.
RESEARCH.md pitfall 5 + Threat-pattern row "Malicious ref name injection"
both ratify this design.
