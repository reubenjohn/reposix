← [back to index](./index.md)

# Hard constraints (carried into the plan body)

Per the user's directive (orchestrator instructions for P81) and
CLAUDE.md operating principles:

1. **Catalog-first (QG-06).** T01 mints THREE rows + TWO verifier shells
   BEFORE T02–T04 implementation. Initial status `FAIL`. The
   `perf-targets.json` and `agent-ux.json` rows are hand-edited per
   documented gap (NOT Principle A) — annotated in commit message
   referencing GOOD-TO-HAVES-01. The `doc-alignment.json` row is
   minted via `reposix-quality doc-alignment bind` (Principle A applies
   to docs-alignment dim).
2. **Per-crate cargo only (CLAUDE.md "Build memory budget").** Never
   `cargo --workspace`. Use `cargo check -p reposix-cache`,
   `cargo check -p reposix-remote`, `cargo check -p reposix-cli`,
   `cargo nextest run -p <crate>`. Pre-push hook runs the workspace-wide
   gate; phase tasks never duplicate.
3. **Sequential execution.** Tasks T01 → T02 → T03 → T04 — never parallel,
   even though T02 (cache + remote) and T03 (cli) touch different crates.
   CLAUDE.md "Build memory budget" rule is "one cargo invocation at a
   time" — sequencing the tasks naturally honors this.
4. **L1-strict delete trade-off RATIFIED (D-01).** The plan body, the
   inline comment in `precheck.rs`, and CLAUDE.md all carry the same
   verbatim trade-off statement.
5. **Both push paths use the same L1 mechanism (DVCS-PERF-L1-03).** No
   path-specific copies. `precheck_export_against_changed_set` lives in
   `crates/reposix-remote/src/precheck.rs` so both `handle_export` (P81)
   and the future bus handler (P82–P83) consume the same code path.
6. **`last_fetched_at` cursor is meta-table, not new table (S1).** Two
   thin Cache wrappers (`read_last_fetched_at`, `write_last_fetched_at`)
   over the existing `meta::get_meta`/`set_meta` API keyed by
   `"last_fetched_at"`. NO new table; NO new SQL DDL.
7. **Per-phase push BEFORE verifier (CLAUDE.md "Push cadence — per-phase",
   codified 2026-04-30).** T04 ends with `git push origin main`; pre-push
   gate must pass; verifier subagent grades the three catalog rows
   AFTER push lands. Verifier dispatch is an orchestrator-level action
   AFTER this plan completes — NOT a plan task.
8. **CLAUDE.md update in same PR (QG-07; D-05).** T04 documents
   `reposix sync --reconcile` (§ Commands) + the L1 cost-vs-correctness
   trade-off (§ Architecture, citing `architecture-sketch.md`).
9. **First-push fallback (S2; D-02).** When `last_fetched_at` is `None`,
   the helper falls through to the existing `list_records` walk for
   THIS push only, then writes the cursor. NOT `epoch`-fallback. Surfaced
   via `tracing::info!` (single log line, NOT a hot path at scale).
10. **Performance regression test with positive control.** N=200 records
    via wiremock harness; counter-based assertion (`expect(0)` for
    `list_records`, `expect(1+)` for `list_changed_since`); positive-control
    test included as a sibling that flips `expect(0)` to `expect(1)` and
    confirms wiremock fails RED if the matcher is reverted (closes the
    MEDIUM risk in RESEARCH.md § Pitfalls and Risks).
11. **No new error variants.** Per the existing `Cache::log_*` family
    pattern, cursor-write failure WARN-logs and does NOT poison the push
    ack. NO new `RemoteError` variant nor new `cache::Error` variant.

# Threat model crosswalk

Per CLAUDE.md § "Threat model" — this phase introduces NO new
trifecta surface. The L1 migration changes WHICH REST endpoint the
helper hits but does not introduce a new HTTP construction site:

| Existing surface              | What P81 changes                                                                                                                                                                                                                                                                |
|-------------------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Helper outbound HTTP          | UNCHANGED — `list_changed_since` is already implemented in all 4 connectors (`reposix-sim`, `reposix-confluence`, `reposix-github`, `reposix-jira`); no new HTTP call site is introduced. The same `client()` factory + `REPOSIX_ALLOWED_ORIGINS` allowlist applies. |
| Cache prior parse (Tainted bytes) | NEW: the precheck reads `cache.read_blob_cached(prior_oid)` (NEW sync gix-only primitive — H1 fix; returns `Ok(Some(Tainted<Vec<u8>>))` or `Ok(None)` on cache miss; does NOT touch the backend). Parsing tainted bytes is fine (no I/O side effects), but care is needed not to leak the tainted body into a log line. STRIDE category: Information Disclosure — mitigated by reusing the existing `log_helper_push_rejected_conflict` shape (records `id + versions` only; never echoes blob body). |
| Cursor write (`last_fetched_at`) | NEW: writes a single-row SQL upsert into `meta`. SQLite autocommit makes this atomic. Best-effort semantics match P80's `write_mirror_synced_at`. STRIDE category: Tampering — mitigated by the existing `meta::set_meta` API (parameterized SQL; no string concatenation). |
| Push reject diagnostics       | UNCHANGED — same `log_helper_push_rejected_conflict` shape with id + versions; no new bytes leak.                                                                                                                                                                              |

No `<threat_model>` STRIDE register addendum required beyond the three
threats the plan body's `<threat_model>` section enumerates per CLAUDE.md
template requirements:

- **T-81-01 (Tampering — cursor):** `meta.set_meta` parameterized SQL.
- **T-81-02 (Information Disclosure — Tainted prior bytes):** existing
  log_helper_push_rejected_conflict shape preserved (no body bytes leak).
- **T-81-03 (Denial of Service — false-positive on own-write race after
  cursor tick):** documented as a known L1 quirk (RESEARCH.md § Pitfall 2);
  self-healing on next push via `find_oid_for_record` returning the
  just-synced version. No new mitigation needed.
