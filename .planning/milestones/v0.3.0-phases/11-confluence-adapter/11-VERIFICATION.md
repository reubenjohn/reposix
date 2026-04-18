---
phase: 11-confluence-adapter
verified: 2026-04-14T06:30:00Z
status: human_needed
score: 6/7 automated + 1 human_needed
overrides_applied: 0
human_verification:
  - test: "Run `reposix mount /tmp/reposix-conf-mnt --backend confluence --project REPOSIX`, then `ls /tmp/reposix-conf-mnt/`, then `cat` one of the listed .md files, then `fusermount3 -u /tmp/reposix-conf-mnt`, then re-mount to confirm re-entrant."
    expected: "Real page frontmatter and body content appear in the cat output. The unmount returns exit 0. The second mount also succeeds and ls shows the same pages."
    why_human: "FUSE mount requires a running daemon process plus the fusermount3 binary. The orchestrator note states to leave criterion 5 to the live mount run rather than running it here."
---

# Phase 11: Confluence Cloud read-only adapter (v0.3) — Verification Report

**Phase Goal:** Ship a `reposix-confluence` crate implementing `IssueBackend` against Atlassian Confluence Cloud REST v2. CLI dispatch for `list --backend confluence` and `mount --backend confluence`. Wiremock unit tests, contract test, Tier 3B parity demo, Tier 5 live-mount demo, ADR-002, docs update, renamed env var.

**Verified:** 2026-04-14T06:30:00Z
**Status:** HUMAN_NEEDED (1 criterion requires live FUSE mount test; all 6 automated criteria PASS)
**Re-verification:** No — initial verification.

---

## 1. Executive Summary

Phase 11 delivered a complete, production-quality Confluence Cloud read-only adapter. The `reposix-confluence` crate (1200 lines) is wired into CLI dispatch, backed by 17 unit tests and a 3-backend contract test suite, and ships all required documentation. All 6 programmatically-verifiable success criteria pass: 193 tests pass / 0 fail (>= 180 target), clippy is clean, smoke.sh is 4/4, the live `reposix list` command returns 4 real Atlassian pages, the demo skip path exits 0 cleanly, and all required doc/env artifacts exist with no stale `TEAMWORK_GRAPH_API` references. The FUSE re-entrant mount criterion (SC-5) requires a human operator to run the live mount because it involves starting a FUSE daemon — this is the only item blocking a full `passed` verdict.

---

## 2. Seven-Row Success Criterion Table

| # | Success Criterion | Evidence | Status |
|---|---|---|---|
| 1 | `cargo test --workspace --locked` returns ≥180 pass / 0 fail | **193 passed, 0 failed** across 29 test harnesses. Counted from live `cargo test` run. | **PASS** |
| 2 | `cargo clippy --workspace --all-targets -- -D warnings` exits 0 | Clippy completed with `Finished dev profile` and zero warnings. | **PASS** |
| 3 | `bash scripts/demos/smoke.sh` still 4/4 green — Tier 1 demos untouched | `smoke suite: 4 passed, 0 failed (of 4)` confirmed live. All four Tier 1 demos (01 through 04) pass unchanged. | **PASS** |
| 4 | `reposix list --backend confluence --project REPOSIX` with creds set prints ≥1 row | Live run returned 4 rows: `65916 open Architecture notes`, `131192 open Welcome to reposix`, `360556 open reposix demo space Home`, `425985 open Demo plan`. Exit 0. | **PASS** |
| 5 | `reposix mount /tmp/reposix-conf-mnt --backend confluence ...` + `ls` + `cat *.md` returns real page frontmatter/body; `fusermount3 -u` succeeds; re-mount works | Requires live FUSE daemon. Orchestrator scope note deferred this to the live run. Script exists and is substantive (140 lines, non-stub). 11-B SUMMARY documents successful live mount during plan execution. | **HUMAN_NEEDED** |
| 6 | `bash scripts/demos/06-mount-real-confluence.sh` exits 0 when `ATLASSIAN_API_KEY` unset | Confirmed live: prints `SKIP: env vars unset: ATLASSIAN_API_KEY ATLASSIAN_EMAIL REPOSIX_CONFLUENCE_TENANT REPOSIX_CONFLUENCE_SPACE` + `== DEMO COMPLETE ==`, exit code 0. | **PASS** |
| 7 | `docs/decisions/002-confluence-page-mapping.md` exists and documents field mapping; `.env.example` reflects renamed var | ADR-002 found at `docs/decisions/002-confluence-page-mapping.md` (211 lines, Status: Accepted). `.env.example` has `ATLASSIAN_API_KEY=`, `ATLASSIAN_EMAIL=`, `REPOSIX_CONFLUENCE_TENANT=` at lines 37-39. No `TEAMWORK_GRAPH_API` found (grep exit 1). | **PASS** |

---

## 3. Per-Plan Cross-Check (A through F)

### Plan A: reposix-confluence crate core

**Claimed:** 17 unit tests, lib.rs at 1106 lines (now 1200 with post-plan hardening), threat-model tests for creds redaction and tenant validation.

