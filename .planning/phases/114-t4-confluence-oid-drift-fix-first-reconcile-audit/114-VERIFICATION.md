---
phase: 114-t4-confluence-oid-drift-fix-first-reconcile-audit
verified: 2026-07-15T17:39:22Z
status: verified
score: all success criteria verified — 4/4 artifact-verifiable + SC1+SC2 real-backend GREEN (coordinator-run 2026-07-15T17:56Z)
overrides_applied: 0
re_verification:
  previous_status: none
  note: initial verification
human_verification:  # RESOLVED 2026-07-15T17:56Z — both SC1 + SC2 run live and PASS; see § Real-backend acceptance below.
  - test: "SC1 — `reposix init confluence::TokenWorld /tmp/x && cd /tmp/x && git checkout -B main refs/reposix/origin/main` (start-point ref materializes blobs so the drift check runs) against live Confluence TokenWorld (including page 7766017)"
    expected: "checkout completes with ZERO `Error::OidDrift` abort"
    why_human: "Requires live ATLASSIAN_* creds + a non-default REPOSIX_ALLOWED_ORIGINS and the real TokenWorld substrate; explicitly out of this verifier's scope (real-backend, coordinator-run). RISK (OQ1/Assumption A1): the fix closes drift for ADF-native pages ONLY — the LIST path now requests body-format=atlas_doc_format with NO storage fallback while get_record DOES fall back to body-format=storage; a pre-ADF (storage-only) page would still trip OidDrift. If the live run aborts on a page id OTHER than 7766017, that page is very likely pre-ADF — file a list-path storage-fallback GOOD-TO-HAVE, do not claim universal Confluence compatibility."
  - test: "SC2 — `bash quality/gates/agent-ux/t4-conflict-rebase-ancestry-real-backend.sh` against live Confluence TokenWorld (refresh mirror first per plan: `bash scripts/refresh-tokenworld-mirror.sh`)"
    expected: "gate exits 0 (GREEN)"
    why_human: "P0 real-backend gate; needs live creds + substrate. Same pre-ADF residual risk as SC1."
---

# Phase 114: t4 Confluence oid-drift fix-first + reconcile audit — Verification Report

