# P80 verdict — Mirror-lag refs (`refs/mirrors/<sot-host>-{head,synced-at}`)

**Verifier:** unbiased subagent · zero session context
**Verdict:** **GREEN** (with two advisory items — see § Advisory items)
**Date:** 2026-05-01
**Milestone:** v0.13.0 — DVCS over REST
**Precondition:** P79 verdict GREEN at `quality/reports/verdicts/p79/VERDICT.md` (confirmed)

---

## Catalog row grading

All 3 P80 catalog rows in `quality/catalogs/agent-ux.json` were re-graded
by invoking each verifier script directly from a fresh shell (zero
session context):

| Catalog row                                   | Verifier exit | Asserts | Status   |
| --------------------------------------------- | ------------- | ------- | -------- |
| `agent-ux/mirror-refs-write-on-success`       | 0             | 4/4     | **PASS** |
| `agent-ux/mirror-refs-readable-by-vanilla-fetch` | 0          | 3/3     | **PASS** |
| `agent-ux/mirror-refs-cited-in-reject-hint`   | 0             | 4/4     | **PASS** |

Run-time evidence:

```text
quality/gates/agent-ux/mirror-refs-write-on-success.sh
  → cargo test -p reposix-remote --test mirror_refs write_on_success_updates_both_refs
  → 1 passed; 0 failed
  → "PASS: mirror-refs written on push success; both refs resolvable;
     tag message body well-formed; audit row present"

quality/gates/agent-ux/mirror-refs-readable-by-vanilla-fetch.sh
  → cargo test -p reposix-remote --test mirror_refs vanilla_fetch_brings_mirror_refs
  → 1 passed; 0 failed
  → "PASS: vanilla-mirror-clone brings refs/mirrors/* along to a fresh bare clone"

quality/gates/agent-ux/mirror-refs-cited-in-reject-hint.sh
  → cargo test -p reposix-remote --test mirror_refs reject_hint
  → 2 passed; 0 failed
  → "PASS: conflict-reject hint cites refs/mirrors/<sot>-synced-at with
     RFC3339 + (N minutes ago); first-push None case omits cleanly"
```

Catalog row `status: PASS` and `last_verified: 2026-05-01T08:58:34Z..35Z`
are consistent with the executor's claim.

---

## REQ-ID grading

| REQ-ID               | Status | Evidence                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
| -------------------- | ------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| DVCS-MIRROR-REFS-01  | PASS   | `crates/reposix-cache/src/mirror_refs.rs:74` (`format_mirror_head_ref_name`), `:80` (`format_mirror_synced_at_ref_name`), `:89` (`parse_synced_at_message`), `:110` (`Cache::write_mirror_head`), `:155` (`Cache::write_mirror_synced_at`), `:227` (`Cache::read_mirror_synced_at`). 5 unit tests under `mod tests` (lines 306–360) all PASS. Refs namespace `refs/mirrors/<sot>-{head,synced-at}` per Q2.1.                                          |
| DVCS-MIRROR-REFS-02  | PASS   | `crates/reposix-remote/src/main.rs:506-529` — handle_export success branch invokes `cache.refresh_for_mirror_head()` → `cache.write_mirror_head(...)` → `cache.write_mirror_synced_at(...)` → `cache.log_mirror_sync_written(...)`. Audit row UNCONDITIONAL per OP-3 (called after best-effort writes). Behaviorally proven by `vanilla_fetch_brings_mirror_refs` (mirror.git copies BOTH refs via plain `git clone --mirror`). Bus push / webhook coverage deferred to P81+. |
| DVCS-MIRROR-REFS-03  | PASS   | `crates/reposix-remote/src/main.rs:399-415` — conflict-reject branch reads `cache.read_mirror_synced_at(...)`; on Some, emits a `diag(...)` line with `refs/mirrors/{sot}-synced-at` + RFC3339 timestamp + `(N minutes ago)` rendering; on None, omits the hint cleanly. Behaviorally proven by `reject_hint_cites_synced_at_with_age` (asserts citation + RFC3339 regex + `\d+ minutes ago` regex) AND `reject_hint_first_push_omits_synced_at_line` (see Honesty spot-check below).               |

