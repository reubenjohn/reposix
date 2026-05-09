# Phase P82 Audit — bus-remote-url-parser

**Auditor:** unbiased subagent (zero session context)
**Date:** 2026-05-08
**Phase:** P82 — Bus remote: URL parser, prechecks, fetch dispatch
**Phase verdict on file:** GREEN (advisory items 1-3 noted)
**Source-of-truth artifacts read:**
- `.planning/milestones/v0.13.0-phases/ROADMAP.md` § Phase 82
- `.planning/phases/82-bus-remote-url-parser/82-PLAN-OVERVIEW/{index,decisions,architecture,constraints-and-threats,phase-management}.md`
- `.planning/phases/82-bus-remote-url-parser/82-01-PLAN/{T01-T06,must-haves,threat-model,index}.md`
- `.planning/phases/82-bus-remote-url-parser/PLAN-CHECK.md` (referenced via verdict)
- `quality/reports/verdicts/p82/VERDICT.md`
- `quality/catalogs/agent-ux.json` (six P82 rows)
- `crates/reposix-remote/src/{main.rs,bus_url.rs,bus_handler.rs}`
- `crates/reposix-remote/tests/{bus_url.rs,bus_capabilities.rs,bus_precheck_a.rs,bus_precheck_b.rs}`
- `.planning/research/v0.13.0-real-backend-frictions/01-dark-factory-may02/{SUMMARY.md,T3-bus-push.md}`

## Verdict at a glance

- ALIGNED items: 11
- MISALIGNED items: 8
- SUSPECT items: 1

The P82 verdict is *internally* consistent — six catalog rows match the
shipped code at the unit/integration layer. The misalignments come from
(a) load-bearing claims that were verified only structurally (capability
omission, parser shape) instead of functionally (an actual `git fetch`
working over a bus URL); (b) plan promises that were narrated in CLAUDE.md
and PLAN files but never landed as tests (percent-encoded mirror URLs);
and (c) catalog framing that exempts the entire transport layer of P82
from real-backend coverage (no `--ignored` real-Confluence or real-GH
mirror test exists). These match the SUMMARY.md "structural exemption"
root cause of the v0.13.1 audit.

## Findings

### F1 — DVCS-BUS-FETCH-01 verifies absence of capability, not "fetch falls through to single-backend path" [SEVERITY: HIGH]

**Claim in plan/CLAUDE.md/architecture-sketch:** Q3.4 RATIFIED says "fetch
goes to the SoT directly via existing single-backend code path" and
CLAUDE.md line 26 reads "fetch on a bus URL falls through to the
single-backend path." The catalog row description for
`agent-ux/bus-fetch-not-advertised` says the invariant is "capability
list ... omits stateless-connect ... so fetch falls through to the
single-backend code path."

**Reality:** The implementation simply *omits* `stateless-connect` from
the capabilities list when `state.mirror_url.is_some()`
(`crates/reposix-remote/src/main.rs:196-198`). When git's transport
helper sees a remote without `stateless-connect`, it falls back to the
deprecated `import` capability (also still advertised at
`main.rs:193`) — which is the v0.8 path tagged for removal in Phase 36
(see `main.rs:176-177`: "the v0.8 `import` capability (deprecated — one
release cycle; Phase 36 removes)"). So fetch over a bus URL exercises a
deprecated path, not the v0.9 `stateless-connect` path the
"single-backend code path" phrase implies. The integration tests
(`crates/reposix-remote/tests/bus_capabilities.rs:8-41`) only assert
`!stdout.contains("stateless-connect")` for the bus URL — they do NOT
exercise an actual `git fetch` or `git ls-remote --heads` round-trip
against a bus URL. There is zero behavioral test that fetch over a bus
URL succeeds.

**Evidence:**
- `crates/reposix-remote/src/main.rs:193-198` (capabilities arm; only
  `stateless-connect` is gated, `import` always sent)
- `crates/reposix-remote/tests/bus_capabilities.rs:38-40` (negative
  assertion only)
