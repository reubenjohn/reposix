# P82 verdict — bus remote: URL parser, prechecks, fetch dispatch

**Verifier:** unbiased subagent · zero session context
**Verdict:** **GREEN** (with one advisory item — see § Advisory items)
**Date:** 2026-05-01
**Milestone:** v0.13.0 — DVCS over REST
**Precondition:** P81 verdict GREEN at `quality/reports/verdicts/p81/VERDICT.md` (confirmed)

---

## Catalog row grading

All 6 P82 catalog rows were re-graded by invoking the agent-ux dimension via
`python3 quality/runners/run.py --cadence pre-pr` from a fresh shell (zero
session context):

| Catalog row                                                  | Verifier exit | Asserts | Status   |
| ------------------------------------------------------------ | ------------- | ------- | -------- |
| `agent-ux/bus-url-parses-query-param-form`                   | 0 (0.30s)     | 3/3     | **PASS** |
| `agent-ux/bus-url-rejects-plus-delimited`                    | 0 (0.31s)     | 3/3     | **PASS** |
| `agent-ux/bus-precheck-a-mirror-drift-emits-fetch-first`     | 0 (0.64s)     | 4/4     | **PASS** |
| `agent-ux/bus-precheck-b-sot-drift-emits-fetch-first`        | 0 (0.83s)     | 3/3     | **PASS** |
| `agent-ux/bus-fetch-not-advertised`                          | 0 (0.35s)     | 3/3     | **PASS** |
| `agent-ux/bus-no-remote-configured-error`                    | 0 (0.37s)     | 3/3     | **PASS** |

Pre-pr cadence summary: `14 PASS, 0 FAIL, 0 PARTIAL, 3 WAIVED, 0 NOT-VERIFIED -> exit=0`.
Catalog `status: PASS` and `last_verified` timestamps for the 6 P82 rows
(`2026-05-01T13:26:31..36Z`) are consistent with the executor's claim.

---

## REQ-ID grading (DVCS-BUS-URL-01, DVCS-BUS-PRECHECK-01, DVCS-BUS-PRECHECK-02, DVCS-BUS-FETCH-01)