---

## Honesty spot-check — H3 (vacuous-test-risk) closure

PLAN-CHECK.md raised H3: the original "weaker form" first-push test
asserted `!stderr.contains("minutes ago")` on a SUCCESSFUL push, which
never enters the conflict-reject branch and therefore is vacuously true.

**Verification:** I read the actual integration test
`crates/reposix-remote/tests/mirror_refs.rs::reject_hint_first_push_omits_synced_at_line`
(lines 422–483) end-to-end:

1. **Setup is a real first-push conflict** (lines 433–445): the wiremock
   backend is mounted with issue 42 at `version=5` while the inbound
   stream's blob declares `prior_version=3`. No prior successful push
   ever runs against this cache_dir.
2. **Assertion 1 — non-vacuous proof of branch entry** (lines 466–469):

   ```rust
   assert!(
       stdout.contains("error refs/heads/main fetch first"),
       "expected conflict-reject status; stdout={stdout}; stderr={stderr}"
   );
   ```

   This is the conflict-reject status line emitted ONLY by the
   conflict-reject branch (`main.rs:422`). The test would FAIL if the
   helper accepted the push or rejected with a different status — it
   proves the test actually exercises the SC4 first-push None code path.
3. **Assertion 2 — synced-at line absent on first push** (lines 475–482):

   ```rust
   assert!(!stderr.contains("synced from"), ...);
   assert!(!stderr.contains("minutes ago"), ...);
   ```

   These assertions are non-trivially distinguishing because Assertion 1
   has already proven we're in the conflict-reject branch — where the
   `Some(ts)` arm WOULD emit those substrings. Their absence is direct
   evidence the `None` arm fired (no `refs/mirrors/sim-synced-at`
   present pre-push). Wiring at `main.rs:403` (`if let Ok(Some(synced_at))`)
   is the gating condition; the test would FAIL if the helper crashed
   on `None` or wrote a stub-rendered "synced from <None>" line.

**Honesty grade: H3 fix is substantive, NOT vacuous.** The PLAN-CHECK
"engineer a real first-push conflict by seeding the sim with a record
at higher version" recommendation was implemented faithfully via
wiremock instead of sim seeding (equivalent shape). H3 closed.

---

## CLAUDE.md update confirmation (QG-07)

`git diff 4a5dda0..d50533d -- CLAUDE.md` shows the in-phase update
required by QG-07 ("CLAUDE.md stays current"). Two distinct edits:

1. **New `## Architecture` paragraph** (CLAUDE.md line 29 of new file):
   Documents the namespace `refs/mirrors/<sot-host>-{head,synced-at}`,
   the `<sot-host>` slug enumeration (sim | github | confluence | jira),
   the cache's-bare-repo location (NOT working-tree `.git/`), the
   `git upload-pack --advertise-refs` propagation mechanism, the
   `(N minutes ago)` reject-hint rendering, **and the Q2.2 verbatim
   contract phrase**:

   > "**Important (Q2.2 doc-clarity contract):** `refs/mirrors/<sot>-synced-at`
   > is the timestamp the mirror last caught up to `<sot>` — it is NOT a
   > 'current SoT state' marker."

   Q2.2 phrase appears verbatim with the load-bearing "is NOT a 'current
   SoT state' marker" qualifier. Audit-row trail (`audit_events_cache`
   `op = 'mirror_sync_written'`) is documented as OP-3 unconditional.