- `quality/catalogs/agent-ux.json` row `agent-ux/bus-fetch-not-advertised`
  asserts list-shape only; cf. AUDIT-BRIEF.md failure shape #1
  ("URL-shape only / exists-only / structure-only assertions where the
  description implies functional verification")

**Why it matters:** This is failure shape #1 from the AUDIT-BRIEF
verbatim. Q3.4 promises a behavior; the test certifies a list. If
Phase 36 removes `import` (the existing CLAUDE.md narrative says it
will), bus-URL fetch silently breaks with no test catching it. A real
v0.13.1 user typing `git fetch reposix::sim::demo?mirror=...` has never
been tested.

---

### F2 — Plan-promised "percent-encoded form parses correctly" test does not exist [SEVERITY: MED]

**Claim in plan:** `82-PLAN-OVERVIEW/phase-management.md:41` (risk row):
> "T02 has a test case asserting the percent-encoded form parses
> correctly. The non-encoded form errors with a clear message per Q-C
> (extra `?` introduces an unknown key)."

CLAUDE.md line 26 / verdict QG-07 confirmation echo the same:
> "`?` in mirror URLs must be percent-encoded."

**Reality:** No test (unit or integration) covers a mirror URL containing
`?` — encoded or unencoded. `crates/reposix-remote/src/bus_url.rs` has
zero `percent`/`%3F`/`encoded` references outside the doc-comment at
lines 22-28; `crates/reposix-remote/tests/bus_url.rs` likewise has none.
The parser at `bus_url.rs:79` does `stripped.split_once('?')` (FIRST
unescaped `?`), then queries on `&`. A literal embedded `?` in the
mirror value (e.g. `?mirror=https://gh.example?token=foo`) silently gets
captured into the value as a side effect of `split_once` taking only
the first `?` — *not* validated; an embedded `&` would split the value
and produce an "unknown query parameter" error rather than a
"mirror URL contains unencoded `?`" diagnostic.

**Evidence:**
- `crates/reposix-remote/src/bus_url.rs:79-82, 108-118, 156-224`
  (parser body + 4 unit tests; none cover percent-encoding)
- `crates/reposix-remote/tests/bus_url.rs:1-86` (3 integration tests;
  none cover percent-encoding)
- `.planning/phases/82-bus-remote-url-parser/82-PLAN-OVERVIEW/phase-management.md:41`
  (the risk-row promise)

**Why it matters:** This is failure shape #3 (plan promises N items,
ship delivers N-1 silently). The doc says "the user MUST percent-encode"
but the parser neither requires nor validates encoding. A mirror URL
with an authenticated query string (`https://gh.example?token=foo`)
will silently produce surprising parses, and none of the failure modes
are guarded against regression.

---

### F3 — DVCS-BUS-PRECHECK-01 / -02 / -FETCH-01 have ZERO real-backend coverage [SEVERITY: HIGH]

**Claim in plan / catalog row sources:** All four DVCS-BUS-* requirements
list `architecture-sketch.md` and `decisions.md` as authoritative
sources for transport-layer claims. The verdict's "Pre-push gate
snapshot" certifies 26 PASS / 0 FAIL on pre-push and 14 PASS on pre-pr.

**Reality:** All P82 tests run against (a) the in-process simulator
(`http://127.0.0.1:9` is closed; `instantiate_sim` is no-network),
(b) wiremock, or (c) a `tempfile::tempdir() + git init --bare` file://
mirror. `crates/reposix-cli/tests/agent_flow_real.rs` has zero
references to `bus`, `mirror=`, or any P82-introduced surface; a
`grep -c "bus\|mirror=" agent_flow_real.rs` returns 0. There is no
`#[ignore]` real-Confluence test, no real-GitHub-mirror precheck test,
no test of any kind against the sanctioned testing targets in
`docs/reference/testing-targets.md`.