**Phase Goal:** Confluence page checkouts no longer abort on the list-vs-get oid-drift product defect, and the `sync --reconcile` recovery claim is proven true or corrected to its real scope.
**Verified:** 2026-07-15T17:39:22Z
**Status:** verified — all SCs GREEN (SC1+SC2 real-backend PASS, coordinator-run 2026-07-15T17:56Z; see § Real-backend acceptance below)
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
| --- | --- | --- | --- |
| SC1 | Live TokenWorld `git checkout -B main refs/reposix/origin/main` (incl. page 7766017) completes with zero oid-drift abort | ✓ GREEN | Real-backend PASS (coordinator-run 2026-07-15T17:56Z) — see § **Real-backend acceptance (SC1/SC2)** below: checkout exit 0, `pages/7766017.md` 664 bytes non-empty, all 3 pages materialized, NO `OidDrift`/`written_oid != oid`. Page 7766017 is ADF-native (not pre-ADF); OQ1 residual did NOT manifest on the current substrate. |
| SC2 | P0 gate `agent-ux/t4-conflict-rebase-ancestry-real-backend` GREEN against live TokenWorld | ✓ GREEN | Real-backend PASS (coordinator-run 2026-07-15T17:56Z) — see § **Real-backend acceptance (SC1/SC2)** below: gate exit 0, JSON artifact `status: PASS`, `asserts_failed: []` (7 asserts). |
| SC3 | Root-cause fix lands as adapter render-parity (`list_records`==`get_record` representation) — NOT a workaround, NOT a weakened drift check, NOT per-page GETs in the list walk | ✓ VERIFIED | `client.rs:202` LIST url now `...pages?limit={}&body-format=atlas_doc_format`; `lib.rs:256` `get_record` uses identical `?body-format=atlas_doc_format`; `builder.rs` `written_oid != oid` count == 1 (drift check untouched); no per-page GET added to `list_issues_impl`. |
| SC4 | `Error::OidDrift` / `--reconcile` doc comments accurately name which drift class `reposix sync --reconcile` recovers vs the systematic class it does NOT — verified against a reproduction, not assumed | ✓ VERIFIED | `error.rs:62-81` names both causes with OPPOSITE recovery answers; `sync.rs` module+run docs add `systematic` caveat (count 2); `main.rs:177` `Sync` clap doc scoped. Backed by `oid_drift_reconcile.rs` reconcile-non-recovery test (ran GREEN). |
| T1 (Wave-1) | `list_records` and `get_record` request the SAME Confluence body representation so an unmutated ADF-native page renders byte-identical bytes on both paths | ✓ VERIFIED | Both request `body-format=atlas_doc_format`; both decode through the SAME `translate()`; `list_and_get_render_parity` asserts `recs[0].body == got.body` AND `!recs[0].body.is_empty()` — ran GREEN. |
| T2 (Wave-2) | A backend-agnostic mock proves divergent list/get bodies for the SAME id make `read_blob` abort with `Error::OidDrift` | ✓ VERIFIED | `pre_fix_divergent_bodies_trigger_oid_drift` (DriftingMock, `aligned=false`) asserts `Err(OidDrift{issue_id=="1"})` — ran GREEN. |
| T3 (Wave-2) | A second `build_from()` (== `sync --reconcile`) does NOT clear the stale list-derived oid while bodies diverge — reconcile CANNOT recover the systematic class | ✓ VERIFIED | `reconcile_does_not_clear_stale_list_oid_while_bodies_diverge` asserts `oid_a == oid_b` AND `read_blob` still `Err(OidDrift)` — ran GREEN. |

**Score:** ALL success criteria verified. SC3, SC4, and Wave-1/Wave-2 detail truths verified from committed tests; SC1 + SC2 real-backend GREEN on the coordinator run (2026-07-15T17:56Z — see § Real-backend acceptance below).

### Required Artifacts

| Artifact | Expected | Status | Details |
| --- | --- | --- | --- |
| `crates/reposix-confluence/src/client.rs` | `list_issues_impl` LIST url requests `body-format=atlas_doc_format` | ✓ VERIFIED | Line 202 primary fix; defensive `next_url` cursor re-append at 275-300 (separator-aware, no URL-parse dep). |
| `crates/reposix-confluence/src/lib.rs` | `get_record` parity target (`?body-format=atlas_doc_format`) | ✓ VERIFIED | Line 256 (ADF) + line 296 storage fallback for pre-ADF (get-only — the residual-gap asymmetry). |
| `crates/reposix-confluence/tests/contract.rs` | `list_and_get_render_parity` regression test | ✓ VERIFIED | Lines 699-801; LIST mock gated on `query_param("body-format","atlas_doc_format")`; asserts non-empty + parity. Ran GREEN. |
| `crates/reposix-confluence/src/types.rs` | `ConfPage` doc corrected (no longer claims empty list body) | ✓ VERIFIED | Line 111 `body IS populated when a page has ADF content`. |
| `crates/reposix-cli/tests/agent_flow_real.rs` | stale "list bodies are empty" comment corrected | ✓ VERIFIED | Line 247 `list bodies are now populated for ADF-native pages`. |
| `crates/reposix-cache/tests/oid_drift_reconcile.rs` | `DriftingMock` + 3 named tests (min 80 lines) | ✓ VERIFIED | 283 lines; `struct DriftingMock`; all 3 named fns present; ran GREEN (3 passed). |
| `crates/reposix-cache/src/error.rs` | `OidDrift` doc names systematic class + `CANNOT` | ✓ VERIFIED | Lines 62-81; `systematic backend rendering-representation mismatch` + `CANNOT` present; points at reproduction file. |
| `crates/reposix-cli/src/sync.rs` | `--reconcile` doc scoped with `systematic` caveat | ✓ VERIFIED | Module doc (19-26) + `run` fn doc (54-55); `systematic` count == 2. |
| `crates/reposix-cli/src/main.rs` | `Sync` clap doc consistency + `systematic` note | ✓ VERIFIED | Lines 171-180; `systematic` present; stale `Cache::sync`→`Cache::build_from` also corrected. |
| `crates/reposix-cache/src/cache.rs` | `write_last_fetched_at` cursor-drift doc left accurate + untouched | ✓ VERIFIED | Line 581 cursor-drift claim intact; `git diff` shows NO change to cache.rs across the phase range. |