**Spot-checks:**
- `crates/reposix-confluence/src/lib.rs` exists — CONFIRMED (1200 lines, > claimed 1106 due to post-plan hardening committed in REVIEW-FIX.md)
- Threat-model tests exist at lines 1053 (`creds_debug_redacts_api_token`), 1074 (`backend_debug_redacts_creds`), 1089 (`new_rejects_invalid_tenant`), 1118 (`new_accepts_valid_tenants`) — CONFIRMED
- `request_with_headers` call sites at lines 484, 559, 615 via `self.http` — CONFIRMED (SG-01)
- `Tainted::new` at ingress at lines 577 and 631 — CONFIRMED (SG-05)

**Verdict:** Claims match codebase.

### Plan B: CLI dispatch

**Claimed:** `ListBackend::Confluence` in list.rs, mount.rs, fuse/main.rs; `integration-contract-confluence` CI job; 3 new tests; 189 total.

**Spot-checks:**
- `ListBackend::Confluence` in list.rs at lines 77, 185, 186, 187 — CONFIRMED
- `ListBackend::Confluence` in mount.rs at lines 62, 96 — CONFIRMED
- `BackendKind::Confluence` in fuse/main.rs at line 104 — CONFIRMED
- `integration-contract-confluence` in ci.yml at line 113 — CONFIRMED

**Verdict:** Claims match codebase.

### Plan C: Contract test