**Evidence:**
- `crates/reposix-cli/tests/agent_flow_real.rs` — no bus URL test
- `quality/catalogs/agent-ux.json` row `agent-ux/bus-precheck-a-mirror-drift-emits-fetch-first`
  uses `tests/bus_precheck_a.rs::bus_precheck_a_emits_fetch_first_on_drift`
  (file:// mirror) as evidence
- `quality/catalogs/agent-ux.json` row `agent-ux/bus-precheck-b-sot-drift-emits-fetch-first`
  uses `tests/bus_precheck_b.rs::bus_precheck_b_emits_fetch_first_on_sot_drift`
  (wiremock SoT)
- T3-bus-push.md F9-F15 (the dark-factory exercise) demonstrates that
  a real-Confluence + real-GH-mirror bus push fails for reasons the
  simulator-only tests cannot surface (missing mirror remote, missing
  fetch, frontmatter rejection of workflow files, refresh-emitted
  `.reposix/` files, no shared history, force semantics)

**Why it matters:** P82 ships the entire transport surface for the
v0.13.0 milestone vision (the bus URL is the third role's primary
interface). Per CLAUDE.md OP-6 ("Real backends are first-class test
targets"; "simulator-only coverage does NOT satisfy acceptance for
transport-layer ... claims"), the absence of a real-backend test is
itself a violation. The dark-factory exercise (T3-bus-push.md) found
8 frictions on real Confluence + GH mirror that no simulator test
caught.

---

### F4 — STEP 0 (`resolve_mirror_remote_name`) requires a documented-nowhere prerequisite that even the verdict does not flag [SEVERITY: HIGH]

**Claim in plan:** `82-PLAN-OVERVIEW/index.md:67-71` describes Q3.5
RATIFIED: "Zero matches → emit Q3.5 hint and exit BEFORE PRECHECK A."
Catalog row `agent-ux/bus-no-remote-configured-error` certifies the
error-path-on-zero-matches flow. CLAUDE.md says nothing about needing a
local `git remote add mirror <url>` separately from the bus URL.

**Reality:** The implementation requires the user to run *three separate
operations* before a bus push can succeed:
1. `git remote set-url origin reposix::<sot>?mirror=<plain>`
2. `git remote add mirror <plain>` (separate local remote, MUST exist)
3. `git fetch mirror` (so `refs/remotes/mirror/main` exists for
   PRECHECK A's `git rev-parse` at `bus_handler.rs:409-413`)

T3-bus-push.md F9 + F11 documented this as a HIGH friction with cryptic
errors at each step. The catalog row tests step (1) error path only
(`bus_no_remote_configured_emits_q35_hint`) — it does NOT test the
working steady-state where all three steps succeed against a real
backend. The verdict reports this as "Q3.5 RATIFIED satisfied" but the
satisfaction is single-error-path-only; the success path (mirror
configured + fetched + ref synced) lives only in the synthetic
`make_synced_mirror_fixture` fixture in `bus_precheck_b.rs:60-96` which
silently sets up all three preconditions invisibly.

**Evidence:**
- `crates/reposix-remote/src/bus_handler.rs:321-379` (STEP 0 body)
- `crates/reposix-remote/src/bus_handler.rs:386-433` (PRECHECK A
  shells out `git rev-parse refs/remotes/<name>/main`; if no local
  ref, treated as Drifted at line 418-421)
- `crates/reposix-remote/tests/bus_precheck_b.rs:60-96`
  (`make_synced_mirror_fixture` invisibly sets up all 3 preconditions)
- `.planning/research/v0.13.0-real-backend-frictions/01-dark-factory-may02/T3-bus-push.md:124-152`
  (F9 + F11 frictions on real backend)
- `docs/guides/dvcs-mirror-setup.md` (per F8 in T3) does NOT show the
  bus URL string OR the three-step prereq

**Why it matters:** This is failure shape #5 (documented user-facing
flow rejected by implementation). The Bus URL form documented in
CLAUDE.md is "single git remote, single push, fans out to both." The
real flow requires two local remotes + a `git fetch` *before the bus
URL can be used at all*. The catalog row only tests the negative path,
making this a structurally invisible friction.

---

### F5 — DVCS-BUS-URL-01 catalog row asserts `kind: Sim` that the test does not actually check [SEVERITY: LOW]

**Claim in catalog:** `quality/catalogs/agent-ux.json` row
`agent-ux/bus-url-parses-query-param-form` asserts:
> "bus_url::parse('reposix::sim::demo?mirror=file:///tmp/m.git') returns
> Route::Bus { sot: ParsedRemote { kind: Sim, project: 'demo', .. },
> mirror_url: 'file:///tmp/m.git' }"

**Reality:** The unit test at
`crates/reposix-remote/src/bus_url.rs:163-173` and integration test at
`crates/reposix-remote/tests/bus_url.rs:14-47` only assert
`sot.project == "demo"` and `mirror_url == "file:///tmp/m.git"`. Neither
test references `BackendKind::Sim` or the `kind` field. The URL used
in tests is `reposix::http://127.0.0.1:7878/projects/demo?...` (HTTP
form) or `reposix::http://127.0.0.1:9/projects/demo?...`, NOT the
catalog's quoted `reposix::sim::demo` shorthand.

**Evidence:**
- `crates/reposix-remote/src/bus_url.rs:163-173` (unit test body)
- `crates/reposix-remote/tests/bus_url.rs:14-47` (integration test body)
- `quality/catalogs/agent-ux.json` row asserts list

**Why it matters:** Doc-vs-code drift in the catalog row's assert
language. Not load-bearing — the implementation does work — but a
verifier reading the row's assert string and looking for `Sim`-typed
checks will not find them. This is the kind of small catalog-row
honesty drift that compounds over time.

---

### F6 — `bus_url::parse` accepts `?mirror=` with embedded `?` (no validation) but doc-comment says user MUST encode [SEVERITY: MED]

**Claim in module doc + CLAUDE.md:** `bus_url.rs:22-28`:
> "If the mirror URL itself contains `?` (e.g.,
> `https://gh.example/foo?token=secret`), the user MUST percent-encode
> the value. The first unescaped `?` in the bus URL is the bus
> query-string boundary."

**Reality:** The parser does not enforce, validate, or even detect this.
`split_once('?')` consumes everything after the first `?` into `query`;
the value of `mirror=` is then extracted via `split('&')` then
`split_once('=')`. An unencoded `?` in the mirror value is silently
included as part of the value. An unencoded `&` inside the mirror value
would split off the suffix and the parser would produce a misleading
"unknown query parameter `<suffix>`" error, NOT a clear "mirror URL
must be percent-encoded" diagnostic.

**Evidence:**
- `crates/reposix-remote/src/bus_url.rs:79-82, 108-118, 122-134`
- `crates/reposix-remote/src/bus_url.rs:22-28` (module-doc claim)
- No validation logic between the doc claim and the parsed value

**Why it matters:** Failure shape #5 again — the documented contract
says one thing, the code allows another. A naive user pasting a
GitHub URL with `?token=...` in shell would get a result that looks
correct but produces unpredictable behavior. The architecture-sketch
intends percent-encoding to be load-bearing for security (Q-C / D-03
forward-compat); silent acceptance erodes that contract.

---

### F7 — The verdict cites `bus_precheck_b.rs:262-273` asserting D-02 deferred-shipped error, but the test now asserts the OPPOSITE [SEVERITY: MED]

**Claim in P82 verdict:** Honesty spot-check #2 at
`quality/reports/verdicts/p82/VERDICT.md:79-87`:
> "The stderr substring 'bus write fan-out (DVCS-BUS-WRITE-01..06) is
> not yet shipped — lands in P83' is asserted by
> `tests/bus_precheck_b.rs::bus_precheck_b_passes_when_sot_stable` at
> line 270-273 ... assert!(stderr.contains('bus write fan-out')
> && stderr.contains('P83'), ...) — D-02 closed."

**Reality:** The current `tests/bus_precheck_b.rs` (after P83-01
landed at commit `6978369`) has REWRITTEN this test. Lines 268-280
now assert:
```rust
assert!(
    !stdout.contains("fetch first"),
    "PRECHECK B incorrectly tripped on stable SoT; ..."
);
assert!(
    !stdout.contains("bus-write-not-yet-shipped"),
    "P82 deferred-shipped stub re-appeared after P83-01 — ..."
);
```
That is, the test now asserts the *absence* of the very string the P82
verdict cited as proof the test asserted its presence. The verdict's
D-02 honesty spot-check is no longer literally true; D-02 was a
ratified P82 decision that *got removed* in P83 and the test was
inverted.

**Evidence:**
- `quality/reports/verdicts/p82/VERDICT.md:75-87` (verdict's claim)
- `crates/reposix-remote/tests/bus_precheck_b.rs:262-280` (current)
- Commit `6978369` ("feat(reposix-remote): bus_handler write fan-out
  replacing deferred-shipped stub")

**Why it matters:** The P82 verdict is now historically inaccurate
relative to the live tree. A future auditor reading the verdict to
confirm "D-02 closed" will not find the string in the tests. This is
not a P82 implementation bug — it's a verdict-archival vs.
mutable-tree mismatch that pollutes the audit trail. The verdict
should be marked SUPERSEDED-BY-P83-01 or amended.

---

### F8 — STEP 0's `--get-regexp` URL match silently swallows `git config` exit codes other than 0 or 1 with `Err(...)` BUT ignores stderr [SEVERITY: LOW]

**Claim in plan:** `82-RESEARCH/ch01-architecture.md` and PLAN T04
describe STEP 0 as the resolution layer for Q-A / D-01. The plan
documents the multi-match WARN policy but does not discuss the
error-on-non-1-non-0-exit branch.

**Reality:** `bus_handler.rs:329-338`:
```rust
if !out.status.success() {
    let exit = out.status.code().unwrap_or(-1);
    if exit == 1 {
        return Ok(None);
    }
    return Err(anyhow!(
        "`git config --get-regexp` exited {exit}: {}",
        String::from_utf8_lossy(&out.stderr)
    ));
}
```
This Err propagates up through `?` at `bus_handler.rs:142`, which is
inside `handle_bus_export`'s body. `handle_bus_export` returns
`Result<()>` — an `Err` here causes the parent to bubble the error
out of `real_main`, which prints `git-remote-reposix: <error>` and
exits 2 (`main.rs:108-110`). For the user, the result is a generic
catch-all error rather than a clean protocol error or the Q3.5 hint.

**Evidence:**
- `crates/reposix-remote/src/bus_handler.rs:321-379`
- `crates/reposix-remote/src/main.rs:107-111` (top-level catch)

**Why it matters:** Edge case (git config segfaults, signal-killed,
filesystem read failure on `.git/config`) but not represented in the
catalog or test surface. Pre-existing helper behavior; only worth
flagging because the plan claimed STEP 0 was fully designed.

---

### F9 — Empty `?mirror=` and `?mirror` (no-equals) error paths exist but are not integration-tested [SEVERITY: LOW]

**Claim in plan:** `T02-bus-url-parser.md` lists 4 unit tests; the
empty-value path at `bus_url.rs:143-148` is implemented and the
no-equals path at `bus_url.rs:114-117` is implemented. Catalog row
`bus-url-rejects-plus-delimited` covers the unknown-key reject but
not the empty-value or no-equals paths.

**Reality:** No integration test in `tests/bus_url.rs` exercises:
- `reposix::sim::demo?` (empty query string)
- `reposix::sim::demo?mirror=` (empty mirror value)
- `reposix::sim::demo?mirror` (no equals sign)
- `reposix::sim::demo?mirror=&priority=high` (mirror first, unknown
  key second — order matters for the for-loop break)

The unit tests inside `bus_url.rs::tests` also don't cover them.

**Evidence:**
- `crates/reposix-remote/src/bus_url.rs:101-153` (4 distinct error
  paths exist in `parse`)
- `crates/reposix-remote/src/bus_url.rs:156-224` (4 unit tests)
- `crates/reposix-remote/tests/bus_url.rs` (3 integration tests, none
  cover these forms)

**Why it matters:** Coverage gap. The branches exist; they are
unverified. Low because they are clean error paths returning the
expected `Result::Err(_)`, but a regression that swapped one error
message for another (or downgraded a hard error to a silent default)
would not be caught.

---

### F10 — CLAUDE.md "Bus URL form (P82+)" paragraph contradicts the actual fetch behavior [SEVERITY: HIGH]

**Claim in CLAUDE.md (line 26, in-phase update per QG-07):**
> "Push-only (Q3.4); fetch on a bus URL falls through to the
> single-backend path."

**Reality:** As established in F1, fetch on a bus URL *does not* fall
through to the v0.9 single-backend path (`stateless-connect`-based).
It falls through to the deprecated v0.8 `import` capability path
(`main.rs:213-215` calls `handle_import_batch`), which uses
`state.backend.list_records(&state.project)` to emit a fast-import
stream. The "single-backend path" phrase in CLAUDE.md is a noun phrase
that, to a cold reader, refers to `handle_stateless_connect` — the
documented v0.9 read path. The implementation routes to neither
single-backend's stateless-connect path NOR a dedicated bus-fetch
handler; it routes to a deprecated path slated for removal.

**Evidence:**
- `crates/reposix-remote/src/main.rs:175-198` (capabilities arm — both
  `import` and `stateless-connect` are conditionally advertised)
- `crates/reposix-remote/src/main.rs:213-222` (export branches on
  `mirror_url.is_some()`; import does NOT — bus-mode imports go through
  the same `handle_import_batch` as deprecated v0.8 sim)
- `crates/reposix-remote/src/main.rs:176-177` (comment: "the v0.8
  `import` capability (deprecated — one release cycle; Phase 36
  removes)")
- CLAUDE.md line 26

**Why it matters:** Failure shape #5 (documented user-facing flow
rejected by implementation; or in this case, executed via a path
the rest of the docs describe as deprecated). When `import` is removed,
bus URL fetch silently breaks. This is a CLAUDE.md doc-claim
inaccuracy that is harder to catch than missing tests because it
only manifests when a user actually tries `git fetch reposix::<sot>?mirror=...`
— which no test, and the dark-factory exercise's T3 (which used
`git pull --rebase` against the mirror remote, not the bus remote),
ever did.

---

### F11 — Verifier subagent dispatch (`pre-pr` cadence runner) was the only certifier; the dark-factory exercise that surfaced T3 frictions ran AFTER the verdict and was not part of phase close [SEVERITY: MED]

**Claim in plan:** Catalog-first contract (per CLAUDE.md QG-04) means
the FIRST commit writes catalog rows defining the GREEN contract;
verifier subagent grades from artifacts with zero session context.

**Reality:** The verdict at `quality/reports/verdicts/p82/VERDICT.md`
re-graded the six P82 catalog rows via
`python3 quality/runners/run.py --cadence pre-pr` (verdict line 14-26).
That runner only invokes the in-tree verifier shells under
`quality/gates/agent-ux/`, which in turn run `cargo test -p reposix-remote
--test bus_*`. The verifier never runs anything against a real backend;
the catalog rows have no `cadence: pre-release` marker that would
trigger real-backend coverage. The dark-factory T3 exercise that found
8 frictions (F8-F15) was conducted on 2026-05-02, AFTER the P82 verdict
was issued (2026-05-01). T3's findings are the empirical proof that the
verifier's contract was insufficient.

**Evidence:**
- `quality/reports/verdicts/p82/VERDICT.md:295-303`
- `quality/catalogs/agent-ux.json` (six P82 rows; `cadences: ["pre-pr"]`
  on every one)
- `.planning/research/v0.13.0-real-backend-frictions/01-dark-factory-may02/T3-bus-push.md`
  (dark-factory frictions)
- `crates/reposix-cli/tests/agent_flow_real.rs` (zero P82 surface
  exercised)

**Why it matters:** This is failure shape #2 (deferred / "substrate
gap" deferrals masquerading as GREEN). The framework had no `pre-release`
or real-backend cadence for the bus surface, so a structurally-passing
GREEN was assigned to a transport-layer feature that had never been
exercised against the real targets that v0.13.0 promises (TokenWorld,
`reubenjohn/reposix` mirror, JIRA TEST). This is the kind of
"framework-shaped exemption" the v0.13.1 audit is targeting.

---

### F12 — Plan promised "main.rs lines 150-172" capabilities branching; actual edit landed at lines 192-200 [SEVERITY: LOW]

**Claim in plan:** `82-PLAN-OVERVIEW/index.md:80-82`:
> "Capabilities branching (5-line edit at lines 150-172): `if
> matches!(route, Route::Single(_)) { proto.send_line('stateless-connect')?; }`"

**Reality:** The edit landed at `main.rs:192-200` (lines shifted because
of unrelated growth in the `import` capability comment block during
P82 development). The branching predicate is `state.mirror_url.is_none()`
(checked on State, not on Route directly), which is logically equivalent
but structurally different from the plan's `matches!(route,
Route::Single(_))`.

**Evidence:**
- `crates/reposix-remote/src/main.rs:192-200`
- `.planning/phases/82-bus-remote-url-parser/82-PLAN-OVERVIEW/index.md:80-82`
- D-06 ratification (verdict line 154-156): "3-line conditional addition"

**Why it matters:** Cosmetic. The verdict already noted the line-number
shift; it's an artifact of plan-was-written-before-final-line-numbers.
Worth flagging only because it shows the catalog row's `owner_hint`
("if RED: main.rs capabilities arm branching on state.mirror_url
regressed") wires to the State field, while the plan's
must_haves wires to the Route enum — two different mental models for
the same regression check.

---

### F13 — `bus_handler::resolve_mirror_remote_name` ignores `remote.<name>.pushurl` [SEVERITY: SUSPECT]

**Claim in plan:** D-01 / Q-A says "byte-equal-match values to
`mirror_url`". The plan does not specify whether `remote.<name>.pushurl`
(distinct from `remote.<name>.url`, used by git when push and fetch
URLs differ) should also be considered.

**Reality:** `resolve_mirror_remote_name` only matches against
`^remote\.([^.]+)\.url$` (line 323). If a user has set
`remote.mirror.url = <fetch-url>` and `remote.mirror.pushurl =
<push-mirror-url>`, the bus URL's `?mirror=<push-mirror-url>` would
fail to find any matching remote name and Q3.5 hint would fire
incorrectly.

**Evidence:**
- `crates/reposix-remote/src/bus_handler.rs:321-379`
- `git config(1)` documents `remote.<name>.pushurl` as a distinct key

**Why it matters:** Edge case for users with split push/fetch URLs.
SUSPECT because it's possible the plan deliberately scoped to `.url`
only and the plan-checker accepted that scope. Documenting the limit
would be a doc fix, not a code fix; resolving via `pushurl` when
present would be a small enhancement. Worth surfacing because the
catalog row description ("STEP 0 finds zero matches") could mislead
into believing all configurations are handled.

---

## Summary

**Eight HIGH/MED findings** cluster into three families:

1. **Structural-only verification** (F1, F3, F11) — catalog rows +
   verifier shells certify list-shape, capability-omission, and
   wiremock fixtures, but never exercise real-backend bus push or bus
   fetch. CLAUDE.md OP-6 declares this insufficient; the framework
   never required compliance.

2. **Plan-vs-ship narrative drift** (F2, F4, F6, F10) — CLAUDE.md and
   plan files document contracts (percent-encoding required;
   "single-backend path" for fetch; bus URL is a single-remote
   abstraction) that the implementation either does not enforce,
   does not implement, or implements via a deprecated fallback path.
   These manifested as concrete user friction in T3-bus-push.md
   (F9, F11, F12, F13 of that file).

3. **Verdict drift** (F7) — the P82 verdict cites text in tests that
   P83 has since deleted. The verdict file remains as-shipped,
   creating a misleading audit trail.

**Three LOW findings** (F5, F8, F9, F12) are coverage / cosmetic
gaps that don't block the milestone but compound into framework
debt over time.

**One SUSPECT finding** (F13) marks an edge-case scope question
(`pushurl` vs. `url`) that could be plan-scoped intentionally.

The phase technically meets its catalog-first contract: 6/6 rows
PASS, all unit + integration tests green, CLAUDE.md updated. The
gap is between catalog-first and behavior-first: every assertion
that requires going past `cargo test` to validate (real backend,
real GH SSH, real `git push`) is structurally absent. P82 is the
canonical case study for the v0.13.1 framework-fix phase: the
verifier subagent has no path to grade transport claims against
the targets the milestone was built to serve.
