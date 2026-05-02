# Per-question findings

← [back to index](./index.md)

### 1. Will execution actually deliver the phase goal?

**Mostly YES, with caveats.**

The architectural understanding is correct: refs live in the cache's bare repo (`Cache::repo`), the helper's `stateless-connect` tunnels `git upload-pack --advertise-refs` against that bare repo, vanilla `git fetch` from the working tree pulls all advertised refs. I verified `crates/reposix-remote/src/stateless_connect.rs:196` (`send_advertisement(proto, cache.repo_path())`) — `git upload-pack` advertises every non-hidden ref, so `refs/mirrors/*` automatically propagates without a code change, since `transfer.hideRefs` is configured to hide ONLY `refs/reposix/sync/` (`crates/reposix-cache/src/cache.rs:102–113`). RESEARCH.md A3's "one-line edit" is correctly downgraded by the plan to "possibly zero-line; investigate first" (T03 step 3a).

The cache-as-source-of-truth point is well documented in S1 of the overview and the `mirror_refs.rs` module-doc.

**Caveat (HIGH):** the goal is "vanilla `git fetch` brings these refs along" — but the verifier shells that prove this run `reposix init sim::demo` against a sim bound to a non-default port and CANNOT reach it (see Issue H1 below). The integration tests in Rust have the same flaw. The plan's machinery for connecting the working tree to the helper is broken at the harness layer, which means SC3 cannot pass without a fix.

### 2. All 6 ROADMAP success criteria covered?

| SC | Coverage | Notes |
|----|----------|-------|
| SC1 | YES | `mirror_refs.rs` writers + readers; constants + name formatters re-exported from `lib.rs`; namespace `refs/mirrors/<sot-host>-{head,synced-at}`. |
| SC2 | YES | `handle_export` success branch (lines 470–489 wiring); audit row written via `log_mirror_sync_written`. |
| SC3 | YES (with H1 caveat) | Integration test `vanilla_fetch_brings_mirror_refs` + verifier `mirror-refs-readable-by-vanilla-fetch.sh`. Both blocked by H1. |
| SC4 | PARTIAL (M3) | Reject hint cites refs + RFC3339 + (N minutes ago). The Q2.2 verbatim clarification phrase ("synced-at is the timestamp the mirror last caught up to confluence — NOT a 'current SoT state' marker") is carried into CLAUDE.md as a doc-clarity contract carrier, NOT into the reject stderr. The ROADMAP wording is ambiguous about whether the verbatim phrase MUST appear in stderr; the plan's reading is defensible but worth flagging — see Issue M3. |
| SC5 | YES | T01 mints rows + verifiers FIRST (in the same atomic commit, BEFORE T02 implementation); CLAUDE.md update lands in T04 same PR. |
| SC6 | YES | T04 ends with `git push origin main`; verifier subagent dispatch is orchestrator-level after push. |

### 3. Catalog-first invariant respected?

**YES.** T01 mints 3 rows in `quality/catalogs/agent-ux.json` with `status: FAIL` BEFORE any Rust changes. The runner re-grades to PASS at T04 BEFORE the terminal push. The hand-edit pathway is correctly annotated as "NOT Principle A" (Principle A applies to docs-alignment dim only); each row's `_provenance_note` references GOOD-TO-HAVES-01. Atomic commit shape matches P79's precedent.

### 4. OP-3 audit row UNCONDITIONAL?

**FUNCTIONALLY YES, but defensively WEAK.** The plan's wiring at lines 1605–1619 calls `cache.write_mirror_head` and `cache.write_mirror_synced_at` (each best-effort with WARN-on-fail; no `?` propagation), then UNCONDITIONALLY calls `cache.log_mirror_sync_written`. Code flow guarantees the audit row writes whether or not the ref writes succeeded.

However: the audit-row write is AFTER the ref-write attempts. If a future refactor changes ref-writes to propagate errors (`?`), the audit row would silently be lost. The defensive form is to write the audit row FIRST (or wrap in panic-recovery via `std::panic::catch_unwind`). The current shape is consistent with the existing `log_helper_push_accepted` precedent (line 471), so this is not strictly out-of-pattern, but the plan should add an inline comment ("audit row MUST remain unconditional even if ref writes are upgraded to propagate errors") to prevent future regression.

### 5. Cargo discipline?

**YES.** Every cargo invocation in the plan is per-crate (`-p reposix-cache` / `-p reposix-remote`). T02 → T03 → T04 are strictly sequential. The plan explicitly cites CLAUDE.md "Build memory budget" multiple times. Pre-push hook handles the workspace-wide gate; phase tasks never duplicate it. No parallel cargo invocations.

### 6. Threat model integrity?

**YES.** The `<threat_model>` section enumerates 3 STRIDE threats:
- T-80-01 Tampering (ref name composition) → mitigated by `gix::refs::FullName::try_from` + controlled `state.backend_name` enum.
- T-80-02 Information Disclosure (misleading staleness window) → mitigated by Q2.2 doc-clarity contract.
- T-80-03 DoS (reflog growth) → accepted, deferred to v0.14.0.

No new HTTP origin, no new `Tainted<T>` propagation path, no new shell-out. Refs are written using attacker-uninfluenced data: timestamps from `chrono::Utc::now()` (local clock), `sot_sha` from `cache.build_from()` (helper-derived synthesis-commit OID, not attacker-influenced — see plan S2), `sot_host` slug from controlled `state.backend_name` enum. The annotated tag's message body is RFC3339 + a fixed prefix — no shell escape risk. All correct.

