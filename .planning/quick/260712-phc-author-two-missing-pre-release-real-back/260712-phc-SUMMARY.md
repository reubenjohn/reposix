---
phase: quick-260712-phc
plan: 01
subsystem: testing
tags: [quality-gates, shell, pre-release-real-backend, confluence, github, kcov]

# Dependency graph
requires:
  - phase: quick-260712-oke
    provides: v0.15.0 GOOD-TO-HAVES/ROADMAP surface (unrelated intake landing, prior session)
provides:
  - "B4 verifier: quality/gates/agent-ux/t4-conflict-rebase-ancestry-real-backend.sh + sourced lib/t4-real-backend-flow.sh"
  - "B5 verifier: quality/gates/agent-ux/github-front-door-real-backend.sh"
  - "Hermetic self-test: quality/gates/agent-ux/real-backend-env-gate.selftest.sh"
  - "kcov coverage harness: quality/gates/code/shell-coverage-tests/real-backend-env-gate.sh"
  - "B5 catalog transcript-filename fix-twice in quality/catalogs/agent-ux.json"
affects: [v0.14.0-tag-owner-decision, pre-release-real-backend-cadence, milestone-close-9th-probe]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Sourced-only .sh lib factoring under the 10k file-size budget (dark-factory/lib.sh + lib/litmus-flow.sh precedent)"
    - "EXIT-trap artifact finalizer for kind:mechanical verifiers (single write site for every exit path)"
    - "ASSERT <label>: PASS|FAIL stdout convention for kind:shell-subprocess rows (F-K4b congruence readiness)"

key-files:
  created:
    - quality/gates/agent-ux/t4-conflict-rebase-ancestry-real-backend.sh
    - quality/gates/agent-ux/lib/t4-real-backend-flow.sh
    - quality/gates/agent-ux/github-front-door-real-backend.sh
    - quality/gates/agent-ux/real-backend-env-gate.selftest.sh
    - quality/gates/code/shell-coverage-tests/real-backend-env-gate.sh
  modified:
    - quality/catalogs/agent-ux.json
    - quality/gates/agent-ux/lib/transcript.sh

key-decisions:
  - "B4 uses kind:mechanical (no transcript.sh) and writes its own artifact via an EXIT trap so every exit path (env-gate/sanctioned-target/git-version/scenario) is covered by one write site"
  - "B4 was split into a main script + sourced lib/t4-real-backend-flow.sh after the initial single-file draft landed at 15657 chars, over the 10000-char .sh budget"
  - "B5 drives the real reposix binary's init subcommand directly (not a cargo test wrapper) since transport_claim:true requires a real binary/backend invocation"
  - "B5's raw-slug-not-sanitized assert is proven via remote.origin.url inspection post-init, since the outbound HTTP path isn't independently observable from a plain CLI invocation"

requirements-completed: [agent-ux/t4-conflict-rebase-ancestry-real-backend, agent-ux/github-front-door-real-backend]

# Metrics
duration: ~25min (research/reading) + 7min (commit span)
completed: 2026-07-12
---

# Quick 260712-phc: Author two missing pre-release-real-backend verifiers Summary

**Authored the B4 (Confluence two-writer conflict/ancestry) and B5 (GitHub front-door 200-not-404) real-backend verifier scripts that were blocking the v0.14.0 tag's non-skippable 9th probe — both now exist, are executable, git-tracked, and honestly grade NOT-VERIFIED/env-missing (never "verifier not found") when creds are absent.**

## Performance

- **Duration:** commit span ~7 min (18:45:30 → 18:52:09), plus prior research/reading of 8+ reference scripts and the runner internals
- **Started:** 2026-07-12T18:45:30-07:00 (first task commit)
- **Completed:** 2026-07-12T18:52:09-07:00 (last task commit)
- **Tasks:** 3/3 complete (plus 1 deviation commit)
- **Files modified:** 7 (5 created, 2 modified)

## Accomplishments

- B4 `t4-conflict-rebase-ancestry-real-backend.sh` ports the sim-arm two-writer conflict/ancestry scenario onto real Confluence TokenWorld, with env-gate-first, sanctioned-target+tenant guard, git-version gate, the confluence `pages/` bucket (never `issues/`), and a mass-delete guard that refuses any delete-shaped push diff or protected-fixture (7766017/7798785) touch before either push.
- B5 `github-front-door-real-backend.sh` drives a real `reposix init github::reubenjohn/reposix` under creds, asserting HTTP 200 (not 404), the raw (unsanitized) slug, and emitting `ASSERT ...: PASS|FAIL` lines for F-K4b congruence.
- Fixed the B5 catalog row's transcript-filename assert (dropped the stale `-<RFC3339>` segment to match `transcript.sh`'s actual stable-filename behavior) — `agent-ux.json` remains valid JSON.
- Added a hermetic self-test (both scripts, no creds, exit 75 + well-formed artifact) and a kcov coverage harness so the two new scripts' env-gate lines count toward the shell-coverage aggregate.
- Confirmed via a validate-only, no-creds, no-`--persist` runner invocation that both rows now grade `NOT-VERIFIED`/env-missing — the string `"verifier not found"` no longer appears for either script.