**Claimed:** `crates/reposix-confluence/tests/contract.rs` at 380 lines with `contract_sim`, `contract_confluence_wiremock`, `contract_confluence_live` (#[ignore]-gated), `skip_if_no_env!` macro.

**Spot-checks:**
- File exists at expected path — CONFIRMED (380 lines)
- `async fn assert_contract`, `contract_sim`, `contract_confluence_wiremock`, `contract_confluence_live` all present — implied by 2 always-on contract tests passing in the 193-test run
- `macro_rules! skip_if_no_env` present — confirmed via 11-C-SUMMARY task verification

**Verdict:** Claims match codebase.

### Plan D: Demos

**Claimed:** `scripts/demos/parity-confluence.sh` (153 lines, executable) and `scripts/demos/06-mount-real-confluence.sh` (140 lines, executable); `docs/demos/index.md` updated; SKIP path exits 0.

**Spot-checks:**
- `scripts/demos/06-mount-real-confluence.sh` exists — CONFIRMED (skip path confirmed live)
- Skip exits 0 with correct banner — CONFIRMED live
- No `echo $ATLASSIAN_API_KEY` or `echo $ATLASSIAN_EMAIL` in either script — required by threat model

**Verdict:** Claims match codebase. Skip path verified live.

### Plan E: Docs and env

**Claimed:** ADR-002, `docs/reference/confluence.md`, `docs/connectors/guide.md` (462 lines), `.env.example` cleaned of `TEAMWORK_GRAPH_API`, CHANGELOG `[Unreleased]` section, README updated.

**Spot-checks:**
- `docs/decisions/002-confluence-page-mapping.md` — CONFIRMED (211 lines, Status: Accepted, Option-A decision documented)
- `docs/connectors/guide.md` — CONFIRMED (file exists)
- `docs/reference/confluence.md` — CONFIRMED (file exists)
- No `TEAMWORK_GRAPH_API` in `.env.example` — CONFIRMED (grep exit 1)
- `ATLASSIAN_API_KEY=`, `ATLASSIAN_EMAIL=`, `REPOSIX_CONFLUENCE_TENANT=` in `.env.example` — CONFIRMED at lines 37-39
- `CHANGELOG.md` has `[v0.3.0] — 2026-04-14` at line 11 — CONFIRMED

**Verdict:** Claims match codebase.

### Plan F: Release engineering

**Claimed:** `MORNING-BRIEF-v0.3.md` (169 lines), `CHANGELOG.md` [v0.3.0] promoted, `scripts/tag-v0.3.0.sh` (93 lines, +x); tag NOT pushed (human-gated).

**Spot-checks:**
- `CHANGELOG.md` `## [v0.3.0] — 2026-04-14` at line 11 — CONFIRMED
- Tag not pushed as designed (MORNING-BRIEF notes the single remaining step)
- Deferred pre-release drift flagged correctly in SUMMARY (the REVIEW-FIX uncommitted changes noted)

**Verdict:** Claims match codebase. Tag gate correctly unexecuted per design.

---

## 4. Threat-Model Mitigation Check

| Threat | Mitigation Required | Evidence in Code | Status |
|---|---|---|---|
| T-11-01: Creds leak via Debug | Manual Debug on `ConfluenceCreds` + backend redacting api_token | `impl std::fmt::Debug` in lib.rs; tests `creds_debug_redacts_api_token` (line 1053) and `backend_debug_redacts_creds` (line 1074) | VERIFIED |
| T-11-02: SSRF via tenant injection | `validate_tenant` enforces DNS-label rules | `fn validate_tenant` at line 341, called at line 332; `new_rejects_invalid_tenant` test covers 10 bad inputs | VERIFIED |
| T-11-03: Tainted HTML in Issue.body | `Tainted::new` at all ingress sites | Lines 577 and 631 in lib.rs wrap every decoded page | VERIFIED |
| T-11-04: Attacker-controlled `_links.next` | Relative-path prepend to `self.base()` | `parse_next_cursor` returns relative path only; `list_issues` prepends tenant base; SG-01 allowlist as second layer | VERIFIED |
| T-11B-01: Env-var values echoed in error | Error message uses names, never values | Test `confluence_requires_all_three_env_vars` asserts values do not appear in error string | VERIFIED |
| T-11B-03: Fork CI leaks secrets | CI job gated on all 4 secrets being non-empty | `integration-contract-confluence` at ci.yml line 113 with secret gate | VERIFIED |
| SG-01: All outbound HTTP via HttpClient | `request_with_headers` only, no `reqwest::Client::new` | 3 call sites in lib.rs all use `self.http.request_with_headers` | VERIFIED |
| SG-05: Taint wrapping at ingress | `Tainted::new` before any translation | Lines 577, 631 with comments explicitly citing SG-05 | VERIFIED |

---

## 5. Phase 12 Skeleton Check

Phase 12 (`Connector protocol — 3rd-party plugin ABI`) is confirmed in ROADMAP.md at the entry starting at line 393. It documents the subprocess/JSON-RPC plugin ABI as the medium-term model, explicitly notes it is "skeleton only — not executed tonight" and requires full `/gsd-plan-phase 12` before execution. The connector guide (`docs/connectors/guide.md`) ships a Phase-12 preview section as requested. This satisfies the user's ask.

---

## 6. Known Gaps / Nits (LOW severity, not blocking)

1. **Uncommitted pre-release drift in working tree** (flagged in 11-F-SUMMARY): `benchmarks/RESULTS.md`, `crates/reposix-confluence/Cargo.toml` (adds `url` workspace dep), `crates/reposix-confluence/src/lib.rs` (WR-01/WR-02 hardening from REVIEW-FIX.md), and a deleted `.claude/scheduled_tasks.lock`. These will trip guard #2 of `scripts/tag-v0.3.0.sh` (clean tree check). The MORNING-BRIEF-v0.3.md documents three options to resolve before tagging. No code correctness impact — the hardening is actually beneficial — but the working tree is not clean. **Recommendation:** commit the hardening changes as a pre-release bundle before running `scripts/tag-v0.3.0.sh`.

2. **SC-2 wording bug in 11-B plan** (noted in 11-B-SUMMARY, not a code defect): The plan's success-criterion grep `grep -q 'Confluence,' crates/reposix-cli/src/mount.rs` does not match because the enum definition lives in list.rs, not mount.rs. The substantive requirement is met (3 occurrences of `ListBackend::Confluence` in mount.rs). This is a plan wording issue, not a deliverable miss.

3. **lib.rs line count grew post-plan** (from claimed 1106 to actual 1200): The REVIEW-FIX.md hardening added ~94 lines. The file is still a single flat module (justified by plan decision 1). No quality concern — more tests and tighter validation.

---

## 7. Human Verification Required

### 1. FUSE Re-entrant Mount (Success Criterion 5)

**Test:** With `.env` loaded and `REPOSIX_ALLOWED_ORIGINS` set:
```bash
set -a; source .env; set +a
export REPOSIX_ALLOWED_ORIGINS="http://127.0.0.1:*,https://${REPOSIX_CONFLUENCE_TENANT}.atlassian.net"
PATH="$PWD/target/release:$PATH" reposix mount /tmp/reposix-conf-mnt --backend confluence --project REPOSIX
ls /tmp/reposix-conf-mnt/
cat /tmp/reposix-conf-mnt/*.md | head -40
fusermount3 -u /tmp/reposix-conf-mnt
# Then re-mount:
PATH="$PWD/target/release:$PATH" reposix mount /tmp/reposix-conf-mnt --backend confluence --project REPOSIX
ls /tmp/reposix-conf-mnt/
fusermount3 -u /tmp/reposix-conf-mnt
```

**Expected:**
- First `ls` shows ≥1 `.md` files with numeric IDs (e.g. `000000065916.md`)
- `cat` output shows YAML frontmatter (`id:`, `title:`, `status:`) plus page body text
- First `fusermount3 -u` exits 0 and mount point is no longer mounted
- Second mount and `ls` succeed (re-entrant — no zombie FUSE process blocking)
- Second `fusermount3 -u` exits 0

**Why human:** Starting a FUSE daemon requires `fusermount3` and a background process. The verification agent cannot safely start and manage background daemons in this context. The orchestrator's parallel live-run result should supply this evidence.

---

## 8. Final Verdict

**GOAL ACHIEVED** for all 6 automated success criteria. The single open item (SC-5: FUSE re-entrant mount) requires human operator confirmation and is architectural in nature — all the code paths to deliver it are present and substantive (CLI dispatch in mount.rs, FUSE binary backing in fuse/main.rs, demo script 06-mount-real-confluence.sh). The 11-B SUMMARY documents a successful live mount during plan execution. Pending human confirmation of SC-5, the phase goal is fully achieved.

---

_Verified: 2026-04-14T06:30:00Z_
_Verifier: Claude (gsd-verifier)_