| REQ-ID                | Status | Evidence                                                                                                                                                                                                                                                                                                                                |
| --------------------- | ------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| DVCS-BUS-URL-01       | PASS   | `crates/reposix-remote/src/bus_url.rs:42-58` ships `Route { Single(ParsedRemote) \| Bus { sot, mirror_url } }`; `:68-154` ships `parse(url)` which strips `?<query>`, rejects `+` (`:83-88`), rejects unknown query keys (`:127-132`), rejects empty query (`:101-106`), rejects empty `mirror=` (`:143-148`). Behaviorally proven by `tests/bus_url.rs::parses_query_param_form_round_trip` (positive cap-ad), `rejects_plus_delimited_bus_url` (line 50-66), `rejects_unknown_query_param` (line 68-85), and the inline unit tests `route_single_for_bare_reposix_url` + `parses_query_param_form_round_trip` (`bus_url.rs:178-187, 162-173`). |
| DVCS-BUS-PRECHECK-01  | PASS   | `crates/reposix-remote/src/bus_handler.rs:245-292` ships `precheck_mirror_drift` shelling out `git ls-remote -- <mirror_url> refs/heads/main` (`:248`, `--` separator unconditional) and comparing to `git rev-parse refs/remotes/<name>/main` (`:269-272`). T-82-01 mitigation: `-`-prefix reject at `bus_handler.rs:82-89` BEFORE shell-out. Behaviorally proven by `tests/bus_precheck_a.rs::bus_precheck_a_emits_fetch_first_on_drift` (line 95-125, real `git init --bare` fixture, asserts `error refs/heads/main fetch first` on stdout AND `your GH mirror has new commits` on stderr) + `bus_precheck_a_passes_when_mirror_in_sync` + `rejects_dash_prefixed_mirror_url` (T-82-01). |
| DVCS-BUS-PRECHECK-02  | PASS   | `crates/reposix-remote/src/precheck.rs:352-377` ships `precheck_sot_drift_any` (NEW 26-line wrapper; non-empty `list_changed_since` → `Drifted`, empty / no-cursor → `Stable`); `precheck_export_against_changed_set` body unchanged (only doc-comment additions per `git diff b73ab67..HEAD`). `bus_handler.rs:130-170` wires the wrapper, emits `error refs/heads/main fetch first` on `Drifted`. Behaviorally proven by `tests/bus_precheck_b.rs::bus_precheck_b_emits_fetch_first_on_sot_drift` (line 99-200, real wiremock SoT, synced file:// mirror so PRECHECK A passes, drifted SoT emits fetch-first) + `bus_precheck_b_passes_when_sot_stable` (line 202-287, `expect(0)` on PATCH path, asserts deferred-shipped error reached). |
| DVCS-BUS-FETCH-01     | PASS   | `crates/reposix-remote/src/main.rs:192-200` ships capability branching: `import` / `export` / `refspec` / `object-format=sha1` always emitted; `stateless-connect` gated on `state.mirror_url.is_none()` (line 195-197). Behaviorally proven by `tests/bus_capabilities.rs::bus_url_omits_stateless_connect` (asserts `!stdout.contains("stateless-connect")` for bus URL) AND `single_backend_url_advertises_stateless_connect` (asserts bare URL DOES advertise — regression check). |

---

## Honesty spot-check (5 items)

### 1. `tests/bus_url.rs::parses_query_param_form_round_trip` uses positive capability-advertise assertion

**Verified.** `tests/bus_url.rs:43-46`:

```rust
assert!(
    stdout.contains("import") && stdout.contains("export"),
    "expected helper to advertise capabilities; stdout={stdout} stderr={stderr}"
);
```

This is a POSITIVE assertion on stdout containing both `import` and `export`
(the capabilities arm's first two lines), NOT a negative
`!stderr.contains("parse remote url")` (the H1 false-GREEN form). Test driver
sends `capabilities\n\n` on stdin (`:38`), and the test comment block
(`:14-31`) explicitly cites the H1 fix from plan-check. **H1 closed.**

### 2. `bus_handler::handle_bus_export` emits the D-02 deferred-shipped stderr after both prechecks pass

**Verified.** `bus_handler.rs:300-310`:

```rust
fn emit_deferred_shipped_error<R: std::io::Read, W: std::io::Write>(
    proto: &mut Protocol<R, W>,
    state: &mut State,
) -> Result<()> {
    crate::diag("bus write fan-out (DVCS-BUS-WRITE-01..06) is not yet shipped — lands in P83");
    proto.send_line("error refs/heads/main bus-write-not-yet-shipped")?;
    ...
```

Called at line 173 (`emit_deferred_shipped_error(proto, state)`) AFTER both
PRECHECK A (line 104) and PRECHECK B (line 137) succeed. The stderr substring
"bus write fan-out (DVCS-BUS-WRITE-01..06) is not yet shipped — lands in P83"
is asserted by `tests/bus_precheck_b.rs::bus_precheck_b_passes_when_sot_stable`
at line 270-273:

```rust
assert!(
    stderr.contains("bus write fan-out") && stderr.contains("P83"),
    "expected D-02 stderr diagnostic; got: {stderr}"
);
```

**D-02 closed.**

### 3. `bus_url::parse` rejects unknown query keys with verbatim "unknown query parameter `<key>`" hint

**Verified.** `bus_url.rs:127-132`:

```rust
other => {
    return Err(anyhow!(
        "unknown query parameter `{other}` in bus URL; \
         only `mirror=` is supported"
    ));
}
```

Matched by `tests/bus_url.rs::rejects_unknown_query_param` (line 80-83):

```rust
assert!(
    stderr.contains("unknown query parameter") && stderr.contains("priority"),
    "expected stderr to name the unknown key; got: {stderr}"
);
```

**Q-C / D-03 closed.** Forward-compat-via-explicit-opt-in satisfied (future
`priority=` / `retry=` params will need explicit code support, not silent
backward-compat rotting).

### 4. Capabilities branching at `main.rs:192-200` is a small surgical edit

**Verified.** Diff hunk shows the change is a 3-line conditional addition:

```rust
proto.send_line("import")?;
proto.send_line("export")?;
proto.send_line("refspec refs/heads/*:refs/reposix/*")?;
-proto.send_line("stateless-connect")?;
+if state.mirror_url.is_none() {
+    proto.send_line("stateless-connect")?;
+}
proto.send_line("object-format=sha1")?;
```

Plan called for "5-line edit" per D-06; actual is 3 lines + 6 lines of
explanatory comment. Within budget. **D-06 closed.**

### 5. `parse_remote_url` (single-backend path) stayed unchanged

**Verified.** `git diff b73ab67..HEAD -- crates/reposix-remote/src/backend_dispatch.rs`
returns NO output. `parse_remote_url` is `pub(crate) fn parse_remote_url(url:
&str) -> Result<ParsedRemote>` at `backend_dispatch.rs:102` — same shape, same
visibility, same body. `bus_url::parse` is a SIBLING that delegates the base
form (after `?`-strip) to `parse_remote_url` rather than replacing it
(`bus_url.rs:93-94`). Single-backend regression intact.

---

## D-01..D-06 ratification cross-check

| Decision | Description | Evidence |
| -------- | ----------- | -------- |
| **D-01** (Q-A) | STEP 0 resolves local mirror remote name via `git config --get-regexp ^remote\..+\.url$`; byte-equal-match values to `mirror_url` (trailing-slash normalized); pick first alphabetical + WARN if multiple; zero matches → Q3.5 hint, exit before PRECHECK A. | `bus_handler.rs:180-238` (`resolve_mirror_remote_name`); WARN at `:231-233`; Q3.5 hint at `:95-100`; tested by `tests/bus_precheck_a.rs::bus_no_remote_configured_emits_q35_hint`. |
| **D-02** (Q-B) | After both prechecks pass, P82 emits "bus write fan-out (DVCS-BUS-WRITE-01..06) is not yet shipped — lands in P83" on stderr + `error refs/heads/main bus-write-not-yet-shipped` protocol error. P83 replaces this stub. | `bus_handler.rs:300-310` (`emit_deferred_shipped_error`); tested by `bus_precheck_b_passes_when_sot_stable:262-273`. |
| **D-03** (Q-C) | Unknown query keys (anything other than `mirror=`) rejected with verbatim "unknown query parameter `<key>`" hint; forward-compat-via-explicit-opt-in. | `bus_url.rs:127-132`; tested by `bus_url::tests::rejects_unknown_query_param` (unit) + `tests/bus_url.rs::rejects_unknown_query_param` (integration). |
| **D-04** | (CLAUDE.md Build memory budget — sequential per-crate cargo) | Per-crate cargo invocations in 6 verifier shells: `cargo test -p reposix-remote --test <name>`. Verifier shells sequential within `quality/runners/run.py`. |
| **D-05** | (P81 ratification — applied to `read_blob_cached` + cursor; carried forward to P82's `precheck_sot_drift_any` calling `cache.and_then(|c| c.read_last_fetched_at().ok().flatten())` instead of `read_blob`/`read_records`.) | `precheck.rs:359` uses `read_last_fetched_at` cursor (sync, gix-local); no `read_blob` / `read_records` call. |
| **D-06** | Capabilities branching is a small surgical edit (3-line conditional, not a rewrite of the capabilities arm). T-82-01 `--` separator + `-`-prefix reject unconditional. | Capabilities: `main.rs:192-200` (3-line `if` branch). T-82-01: `bus_handler.rs:82-89` (reject `-`-prefix) + `:248` (`["ls-remote", "--", mirror_url, "refs/heads/main"]` — unconditional `--`). Tested by `tests/bus_precheck_a.rs::rejects_dash_prefixed_mirror_url`. |

All six ratifications observable in code/tests/docs. No watered-down workarounds.

---

## CLAUDE.md update confirmation (QG-07)

`git diff b73ab67..HEAD -- CLAUDE.md` shows the in-phase update required by
QG-07 (commit `d38e6e1`). Two distinct edits:

1. **§ Architecture — new "Bus URL form (P82+)" paragraph** (CLAUDE.md line 33).
   Verbatim core sentences:

   > "**Bus URL form (P82+).** `reposix::<sot-spec>?mirror=<mirror-url>` per
   > Q3.3 — the SoT side dispatches via the existing `BackendConnector`
   > pipeline (sim / confluence / github / jira); the mirror is a plain-git
   > URL consumed as a shell-out argument to `git ls-remote` / `git push`.
   > Bus is PUSH-only (Q3.4) — fetch on a bus URL falls through to the
   > single-backend code path, so the helper does NOT advertise
   > `stateless-connect` for bus URLs. The `+`-delimited form is dropped;
   > unknown query keys (anything other than `mirror=`) are rejected. Mirror
   > URLs containing `?` must be percent-encoded (the first unescaped `?`
   > in the bus URL is the bus query-string boundary). On push, the bus
   > handler runs two cheap prechecks BEFORE reading stdin: PRECHECK A
   > (`git ls-remote -- <mirror>` versus local `refs/remotes/<name>/main`)
   > and PRECHECK B (`list_changed_since` against the SoT cursor); both
   > reject with `error refs/heads/main fetch first` on drift. P82 ships
   > the URL-recognition + dispatch + precheck surface; the SoT-first
   > write fan-out lands in P83. See
   > `.planning/research/v0.13.0-dvcs/architecture-sketch.md § 3` and
   > `decisions.md § Q3.3-Q3.6` for the algorithm + open-question
   > resolutions."

   All required QG-07 elements present:
   - Bus URL form `?mirror=` (Q3.3) — YES
   - PUSH-only / no `stateless-connect` (Q3.4) — YES
   - `+` form dropped — YES
   - Unknown query keys rejected (Q-C / D-03) — YES
   - Percent-encoding boundary — YES
   - Both prechecks (A + B) — YES
   - P83 deferral — YES

2. **§ Commands — new bus push bullet** (CLAUDE.md line 183-184):

   ```
   # Bus push (v0.13.0+ P82): URL form `reposix::<sot>?mirror=<mirror-url>` recognized + dispatched; cheap prechecks (mirror drift + SoT drift) gate the push BEFORE reading stdin. Write fan-out (DVCS-BUS-WRITE-01..06) lands in P83.
   git push reposix main                                     # bus push (URL: reposix::<sot>?mirror=<url>; SoT-first writes land in P83)
   ```

**QG-07 satisfied.** Both required updates present and integrated into
existing sections (NOT appended as a narrative addendum, matching CLAUDE.md
"revising existing sections to reflect the new state").

---

## SURPRISES-INTAKE review

`grep -nE "Discovered-by.*P82|## P82" .planning/milestones/v0.13.0-phases/SURPRISES-INTAKE.md` → ZERO matches.
`grep -n "P82" SURPRISES-INTAKE.md` → 1 hit (in P80's entry, referencing P82
as the territory where helper first-push ref-advertisement work belongs).

**Executor's claim:** zero P82 entries appended; all 7 deviations were
eager-resolved per OP-8 carve-out.

**Sanity check on the 7 deviations:**

| # | Deviation | < 1hr? | New dependency? | OP-8 fit |
| - | --------- | ------ | --------------- | -------- |
| 1 | Verifier-shell test-name fix (typo in `bus-url-rejects-plus-delimited.sh` — corrected in d38e6e1) | YES | NO | OK |
| 2 | Working-tree fixture for bus_precheck_a (tempfile + `git init --bare`) | YES (mirrors P80 dark-factory idiom) | NO | OK |
| 3 | Branch checkout (`git checkout -b main`) added to fixture builder for git-default-branch hygiene | YES | NO | OK |
| 4 | `clippy --all-targets` ran clean post-T05 (no separate fix commit needed beyond 682a8dc clippy-match-wildcard fix) | YES | NO | OK |
| 5 | `scripts/p82-validate-catalog-rows.py` promoted from ad-hoc bash per CLAUDE.md §4 | YES (59 lines) | NO | OK |
| 6 | `dead_code` allows on `last_fetch_want_count` field — preserved verbatim, no expansion | YES | NO | OK |
| 7 | Visibility widening (`State.backend_name`, `State.mirror_url`, `diag`, `ensure_cache` to `pub(crate)`) | YES | NO | OK — purely additive, M2 fix |

All 7 deviations meet OP-8 criteria (< 1hr incremental, no new dep). Empty
intake is consistent with executor honestly looking AND eagerly resolving.
The OP-8 carve-out is the correct response. No "found-it-but-skipped-it"
pattern observable.

---

## Plan-checker H1+M1+M2+M3 honesty spot-check

PLAN-CHECK.md graded YELLOW with 1 HIGH + 3 MEDIUM issues
(`82-bus-remote-url-parser/PLAN-CHECK.md:8-50`). All four fixes substantively
present in code:

### H1 — `tests/bus_url.rs::parses_query_param_form_round_trip` false-GREEN
**Required fix:** positive `stdout.contains("import") && stdout.contains("export")`
assertion instead of negative `!stderr.contains("parse remote url")`.
**Verification:** `tests/bus_url.rs:43-46` is positive assertion form (Item 1
of honesty spot-check above). Test driver sends `capabilities\n\n` on stdin
(`:38`). Comment block (`:14-31`) explicitly cites the H1 fix.
**H1 closed: substantive fix.**

### M1 — `tests/common.rs` copy is a buried conditional
**Required fix:** lift to atomic step T05 step 5a-prime; unconditional copy
+ atomic commit `test(remote): copy tests/common.rs from reposix-cache (P81 M3 gap)`.
**Verification:** Commit `455cd4a` "test(remote): copy tests/common.rs from
reposix-cache (P81 M3 gap)" lands BEFORE commit `74f915b` (the 4 integration
tests including `tests/bus_precheck_b.rs` which imports `mod common`).
`crates/reposix-remote/tests/common.rs` exists, 127 lines, copied from
`crates/reposix-cache/tests/common/mod.rs`. `bus_precheck_b.rs:24-25` uses
`mod common; use common::{sample_issues, seed_mock, sim_backend, CacheDirGuard};`.
**M1 closed: substantive fix.**

### M2 — `diag`/`ensure_cache` widening missing from `<must_haves>`
**Required fix:** add `diag` (L80) and `ensure_cache` (L219) to T04
`<must_haves>` "main.rs dispatch wiring" bullet.
**Verification:** `main.rs:90` declares `pub(crate) fn diag(msg: &str)`;
`main.rs:256` declares `pub(crate) fn ensure_cache(state: &mut State)`.
The widening lands in commit `a721b56` "feat(remote): bus_handler + main.rs
Route dispatch + capabilities branching + State extension". `bus_handler.rs`
calls both: `crate::diag(...)` (line 112, 117, 147, 156, 163, 304, 323) and
`crate::ensure_cache(state)` (line 136). `fail_push` stays private — bus
handler defines local `bus_fail_push` (`bus_handler.rs:317-329`).
**M2 closed: substantive fix.**

### M3 — `bus_precheck_b.rs` placeholder ellipses
**Required fix:** spell out body verbatim referencing perf_l1.rs matchers
(~150L addition); OR split to its own task.
**Verification:** `tests/bus_precheck_b.rs` is 287 lines (NOT placeholder).
Both test functions (`bus_precheck_b_emits_fetch_first_on_sot_drift` at
`:99-200` + `bus_precheck_b_passes_when_sot_stable` at `:202-287`) are
fully written with concrete wiremock fixtures + synced mirror fixture. The
`HasSinceQueryParam` matcher is verbatim from `tests/perf_l1.rs:32-49`
(per the file-level doc comment at `:27-30`). The synced-mirror fixture
(`make_synced_mirror_fixture` at `:60-96`) is a verbatim donor from
`bus_precheck_a.rs::make_drifting_mirror_fixture` minus the divergent commit.
**M3 closed: substantive fix.**

**All 4 PLAN-CHECK YELLOW fixes are present in code. No watered-down workarounds.**

---

## Pre-push gate snapshot

```text
$ python3 quality/runners/run.py --cadence pre-push
summary: 26 PASS, 0 FAIL, 0 PARTIAL, 0 WAIVED, 0 NOT-VERIFIED -> exit=0
```

```text
$ python3 quality/runners/run.py --cadence pre-pr
summary: 14 PASS, 0 FAIL, 0 PARTIAL, 3 WAIVED, 0 NOT-VERIFIED -> exit=0
```

26/26 GREEN on pre-push (the 6 P82 rows are pre-pr cadence; pre-pr 14/14 GREEN
ex-3-WAIVED). All 8 P82 commits visible on `origin/main` per `git log
origin/main -10`:

```
682a8dc fix(remote): clippy match-wildcard-for-single-variants + doc-markdown SoT/Route in test panics (P82-01 pre-push fix)
d38e6e1 quality(agent-ux): flip 6 P82 rows FAIL→PASS + CLAUDE.md update + verifier-shell test-name fix (DVCS-BUS-URL-01..02-PRECHECK + DVCS-BUS-FETCH-01 close)
74f915b test(remote): 4 integration tests — bus_url + bus_capabilities + bus_precheck_a + bus_precheck_b (DVCS-BUS-URL-01..02-PRECHECK + DVCS-BUS-FETCH-01)
455cd4a test(remote): copy tests/common.rs from reposix-cache (P81 M3 gap)
a721b56 feat(remote): bus_handler + main.rs Route dispatch + capabilities branching + State extension (DVCS-BUS-PRECHECK-01..02 + DVCS-BUS-FETCH-01)
b88dad0 feat(remote): coarser SoT-drift wrapper precheck_sot_drift_any (DVCS-BUS-PRECHECK-02 substrate)
c754eac feat(remote): bus URL parser — bus_url::parse + Route::Single|Bus enum (DVCS-BUS-URL-01)
9818f16 quality(agent-ux): mint 6 bus-remote catalog rows + 6 TINY verifiers (DVCS-BUS-URL-01..02-PRECHECK + DVCS-BUS-FETCH-01 catalog-first)
```

---

## Phase-close protocol

| Item                                                              | Status                                                                                                                                                              |
| ----------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| All 6 catalog rows graded PASS                                    | YES (re-verified with zero session context; verifier exit 0 on all 6 via pre-pr cadence runner)                                                                     |
| All 8 P82 commits pushed                                          | YES — `9818f16`, `c754eac`, `b88dad0`, `a721b56`, `455cd4a`, `74f915b`, `d38e6e1`, `682a8dc` all on `origin/main`                                                   |
| Pre-push gate locally GREEN                                       | YES — 26 PASS / 0 FAIL                                                                                                                                              |
| CLAUDE.md updated in-phase (QG-07)                                | YES — both Architecture paragraph (Bus URL form P82+) + § Commands bullet in commit `d38e6e1`; all D-03 / Q3.3-3.5 elements present                                 |
| Catalog-first contract honored                                    | YES — T01 commit `9818f16` mints catalog rows + verifier shells with `status: FAIL`; T06 commit `d38e6e1` flips them PASS only after the implementation lands       |
| Push BEFORE verifier dispatch                                     | YES — pre-push gate ran on each of the 8 commits (per CLAUDE.md per-phase push cadence; `682a8dc` is the post-fix-forward commit closing the clippy + doc-markdown drift)|
| H1+M1+M2+M3 PLAN-CHECK YELLOW fixes present in code               | YES — all 4 fixes verified file:line above                                                                                                                          |
| `parse_remote_url` (single-backend) unchanged                     | YES — `git diff b73ab67..HEAD -- backend_dispatch.rs` returns no output                                                                                             |
| `precheck_export_against_changed_set` body unchanged              | YES — only doc-comment additions (lines 22-32); body at `:90-313` is unchanged                                                                                      |
| T-82-01 mitigation                                                | YES — `bus_handler.rs:82-89` rejects `-`-prefix mirror URL BEFORE shell-out; `:248` shells `git ls-remote --<separator> <url> refs/heads/main`; tested by `rejects_dash_prefixed_mirror_url` |
| Bus URL omits `stateless-connect`; bare URL retains it            | YES — `tests/bus_capabilities.rs::bus_url_omits_stateless_connect` AND `single_backend_url_advertises_stateless_connect` both PASS                                  |
| `precheck_sot_drift_any` is a NEW wrapper (not a refactor)        | YES — `git diff b73ab67..HEAD -- precheck.rs` shows additive 107L; `precheck_export_against_changed_set` body unchanged                                             |
| SURPRISES-INTAKE updated for P82 deviations                       | EXPLICITLY EMPTY — executor's claim of "no entries; all 7 eager-resolved" honestly verified (zero P82-attributed entries; OP-8 honesty check satisfied)            |
| Ad-hoc bash promotion (CLAUDE.md §4)                              | YES — `scripts/p82-validate-catalog-rows.py` (59L, committed 9818f16)                                                                                               |
| Plan SUMMARY present                                              | **NO — 82-01-SUMMARY.md MISSING** (advisory; same shape as P79/P80/P81's missing SUMMARYs)                                                                          |

---

## Advisory items (do NOT block GREEN)

1. **82-01-SUMMARY.md missing.** No phase-close summary file exists under
   `.planning/phases/82-bus-remote-url-parser/`. Recommend orchestrator
   authors a thin `82-01-SUMMARY.md` post-hoc citing `682a8dc` (the
   pre-push fix-forward commit) as the close commit. Not a GREEN-blocker
   — close-out is reconstructable from PLAN + commit messages + this
   verdict. Same advisory as P79/P80/P81; pattern is now an established
   v0.13.0 close-out chore that the orchestrator handles after
   verifier-grading.

2. **REQUIREMENTS.md DVCS-BUS-URL-01 / -PRECHECK-01 / -PRECHECK-02 /
   -FETCH-01 checkboxes.** Still `[ ]` per `.planning/REQUIREMENTS.md:71-74`.
   Standard phase-close chore — should land in the same post-hoc commit
   as item 1. (P79/P80/P81 close had this same advisory.)

3. **STATE.md cursor.** Not freshly inspected here; same close-out chore
   pattern. Should land in post-hoc commit.

---

## Summary

6/6 catalog rows PASS (artifact-checked, not status-claimed). 4/4 DVCS-BUS-*
requirements have observable test coverage (`bus_url.rs` parse + reject paths
+ `bus_handler.rs` STEP 0 / PRECHECK A / PRECHECK B / D-02 deferred-shipped
emit + `tests/bus_url.rs` integration assertions + `tests/bus_capabilities.rs`
positive+negative cap-ad assertions + `tests/bus_precheck_a.rs` real
`git init --bare` fixture + drifted/synced/T-82-01-injection cases +
`tests/bus_precheck_b.rs` real wiremock + synced file:// mirror fixture +
drifted/stable cases with `expect(0)` PATCH backstop). CLAUDE.md updated
in-phase with the Bus URL form (P82+) Architecture paragraph (Q3.3 form,
PUSH-only Q3.4, `+` reject, unknown-key reject Q-C, percent-encoding,
both prechecks, P83 deferral) AND the § Commands bus push bullet. Pre-push
gate 26/26 GREEN; pre-pr 14 PASS / 0 FAIL / 3 WAIVED. All 8 P82 commits
(`9818f16`, `c754eac`, `b88dad0`, `a721b56`, `455cd4a`, `74f915b`, `d38e6e1`,
`682a8dc`) pushed to `origin/main`. All 4 PLAN-CHECK YELLOW issues
(H1+M1+M2+M3) are substantively fixed in code: positive cap-ad assertion
landed; `tests/common.rs` lifted to atomic commit `455cd4a`;
`diag`/`ensure_cache` widened to `pub(crate)`; `bus_precheck_b.rs` body
spelled out verbatim (NOT placeholder ellipses). T-82-01 argument-injection
mitigation present and tested (`-`-prefix reject + `--` separator unconditional).
`parse_remote_url` (single-backend) unchanged; `precheck_export_against_changed_set`
body unchanged. SURPRISES-INTAKE empty per executor's claim — OP-8 honesty
check passes (7 eager-resolved deviations all meet < 1hr / no-new-dep
criteria). The bus URL form (`reposix::<sot>?mirror=<url>`) recognizes,
dispatches, runs both cheap prechecks before stdin read, emits the D-02
deferred-shipped error after prechecks pass, and omits `stateless-connect`
for bus URLs while preserving it for bare `reposix::<sot>` URLs.

**Verdict: GREEN.** Phase 82 ships. Three advisory items above route to a
small follow-up commit (post-hoc 82-01-SUMMARY + REQUIREMENTS/STATE flips)
but do not loop the phase back.