### Key Link Verification

| From | To | Via | Status | Details |
| --- | --- | --- | --- | --- |
| `list_issues_impl` (list body) | `get_record` (get body) | identical `?body-format=atlas_doc_format` request shape | ✓ WIRED | Both grep-confirmed; render-parity test asserts byte-equality. |
| `list_issues_impl` | `translate()` | SAME `translate()` `get_record` uses — no new translation path | ✓ WIRED | `client.rs` list path calls `translate(tainted.into_inner())`; `get_record` calls the same `translate()`. |
| `sync.rs::run (--reconcile)` | `Cache::build_from` | reconcile IS a forced full `build_from` | ✓ WIRED | Actual code call `.build_from()` at `sync.rs:128` (not merely documented). |
| `oid_drift_reconcile.rs` (non-recovery test) | corrected `error.rs`/`sync.rs` doc scope | test empirically backs the SC4 doc claim | ✓ WIRED | `reconcile_does_not_clear_stale_list_oid_while_bodies_diverge` is the reproduction the docs cite by path. |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
| --- | --- | --- | --- |
| Wave-2 reconcile-audit tests pass | `cargo test -p reposix-cache --test oid_drift_reconcile` | 3 passed; 0 failed | ✓ PASS |
| Wave-1 render-parity test passes | `cargo test -p reposix-confluence --test contract list_and_get_render_parity` | 1 passed; 0 failed | ✓ PASS |
| Drift guard NOT weakened | `grep -c 'written_oid != oid' builder.rs` | 1 (unchanged) | ✓ PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
| --- | --- | --- | --- | --- |
| FIX-01 | 114-01 | Confluence list-vs-get render-parity (adapter root-cause fix) | ✓ SATISFIED | SC3 + T1 + render-parity test GREEN. |
| FIX-02 | 114-02 | `sync --reconcile` recovery-claim audit (reproduction-backed doc correction) | ✓ SATISFIED | SC4 + T2/T3 + 3 reconcile tests GREEN + scoped docs. |

_No REQUIREMENTS.md exists for v0.15.0-phases; FIX-01/FIX-02 descriptions taken from ROADMAP + plan frontmatter. No orphaned requirements for this phase._

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| --- | --- | --- | --- | --- |
| `crates/reposix-cli/src/main.rs` | 148 | "the offline read path is not yet implemented; this flag is accepted but currently returns an error" | ℹ️ Info | PRE-EXISTING and OUT OF SCOPE — line last touched by `122ae22` (FUSE-era doc scrub), NOT by any phase-114 commit. It documents the `--offline` flag on the `List` subcommand, unrelated to the phase-114 `Sync` clap-doc change. Honest disclosure of a known-incomplete flag, not a phase-114 stub. |

No blocker or warning anti-patterns in phase-114-touched code. `read_blob`'s coherence guard (Pitfall 1) held; no `return null`/empty-render stubs; no weakened taint path.

### Human Verification Required

> **RESOLVED 2026-07-15T17:56Z (coordinator-run).** Both SC1 and SC2 were run against live
> Confluence TokenWorld and PASS — full evidence in § **Real-backend acceptance (SC1/SC2)**
> below. The items below are retained as the record of what was required.