## Task Commits

1. **Task 1: Author B4 t4-conflict-rebase-ancestry-real-backend.sh** - `b635c3b` (feat)
   - *Deviation follow-up:* **Task 1a: Split B4 under the 10k .sh file-size budget** - `8c48fc5` (refactor)
2. **Task 2: Author B5 github-front-door-real-backend.sh + catalog fix-twice** - `1467eb2` (feat)
3. **Task 3: Hermetic self-test + kcov coverage harness** - `fe8febb` (test)

_Plan metadata (STATE.md/ROADMAP.md/this SUMMARY) will be committed separately by the orchestrator, per this session's constraints._

## Files Created/Modified

- `quality/gates/agent-ux/t4-conflict-rebase-ancestry-real-backend.sh` - B4 main verifier: env-gate, sanctioned-target+tenant guard, git-version gate, cargo build, EXIT-trap artifact finalizer, then sources the flow lib
- `quality/gates/agent-ux/lib/t4-real-backend-flow.sh` - B4's two-cache bootstrap + bucket-aware record select + mass-delete guard + two-writer conflict/refetch-ancestry scenario body (factored out under the 10k budget)
- `quality/gates/agent-ux/github-front-door-real-backend.sh` - B5 verifier: env-gate (writes its own NOT-VERIFIED artifact since transcript.sh hasn't run yet), sanctioned-target guard, real-run body wrapped by `write_transcript_and_artifact`
- `quality/gates/agent-ux/real-backend-env-gate.selftest.sh` - hermetic self-test asserting both scripts exit 75 + write well-formed NOT-VERIFIED artifacts with all creds unset
- `quality/gates/code/shell-coverage-tests/real-backend-env-gate.sh` - kcov harness driving both scripts' no-creds env-gate path
- `quality/catalogs/agent-ux.json` - B5 row's transcript assert corrected to the stable filename (fix-twice)
- `quality/gates/agent-ux/lib/transcript.sh` - corrected the same stale `-<RFC3339>` claim in the file's own top-of-file docstring (the function body already documented the stable-filename behavior correctly; only the header comment had drifted)

## Decisions Made

- **B4 artifact strategy:** since `kind:mechanical` rows get no automatic stdout-ASSERT parsing (that's a `transcript.sh`-only mechanism), B4 writes its own artifact via a single `finalize_artifact` EXIT trap, so every exit path (env-gate NOT-VERIFIED, sanctioned-target FAIL, git-version NOT-VERIFIED, scenario FAIL, and the eventual real PASS) is covered by one well-formed write site, with `asserts_passed` populated for F-K4b congruence on the eventual real PASS grade.
- **B4 file-size split:** the initial single-file draft landed at 15657 chars, over the `structure/file-size-limits` 10000-char ceiling for `.sh` files (the row is currently WAIVED `--warn-only` until 2026-08-08, so this only printed a pre-commit WARN, not a block). Rather than ride the waiver, split the two-cache scenario body + mass-delete guard into a sourced-only `lib/t4-real-backend-flow.sh`, mirroring the project's own `dark-factory/lib.sh` / `lib/litmus-flow.sh` factoring precedent. Both files now land at 9097 + 8777 chars.
- **B5 real-run invocation:** drives the real `reposix` binary's `init` subcommand directly (not a `cargo test` wrapper, unlike the p93/attach-sync-real-backend sibling scripts) since the row's `transport_claim: true` requires a real binary/backend invocation, and this is the more literal "front door" proof.
- **B5 raw-slug assertion:** since the outbound HTTP request path isn't independently observable from a plain CLI invocation without instrumentation, the "raw slug, never sanitized" assert is proven via `git config remote.origin.url` on the resulting checkout (which encodes the project slug), checking it contains the raw `owner/repo` form and not the sanitized `owner-repo` form.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug/hygiene] Split B4 verifier to fit the file-size budget**
- **Found during:** Task 1 (authoring B4)
- **Issue:** The initial single-file B4 script was 15657 chars, over the `structure/file-size-limits` 10000-char ceiling for `.sh` files. The row currently carries a `--warn-only` waiver (until 2026-08-08), so this printed a WARN rather than blocking — but riding an expiring waiver for a brand-new file is not "excellent," it's borrowing time.
- **Fix:** Factored the two-cache bootstrap + bucket-aware record select + mass-delete guard + scenario body into a new sourced-only `quality/gates/agent-ux/lib/t4-real-backend-flow.sh`, mirroring the project's own `dark-factory/lib.sh` / `lib/litmus-flow.sh` precedent exactly. Not listed in the plan's `files_modified` frontmatter — added as a deviation.
- **Files modified:** `quality/gates/agent-ux/t4-conflict-rebase-ancestry-real-backend.sh`, `quality/gates/agent-ux/lib/t4-real-backend-flow.sh`
- **Verification:** Both files now 9097 / 8777 chars (under 10000); re-ran the no-creds env-gate check post-split — still exits 75 with a well-formed artifact; pre-commit WARN no longer prints.
- **Committed in:** `8c48fc5`

**2. [Rule 1 - Bug/hygiene] Fixed the same stale RFC3339 claim in transcript.sh's own docstring**
- **Found during:** Task 2 (catalog fix-twice)
- **Issue:** While fixing the B5 catalog row's stale `-<RFC3339>` transcript-filename assert, noticed `quality/gates/agent-ux/lib/transcript.sh`'s own top-of-file docstring (lines 5-6) still claimed the transcript carries an RFC3339 stamp — directly contradicted by the function body 15 lines below it, which correctly documents the D-P96-01 stable-filename fix. Same bug class as the catalog row I was already fixing.
- **Fix:** Corrected the docstring to describe the stable filename (no RFC3339 stamp), cross-referencing the D-P96-01 rationale in the function body.
- **Files modified:** `quality/gates/agent-ux/lib/transcript.sh`
- **Verification:** `bash -n` syntax check; the function body's actual behavior is unchanged (doc-only fix).
- **Committed in:** `1467eb2` (part of the Task 2 commit)

---

**Total deviations:** 2 auto-fixed (1 structural/hygiene split, 1 doc-drift fix), both Rule 1.
**Impact on plan:** Neither changed the verifiers' behavior or the plan's acceptance criteria; both are code-quality corrections directly adjacent to the task at hand. No scope creep.

## Issues Encountered

None beyond the two deviations above. The local git version (2.25.1) is below B4's `>= 2.34` gate, but this never mattered for this hermetic plan since the env-gate (missing creds) short-circuits before the git-version check is ever reached.

## Acceptance Proof

**1. Both scripts exist, executable, git-tracked:**

```
100755 6644b11f... quality/gates/agent-ux/t4-conflict-rebase-ancestry-real-backend.sh
100755 70bf5632... quality/gates/agent-ux/github-front-door-real-backend.sh
```

(plus the sourced `lib/t4-real-backend-flow.sh`, also 100755/tracked)

**2. Each, run standalone with ALL real-backend creds/allowlist unset, exits 75 and writes a well-formed NOT-VERIFIED artifact:**

B4 (`env -u ATLASSIAN_API_KEY -u ATLASSIAN_EMAIL -u REPOSIX_CONFLUENCE_TENANT -u REPOSIX_ALLOWED_ORIGINS bash quality/gates/agent-ux/t4-conflict-rebase-ancestry-real-backend.sh`):
```
NOT-VERIFIED: real-backend creds/allowlist unset: ATLASSIAN_API_KEY ATLASSIAN_EMAIL REPOSIX_CONFLUENCE_TENANT REPOSIX_ALLOWED_ORIGINS
  env-gate: exit 75 -> runner maps to NOT-VERIFIED (never skip-as-pass, OD-2)
EXIT=75
```
artifact: `{"exit_code": 75, "status": "NOT-VERIFIED", "skip_reason": "env-missing", ...}`

B5 (`env -u GITHUB_TOKEN -u REPOSIX_ALLOWED_ORIGINS bash quality/gates/agent-ux/github-front-door-real-backend.sh`):
```
NOT-VERIFIED: real-backend creds/allowlist unset: GITHUB_TOKEN REPOSIX_ALLOWED_ORIGINS
  env-gate: exit 75 -> runner maps to NOT-VERIFIED (never skip-as-pass, OD-2)
EXIT=75
```
artifact: `{"exit_code": 75, "status": "NOT-VERIFIED", "skip_reason": "env-missing", ...}`

**3. `run.py --cadence pre-release-real-backend` (validate-only, no creds, no `--persist`) shows BOTH rows NOT-VERIFIED/env-missing, no "verifier not found":**

```
quality/runners/run.py --cadence pre-release-real-backend  [validate-only (catalog writes OFF)]
  catalog: agent-ux.json (66 rows; 6 in scope)
    [NOT-VERIFIED ] agent-ux/milestone-close-vision-litmus-real-backend  (P0, 0.00s) -> skipped: env not set (real-backend origins/creds absent)
    [NOT-VERIFIED ] agent-ux/t4-conflict-rebase-ancestry-real-backend  (P0, 0.00s) -> skipped: env not set (real-backend origins/creds absent)
    [NOT-VERIFIED ] agent-ux/p93-partial-failure-recovery-real-confluence  (P0, 0.00s) -> skipped: env not set (real-backend origins/creds absent)
    [NOT-VERIFIED ] agent-ux/cadence-pre-release-real-backend  (P1, 0.00s) -> skipped: env not set (real-backend origins/creds absent)
    [NOT-VERIFIED ] agent-ux/attach-sync-real-backend  (P1, 0.00s) -> skipped: env not set (real-backend origins/creds absent)
    [NOT-VERIFIED ] agent-ux/github-front-door-real-backend  (P1, 0.00s) -> skipped: env not set (real-backend origins/creds absent)
summary: 0 PASS, 0 FAIL, 0 PARTIAL, 0 WAIVED, 6 NOT-VERIFIED -> exit=1
```
Full output captured at `accept-proof.txt` (this directory). `grep -cF 'verifier not found'` against both script paths returns `0` for each — the string is gone.

**Note on WHY this already worked before the scripts existed for the no-creds case:** `quality/runners/_realbackend.py:is_skipped` short-circuits any row tagged `pre-release-real-backend` BEFORE the runner even checks whether the verifier script exists, whenever env creds are fully absent (as in this hermetic no-creds run) — so the "verifier not found" string was never going to appear in a *fully*-no-creds run regardless of whether the scripts existed. The scripts matter for: (a) a **partially**-credentialed run (e.g. only `GITHUB_TOKEN` set, no Confluence creds — `is_skipped` would then be `False` for BOTH rows since it checks "any one complete cred set," and the runner would actually try to invoke the script, previously hitting "verifier not found" for whichever row's script was missing), (b) a **standalone** invocation (an owner running `bash <script>` directly, never going through `run.py`), and (c) being the actual substrate the eventual real PASS grade needs to exist at all. Task 1 & 2's automated verify (direct `bash <script>` invocation, bypassing the runner) is the test that actually exercises the new code; this is documented here for the record so a future reader isn't confused about why the acceptance item's grep is a `0` either way.

**4. B5 catalog transcript assert corrected; `agent-ux.json` valid JSON:**

```
$ python3 -c "import json;json.load(open('quality/catalogs/agent-ux.json'));print('VALID JSON, rows=', len(...['rows']))"
VALID JSON, rows= 66
```
Row `agent-ux/github-front-door-real-backend`'s transcript assert now reads:
`"a transcript is written to quality/reports/transcripts/github-front-door-real-backend.txt recording argv + env_keys (NAMES only) + cwd + exit_code + stdout/stderr"` (no `-<RFC3339>` segment).

**5. Hermetic self-test tracked + passing:**

```
$ bash quality/gates/agent-ux/real-backend-env-gate.selftest.sh
== B4: t4-conflict-rebase-ancestry-real-backend ==
  PASS: B4 exits 75 with all real-backend creds/allowlist unset
  PASS: B4 artifact is a well-formed NOT-VERIFIED (exit_code=75) at quality/reports/verifications/agent-ux/t4-conflict-rebase-ancestry-real-backend.json
== B5: github-front-door-real-backend ==
  PASS: B5 exits 75 with all real-backend creds/allowlist unset
  PASS: B5 artifact is a well-formed NOT-VERIFIED (exit_code=75) at quality/reports/verifications/agent-ux/github-front-door-real-backend.json

RESULT: 4 passed, 0 failed
```

**Shell-coverage confirmation (`bash quality/gates/code/shell-coverage.sh`, needs kcov — installed, kcov 38):**

```
quality/gates/agent-ux/github-front-door-real-backend.sh              18     59   30.5%  yes
quality/gates/agent-ux/t4-conflict-rebase-ancestry-real-backend.sh     31     66   47.0%  yes
...
AGGREGATE 15.72%  (866/5510 lines)  floor=13.00%
counter validation: clean (all executed files within 15%)
PASS: aggregate shell line-coverage >= floor
```
No regression; floor NOT lowered. `quality/gates/agent-ux/lib/t4-real-backend-flow.sh` (0%, not seen) and `quality/gates/agent-ux/real-backend-env-gate.selftest.sh` (0%, not seen) sit uncovered by design — the lib is only reached past the env-gate short-circuit (never exercised hermetically, matching the "real PASS grading is a separate owner-gated run" scope fence), and no existing harness covers ANY `.selftest.sh` file project-wide either (confirmed: `file-size-limits.selftest.sh` is likewise uncovered) — this is precedented, not a new gap.

## Noticing

Per the ownership charter (OD-3), things noticed near this work:

1. **Widespread stale `-<RFC3339>` transcript-filename references beyond the B5 row.** The plan scoped the fix-twice to ONLY the `agent-ux/github-front-door-real-backend` row (honored — did not touch other rows). But `quality/catalogs/agent-ux.json` carries the SAME stale pattern in at least four more places: `agent-ux/kind-shell-subprocess-worked-example` (both an `expected.assert` and its `claim_vs_assertion_audit`), `agent-ux/attach-sync-real-backend` (a `transcript_path` example value + an `owner_hint`), and `agent-ux/rebase-recovery-reconciles` (a `transcript_path` example value). These are all pre-existing, out of this plan's scope (SCOPE BOUNDARY: only auto-fix issues directly caused by the current task's changes) — **not fixed**. Recommend filing to the next `GOOD-TO-HAVES.md` intake (v0.15.0 surface) as a small doc-hygiene sweep: "grep `agent-ux.json` for `-<RFC3339>` and correct every stale reference to match transcript.sh's actual stable-filename behavior (D-P96-01)."
2. **Confluence `pages/`-vs-`issues/` export risk for B4's eventual real run (flagged per the plan's explicit ask).** Per `lib/litmus-flow.sh`'s GUARD B comment, the push planner is now believed id-keyed/bucket-agnostic (Wave-5.5 fix), so the historical "confluence export only recognises `issues/<id>.md`" mass-delete bug that motivated `milestone-close-vision-litmus.sh`'s guard is likely already closed at the planner layer. B4's own mass-delete guard (refuse any delete-shaped diff or protected-fixture touch) is authored as defense-in-depth, not because the planner bug is known-live. **If the eventual owner-gated real run of B4 ever trips this guard, that itself is a genuine finding worth a fresh `SURPRISES-INTAKE` entry**, not a signal to loosen the guard.
3. **`_realbackend.is_skipped`'s generic (not per-backend) credential check** means a row tagged `pre-release-real-backend` will be actually invoked by the runner whenever ANY ONE backend's complete credential set is present (e.g., `GITHUB_TOKEN` alone), even for a row whose OWN scenario needs a completely different backend (e.g., B4's Confluence-only row). This is why each script's OWN inline env-gate (checking its specific backend's creds) is load-bearing, not redundant with the runner-level skip — worth calling out explicitly since it's easy to assume the runner-level gate alone is sufficient per-row protection. No code change needed; the existing per-script gates already handle it correctly (this is a note for future gate authors, not a bug).

No `<1h` eager-fixable items beyond the two already folded into the Deviations section above.

## User Setup Required

None - no external service configuration required. This plan deliberately never touches a real backend (hermetic-only scope fence); the real credential setup for the eventual owner-gated PASS grade is out of scope here.

## Next Phase Readiness

- Both previously-missing `pre-release-real-backend` verifier scripts now exist, are git-tracked, and honestly grade `NOT-VERIFIED`/env-missing — the v0.14.0 tag's 9th-probe blocker of `"verifier not found"` is resolved.
- **Still owner-gated and NOT this plan's job:** running the actual real-backend PASS grade (owner supplies `ATLASSIAN_API_KEY`/`ATLASSIAN_EMAIL`/`REPOSIX_CONFLUENCE_TENANT` + `GITHUB_TOKEN` + a non-loopback `REPOSIX_ALLOWED_ORIGINS`, then re-runs `pre-release-real-backend --persist`) is a separate, out-of-scope item per the plan's explicit scope fence.
- No blockers for the tag decision from this plan's side; the remaining gate is the owner's real-credential run.

---
*Phase: quick-260712-phc*
*Completed: 2026-07-12*

## Self-Check: PASSED

All 8 claimed files found on disk (5 created scripts, 2 modified files, 1 accept-proof
artifact); all 4 claimed commit hashes (`b635c3b`, `8c48fc5`, `1467eb2`, `fe8febb`) found
in `git log --oneline --all`. No missing items.