The plan correctly resolves the RESEARCH A2 ambiguity (`parsed.commits.last()` does not exist; `ParsedExport` has `commit_message`, `blobs`, `tree`, `deletes`) by using `cache.build_from()` post-write — this is verified against `crates/reposix-remote/src/fast_import.rs:72-81`.

### 7. Plan size + executor context budget

**LARGE BUT NAVIGABLE.** 2,259 lines is at the high end. The plan is a single phase with 4 sequential tasks; the executor reads it linearly. The size is dominated by:

- Three full verifier-shell scripts inlined verbatim (~225 lines).
- The full `mirror_refs.rs` module body inlined (~280 lines).
- The full `handle_export` wiring deltas inlined twice (success + reject; ~150 lines).
- Four full integration-test bodies inlined (~250 lines).
- A long `<canonical_refs>` block (~110 lines) duplicated against 80-RESEARCH.md.

**Trim candidates (~400 lines, no executor signal lost):**

- `<canonical_refs>` block (lines 356–469) duplicates 80-RESEARCH.md's "Sources" section; the plan could replace ~70% of these with a single "see RESEARCH.md § Sources for the full citation map" referral.
- The Architecture preamble in `<objective>` (lines 53–137) duplicates 80-PLAN-OVERVIEW.md § "Subtle architectural points (S1, S2)" almost verbatim; one of the two is sufficient — keep the OVERVIEW version, trim the PLAN version to a 5-line summary.
- Risks table in 80-PLAN-OVERVIEW.md (lines 239–249) duplicates content in the 80-01 plan's `<must_haves>` and `<threat_model>` — the OVERVIEW could narrow to a 3-row table covering only the LOW-likelihood items not already addressed in tasks.

That said: the plan is for a SINGLE executor reading linearly through 4 sequential tasks, with no parallelism, no separate plan files. The redundancy is friction, not a blocker. Estimate: trimming 400 lines saves ~5% executor context; not worth re-revising for that alone unless other revisions are required.

### 8. Open questions Q1–Q6 — deferrable to executor verification?

| Q | Topic | Acceptable to defer? | Notes |
|---|-------|---------------------|-------|
| Q1 | gix `Repository::tag` API name at gix 0.83 | NO (HIGH H2) | Plan writes a concrete invocation (lines 1132–1142) with WRONG argument order. Verified gix 0.83 signature; the plan's call won't compile. The "executor verifies via cargo check" hedge is too soft — at minimum the plan should cite the correct signature OR commit to the fallback path (`write_object` + `tag_reference`) up front. |
| Q2 | `cache.repo.committer()` accessor name | YES | Confirmed at gix 0.83 `repository::identity::committer() -> Option<Result<SignatureRef<'_>, _>>`. Borrows from `&self`. The plan's `.expect(...).map(|c| c.into())` chain may need adjustment but is bounded; executor diagnoses on `cargo check`. |
| Q3 | `stateless_connect.rs` advertisement widening | YES (zero-line) | Confirmed: `send_advertisement` calls `git upload-pack --advertise-refs` against `cache.repo_path()`. `transfer.hideRefs` only hides `refs/reposix/sync/`. Mirror refs propagate automatically. Plan's "investigate first" stance is correct. |
| Q4 | cache.db path layout | NO (M2) | Integration test at line 1967 has `let db_path = cache_bare.join("..").join("cache.db");` — but `db.rs:35–37` puts cache.db INSIDE cache_dir (which IS the bare repo dir). The `..` is wrong. Easy fix; should be `cache_bare.join("cache.db")`. Listed as M-severity below. |
| Q5 | reject-hint integration test shape (weaker form) | NO (HIGH H3) | The "weaker form" asserts nothing meaningful (see issue H3). |
| Q6 | verifier shell two-WORK-tree scenario | YES | Brittleness is real; the plan flags it and proposes a relocation if needed. The hint-text contract (stderr cites refs/mirrors/sim-synced-at + RFC3339 + N minutes ago) is the load-bearing assertion and is testable in either shape. |

### 9. Per-phase push cadence

**YES.** T04 ends with `git push origin main` BEFORE the verifier-subagent dispatch. The dispatch is correctly identified as orchestrator-level (top-level coordinator action AFTER 80-01 T04 pushes), NOT a plan task. Pre-push GREEN is part of phase-close criterion. `--no-verify` and `--amend` correctly forbidden.

### 10. Phase-shape + dependency chain

**SOUND.** T01 → T02 → T03 → T04 strictly sequential.

- T01 (catalog-first): doc-only; produces verifier shells that FAIL until T02–T04 land. The shells reference test files not yet created — acceptable per QG-06 catalog-first contract.
- T02 (cache impl): depends on nothing (T01 doc-only). Cargo-locked on `reposix-cache`.
- T03 (helper wiring): requires T02's `Cache::write_mirror_head` / `write_mirror_synced_at` / `read_mirror_synced_at` / `refresh_for_mirror_head` / `log_mirror_sync_written` APIs. Cargo-locked on `reposix-remote`. T02 must be on disk first — correctly serialized.
- T04 (integration tests + flip + close): requires T03 wiring + T02 unit tests pass. Cargo-locked on `reposix-remote --tests`. Sequential after T03.

No circular dependencies, no skipped-prerequisite relations. Wave plan correctly identifies a single wave.