1. **SC1 — live TokenWorld checkout (incl. page 7766017)**
   - Test: `set -a; . ./.env; set +a; reposix init confluence::TokenWorld /tmp/p114-repro && cd /tmp/p114-repro && git checkout -B main refs/reposix/origin/main` (leaf isolation — `/tmp`, cd in the SAME invocation; the start-point ref materializes blobs so the OidDrift check actually runs — a bare `git checkout -B main` is an empty-branch false-GREEN).
   - Expected: NO `Error::OidDrift` abort.
   - Why human: env-gated live Confluence creds + substrate; coordinator-run.
   - **Watch (OQ1 residual risk):** render-parity closes drift for ADF-native pages ONLY. The LIST path requests `body-format=atlas_doc_format` with NO storage fallback; `get_record` DOES fall back to `body-format=storage` (lib.rs:296). A pre-ADF (storage-only) page therefore still drifts. If the gate aborts on a page id OTHER than 7766017, that page is likely pre-ADF → file a list-path storage-fallback GOOD-TO-HAVE; do NOT claim universal Confluence compatibility. (Assumption A1: TokenWorld has no pre-ADF pages — unproven until this run.)

2. **SC2 — P0 real-backend gate**
   - Test: `set -a; . ./.env; set +a; bash scripts/refresh-tokenworld-mirror.sh && bash quality/gates/agent-ux/t4-conflict-rebase-ancestry-real-backend.sh`.
   - Expected: exit 0 (GREEN).
   - Why human: env-gated live gate; same pre-ADF residual risk as SC1.

### Gaps Summary

No artifact-verifiable gaps. SC3 (adapter render-parity root-cause fix) and SC4 (reproduction-backed `--reconcile` doc scoping) are both fully delivered and independently re-ran GREEN from committed tests — not trusted from SUMMARY narrative. The drift-check coherence guard was NOT weakened (Pitfall 1 held, count == 1). `cache.rs` cursor-drift doc confirmed accurate and byte-untouched.

Two goal truths (SC1, SC2) are real-backend and DEFERRED to the coordinator's live run per the verifier charter — marked PENDING-LIVE-RUN, not graded here. The single carried risk into that run is the OQ1 pre-ADF residual: honestly documented in `error.rs`, `114-01-SUMMARY.md` § Residual gap, and `114-RESEARCH.md` OQ1 — the coordinator must watch for a non-7766017 OidDrift page id.

**SC4 literal-wording note (not a gap):** ROADMAP SC4 names "`builder.rs`/`cache.rs` doc comments," but the actual reconcile-recovery claim-bearing comments live in `error.rs` + `sync.rs` + `main.rs` (114-RESEARCH.md § FIX-02 grep-verified table). `cache.rs:575-592` (`write_last_fetched_at`) is accurate for temporal CURSOR drift, a different class, and was correctly left untouched. The SC4 INTENT — docs accurately describe reconcile's real recovery scope, verified against a reproduction — is fully met; only the literal file names differ, per the authoritative research table.

---

_Verified: 2026-07-15T17:39:22Z_
_Verifier: Claude (gsd-verifier)_

---

## Real-backend acceptance (SC1/SC2) — coordinator-run

**Run:** 2026-07-15T17:56Z · executor: Claude (real-backend acceptance executor) · live Confluence TokenWorld (tenant `reuben-john`, space key `REPOSIX` == display-alias `TokenWorld` == space id 360450).

| SC | Verdict | Evidence |
| --- | --- | --- |
| SC1 — live TokenWorld checkout incl. page 7766017 → zero OidDrift | ✅ **PASS** | `git checkout -B main refs/reposix/origin/main` exited 0; page `7766017.md` materialized non-empty (664 bytes, real ADF frontmatter+body); all 3 pages materialized; `git grep` force-materialized every blob (exit 0). NO `OidDrift` / `written_oid != oid` abort on ANY page. |
| SC2 — `agent-ux/t4-conflict-rebase-ancestry-real-backend` gate | ✅ **PASS** | Gate exit code **0**; artifact `quality/reports/verifications/agent-ux/t4-conflict-rebase-ancestry-real-backend.json` `status: PASS`, `asserts_failed: []` (7 asserts passed). |