2. **OP-3 update** (CLAUDE.md OP-3 line): the parenthetical "cache-internal
   events" list now includes `mirror-refs sync writes`. The new audit op
   is woven into the existing OP-3 prose, NOT appended as a narrative
   addendum (matches CLAUDE.md "Each phase introducing a new file/
   convention/gate updates CLAUDE.md … revising existing sections to
   reflect the new state — not appending a narrative.").

**QG-07 satisfied.** All three required items (namespace, Q2.2 verbatim,
OP-3 audit-row update) present and integrated in-flow.

---

## SURPRISES.md / SURPRISES-INTAKE.md review

The executor reported 4 deviations during P80 execution. I reviewed each
against OP-8's eager-resolution carve-out ("< 1 hour incremental work,
no new dependency introduced"):

| # | Deviation                                                                                                       | Phase-internal eager-resolution? | SURPRISES filing required? |
| - | --------------------------------------------------------------------------------------------------------------- | ------------------------------ | -------------------------- |
| 1 | audit-table CHECK constraint extension (added `mirror_sync_written` to allowed ops list)                        | YES (~5 lines, schema migration in same crate) | NO — eager-resolution OK   |
| 2 | `Cache::open` API mismatch (signature took different args than plan assumed)                                    | YES (one-line wiring fix)      | NO — eager-resolution OK   |
| 3 | `git clone --bare` vs `--mirror` (default `--bare` doesn't copy refs/mirrors/*; switched to `--mirror`)         | YES (one-line test change)     | NO — eager-resolution OK   |
| 4 | **Verifier-shell shape change**: shells rewired as `cargo test --test mirror_refs` thin wrappers instead of the original `reposix init`-based scenario | NO — this is a plan-shape pivot, not a syntactic fix | **YES — should be journaled** |

**Verification:** I read all three verifier shells
(`mirror-refs-{write-on-success,readable-by-vanilla-fetch,cited-in-reject-hint}.sh`)
end-to-end. They are 32–34 lines each and consist of:

```bash
cargo test -p reposix-remote --test mirror_refs <name> --quiet -- --nocapture
echo "PASS: ..."
exit 0
```

Each shell has a 10–15-line module-doc header that explicitly states
"delegates to the integration test … which drives `git-remote-reposix`
directly via stdin against a wiremock backend, [...] with a more
deterministic harness (no port races, no `git fetch` plumbing through
a working-tree clone)."

This is a real shape change from the plan's H1-fixed verifier-shell design
(which would have done `reposix init` + `git config remote.origin.url`
re-pointing + `git push origin main`). The pivot is reasonable — wiremock
+ stdin-driven helper is more deterministic and faster — but it
**bypasses the dark-factory contract layer** (the plan's verifier
shells were the only place that proved an end-to-end agent UX flow
through `reposix init`). The integration test
`vanilla_fetch_brings_mirror_refs` partially compensates by using `git
clone --mirror` against the cache's bare repo — but this is "vanilla
git can copy the refs", NOT "vanilla `git fetch` from a `reposix init`
working tree brings them along."

**Status of intake filing:** `quality/SURPRISES.md` was NOT updated for
P80 (last entry is from 2026-04-28 v0.12.1 P65 era). The v0.13.0
milestone has a `SURPRISES-INTAKE.md` ownership convention per OP-8 but
the file does not exist yet at
`.planning/milestones/v0.13.0-phases/SURPRISES-INTAKE.md` — only
`CARRY-FORWARD.md` and `GOOD-TO-HAVES.md` are present.

**This is an OP-8 honesty-check concern but NOT a GREEN-blocker.** The
shape change does not affect goal achievement (all 3 catalog rows PASS,
all 3 REQs have observable test coverage, vanilla `git clone --mirror`
explicitly proves the dark-factory contract via plain git). It SHOULD
be journaled so the next agent sees it; routing to advisory item below.

---

## Pre-push gate snapshot

```text
$ python3 quality/runners/run.py --cadence pre-push
summary: 26 PASS, 0 FAIL, 0 PARTIAL, 0 WAIVED, 0 NOT-VERIFIED -> exit=0
```

26/26 GREEN. Matches executor's T04 push-time claim. The terminal `git
push origin main` for d50533d landed on origin/main per `git log
origin/main -10` (4 P80 commits visible: 6711e59, a1c29e5, bf0fe95,
d50533d).

---

## Phase-close protocol

| Item                                                              | Status                                                                                                                                                              |
| ----------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| All 3 catalog rows graded PASS                                    | YES (re-verified with zero session context; verifier exit 0 for all 3)                                                                                              |
| All 4 P80 commits pushed (`origin/main..HEAD` empty modulo P80)   | YES — 6711e59, a1c29e5, bf0fe95, d50533d all on `origin/main` (verified via `git log origin/main -10`)                                                              |
| Pre-push gate locally GREEN                                       | YES — 26 PASS / 0 FAIL                                                                                                                                              |
| CLAUDE.md updated in-phase (QG-07)                                | YES — both Architecture paragraph + OP-3 update in commit `d50533d`; Q2.2 verbatim phrase present                                                                  |
| OP-3 audit row UNCONDITIONAL                                      | YES — `crates/reposix-remote/src/main.rs:529` (`cache.log_mirror_sync_written(...)`) called after best-effort write attempts; `audit_events_cache` schema extended in the same commit |
| Plan SUMMARY present                                              | **NO — 80-01-SUMMARY.md MISSING** (advisory; same shape as P79's missing 79-03-SUMMARY.md per advisory)                                                              |
| REQUIREMENTS.md DVCS-MIRROR-REFS-01..03 checkboxes flipped        | **NO — still `[ ]`** (advisory; standard close-out chore)                                                                                                            |
| STATE.md cursor updated                                           | **NO — still shows P79 SHIPPED, P80 next** (advisory)                                                                                                                |
| SURPRISES intake updated for shape change (deviation #4)          | NO — see Advisory 3 below                                                                                                                                             |

---

## Advisory items (do NOT block GREEN)

1. **80-01-SUMMARY.md missing.** No phase-close summary file exists
   under `.planning/phases/80-mirror-lag-refs/`. Recommend orchestrator
   authors a thin `80-01-SUMMARY.md` post-hoc citing d50533d as the
   close commit. Not a GREEN-blocker — close-out is reconstructable
   from PLAN + commit messages + this verdict.

2. **REQUIREMENTS.md + STATE.md not flipped.** DVCS-MIRROR-REFS-01..03
   checkboxes still `[ ]`; STATE.md still shows P79 as last shipped.
   Standard phase-close chore — should land in the same post-hoc
   commit as item 1. (P79 close had this same advisory.)

3. **Verifier-shell shape change not journaled (deviation #4).** The
   plan's H1-fixed `reposix init`-based verifier-shell design was
   replaced by `cargo test`-thin-wrapper shells driving
   wiremock+stdin integration tests. The change is reasonable
   (deterministic, no port-races) but bypasses the dark-factory
   contract surface that the planned verifier shells were the
   sole carrier of. Recommend: append a 1-line entry to
   `quality/SURPRISES.md`:

   > 2026-05-01 P80: Plan-fixed verifier-shell shape (reposix init →
   > working-tree clone → vanilla git push) replaced by cargo-test
   > thin-wrapper shells driving wiremock+stdin helper directly —
   > harness is more deterministic but the dark-factory end-to-end
   > flow (init → fetch → mirror-ref visible) is now covered by
   > `vanilla_fetch_brings_mirror_refs` via `git clone --mirror`
   > only, not via `reposix init` + `git fetch`. Tracked for P85
   > docs treatment + potential P88 good-to-have widening of the
   > dark-factory regression to cover refs/mirrors/* propagation
   > through `reposix init`.

   Not a GREEN-blocker — goal is achieved (all three catalog rows
   PASS with non-vacuous tests; vanilla git can read the refs).
   But the next agent should see the shape change in the journal.

---

## Summary

3/3 catalog rows PASS (artifact-checked, not status-claimed). 3/3
DVCS-MIRROR-REFS-* requirements have observable test coverage (5 unit
tests in `mirror_refs.rs` + 4 integration tests in `tests/mirror_refs.rs`,
all green). CLAUDE.md updated in-phase with namespace + Q2.2 verbatim
contract phrase + OP-3 audit-row extension. Pre-push gate 26/26 GREEN.
All 4 P80 commits (6711e59, a1c29e5, bf0fe95, d50533d) pushed to
`origin/main`. The PLAN-CHECK H3 vacuous-test risk is fully closed:
`reject_hint_first_push_omits_synced_at_line` first asserts the
conflict-reject branch IS entered (`stdout.contains("error
refs/heads/main fetch first")`) BEFORE asserting the synced-at hint is
absent — non-vacuous.

**Verdict: GREEN.** Phase 80 ships. Three advisory items above route to
a small follow-up commit (post-hoc 80-01-SUMMARY + REQUIREMENTS/STATE
flips + SURPRISES journal entry for deviation #4) but do not loop the
phase back.