### Preflight
- `bash scripts/preflight-real-backends.sh` → **exit 0** (reachable). Confluence TokenWorld PASS, GitHub PASS, JIRA PASS. Creds present in `.env` (ATLASSIAN_API_KEY/EMAIL/TENANT + REPOSIX_ALLOWED_ORIGINS). Live run authorized.

### Mirror refresh PRE-STEP (DRAIN-02 mirror-lag guard)
- `set -a; source .env; set +a; bash scripts/refresh-tokenworld-mirror.sh` → **exit 0**.
  - BEFORE mirror versions: `2818063=version:13 7766017=version:1 7798785=version:4`
  - BACKEND materialized versions: `2818063=version:14 7766017=version:1 7798785=version:4`
  - Push: `26f3bb6..d959287  main -> main` (fast-forward, no `--force`); AFTER == BACKEND (byte-current). Only the GitHub mirror written; NO Confluence backend write.

### SC1 — exact commands + raw evidence
```
# leaf-isolated in /tmp; .env sourced same-invocation; git-remote-reposix on PATH
reposix init confluence::TokenWorld /tmp/p114-sc1-repro
cd /tmp/p114-sc1-repro && git checkout -B main refs/reposix/origin/main   # exit 0
```
Raw:
```
refs/reposix/origin/main @ e73cf02513a6022cce1c51fd2d81fd61d0500714
extensions.partialClone=origin  remote.origin.partialclonefilter=blob:none
Reset branch 'main'; branch 'main' set up to track 'origin/main'.  CHECKOUT_EC=0
PRESENT pages/7766017.md bytes=664
pages/ = 2818063.md  7766017.md  7798785.md
git grep -c '' -- pages/ : 2818063.md:11  7766017.md:11  7798785.md:12   GREP_EC=0
git status --short : (clean)
```
NOTE: a bare `git checkout -B main` (no start-point, as literally written in the SC1 row) creates an EMPTY branch — the content ref `refs/reposix/origin/main` MUST be named as the start-point (per `reposix init`'s own "Next:" hint) for blobs to materialize and the drift check to actually run. Verified both: bare form → empty `pages/`; ref-named form → full materialization, zero drift.

- **Aborting page id: NONE.** Page `7766017` (version:1) is ADF-native (renders 664 bytes of real body on BOTH list and get paths post-fix) — NOT pre-ADF. No page in the TokenWorld walk (2818063 / 7766017 / 7798785) tripped `OidDrift`. OQ1 residual (pre-ADF list-path storage-fallback gap) did NOT manifest on the current TokenWorld substrate.

### SC2 — exact command + raw evidence
```
set -a; source .env; set +a
bash quality/gates/agent-ux/t4-conflict-rebase-ancestry-real-backend.sh   # exit 0
```
Gate transcript highlights (all PASS): two independent caches (A/B) bootstrapped · A baseline push succeeded (`* [new branch] main -> main`) · B stale-base push correctly REJECTED (version mismatch / fetch first) · B refetch preserved ancestry — root commit IDENTICAL before/after (HIGH-1 stays fixed) · ref advanced non-vacuously (commit count=2). Only page `2818063` edited in place (safe single-file diff, no deletions, no protected fixture ids). JSON artifact: `status: PASS`, `exit_code: 0`, `asserts_failed: []`.

### Protected-fixture safety
- **`7766017` and `7798785` NOT deleted or mutated.** SC1 checkout was read-only. SC2 gate edited ONLY `2818063` and its mass-delete/protected-id guard passed (`no protected fixture ids touched`). Mirror refresh carried the pair verbatim (`7766017=version:1 7798785=version:4` unchanged BEFORE→AFTER). Confirmed intact.

**Both real-backend success criteria PASS. Phase 114 T4 real-backend acceptance is GREEN.** (Uncommitted — coordinator stages.)
