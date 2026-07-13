---
phase: quick-260712-phc
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - quality/gates/agent-ux/t4-conflict-rebase-ancestry-real-backend.sh
  - quality/gates/agent-ux/github-front-door-real-backend.sh
  - quality/gates/agent-ux/real-backend-env-gate.selftest.sh
  - quality/gates/code/shell-coverage-tests/real-backend-env-gate.sh
  - quality/catalogs/agent-ux.json
autonomous: true
requirements:
  - agent-ux/t4-conflict-rebase-ancestry-real-backend   # B4, P0
  - agent-ux/github-front-door-real-backend             # B5, P1
user_setup: []

must_haves:
  truths:
    - "Both verifier scripts exist on disk, are executable (chmod +x), and are git-tracked"
    - "Run standalone with NO creds/allowlist, each script exits 75 and writes a well-formed NOT-VERIFIED artifact"
    - "`run.py --cadence pre-release-real-backend` (validate-only, no creds) grades BOTH rows NOT-VERIFIED/env-missing, NOT 'verifier not found'"
    - "The B5 catalog row's transcript assert names the STABLE filename (no -<RFC3339>) and agent-ux.json is still valid JSON"
    - "The hermetic self-test is git-tracked and passes; pre-push shell-coverage aggregate stays >= floor (13)"
  artifacts:
    - path: "quality/gates/agent-ux/t4-conflict-rebase-ancestry-real-backend.sh"
      provides: "B4 real-backend (TokenWorld) two-writer conflict/ancestry verifier; env-gate-first -> exit 75 + NOT-VERIFIED artifact"
      contains: "exit 75"
      min_lines: 90
    - path: "quality/gates/agent-ux/github-front-door-real-backend.sh"
      provides: "B5 real-backend GitHub front-door 200-not-404 verifier; transcript via lib/transcript.sh; env-gate-first -> exit 75"
      contains: "write_transcript_and_artifact"
      min_lines: 60
    - path: "quality/gates/agent-ux/real-backend-env-gate.selftest.sh"
      provides: "Hermetic self-test: each new script no-creds -> exit 75 + valid NOT-VERIFIED artifact"
      contains: "NOT-VERIFIED"
      min_lines: 40
    - path: "quality/gates/code/shell-coverage-tests/real-backend-env-gate.sh"
      provides: "kcov harness driving both scripts' no-creds env-gate path so the pre-push shell-coverage aggregate does not regress"
    - path: "quality/catalogs/agent-ux.json"
      provides: "B5 row transcript assert corrected to the stable filename (fix-twice)"
  key_links:
    - from: "quality/gates/agent-ux/github-front-door-real-backend.sh"
      to: "quality/gates/agent-ux/lib/transcript.sh"
      via: "source + write_transcript_and_artifact \"github-front-door-real-backend\""
      pattern: "write_transcript_and_artifact"
    - from: "both new scripts (exit 75)"
      to: "quality/runners/_realbackend.py:map_exit_code_to_status"
      via: "runner maps exit 75 -> NOT-VERIFIED (never skip-as-pass, OD-2)"
      pattern: "exit 75"
    - from: "each script's inline env-gate NOT-VERIFIED artifact write"
      to: "the row's artifact path quality/reports/verifications/agent-ux/<slug>.json"
      via: "same slug -> same artifact path the catalog row references"
      pattern: "quality/reports/verifications/agent-ux/"
---

<objective>
Author the two MISSING `pre-release-real-backend` verifier scripts that block the
v0.14.0 tag: B4 (`agent-ux/t4-conflict-rebase-ancestry-real-backend`, P0) and B5
(`agent-ux/github-front-door-real-backend`, P1). Their catalog rows already exist in
`quality/catalogs/agent-ux.json`, but the scripts were never shipped, so the 9th probe
grades them NOT-VERIFIED with `"error": "verifier not found"`. Author both scripts +
a hermetic self-test + a shell-coverage harness + the B5 catalog fix-twice, so the rows
grade honestly (NOT-VERIFIED/env-missing when creds are unset) instead of "verifier not
found".

Purpose: unblock the owner-gated v0.14.0 tag's non-skippable 9th probe. Once the scripts
exist, the two rows are real (gradeable), and the later owner-run real-backend cadence
(a SEPARATE, out-of-scope item) can grade them PASS against live TokenWorld / GitHub.

Output: two executable, git-tracked verifier scripts; one hermetic self-test; one kcov
harness; a corrected B5 catalog assert.

**SCOPE FENCE — HERMETIC ONLY.** This plan MUST NOT hit any real backend. Every task
exercises ONLY the no-creds env-gate path (exit 75 BEFORE any cargo build / `reposix
init` / network call). Do NOT run any verifier with real creds. Do NOT pass `--persist`.
Real-backend PASS grading is a downstream owner-gated item, not this plan's job. The
scripts are authored correct-by-construction for that later run, but this plan proves
only the fail-closed env-gate.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/STATE.md
@quality/PROTOCOL.md

# --- Verbatim template scripts to PORT / mirror (read before authoring) ---
@quality/gates/agent-ux/t4-conflict-rebase-ancestry.sh        # B4 sim-arm — PORT this
@quality/gates/agent-ux/p93-partial-failure-recovery-real-confluence.sh  # env-gate + sanctioned-target + exit-75
@quality/gates/agent-ux/attach-sync-real-backend.sh           # env-gate + sanctioned-target pattern
@quality/gates/agent-ux/milestone-close-vision-litmus.sh      # mass-delete GUARD + protected-fixture denylist
@quality/gates/agent-ux/lib/transcript.sh                     # B5 MUST use write_transcript_and_artifact (STABLE filename)
@quality/gates/structure/file-size-limits.selftest.sh         # self-test idiom to mirror

<interfaces>
<!-- Extracted contracts the executor uses directly — no codebase exploration needed. -->

Runner exit-code mapping (quality/runners/_realbackend.py:map_exit_code_to_status):
  0 -> PASS   2 -> PARTIAL   75 -> NOT-VERIFIED   anything else -> FAIL
A verifier whose script file is MISSING flips the row to NOT-VERIFIED with
`error: "verifier not found at <path>"` (run.py:280). Once the script EXISTS and env-gates
to `exit 75`, the runner records NOT-VERIFIED + `skip_reason: "env-missing"` — that is the
target state this plan produces (replacing "verifier not found").

Confluence bucket (crates/reposix-core/src/path.rs):
  bucket_for_backend("confluence") == "pages"   (every OTHER backend == "issues")
  So a confluence checkout writes records under `pages/<id>.md`, NOT `issues/<id>.md`.
  B4 (confluence::TokenWorld) MUST glob the record file from the `pages/` bucket — do NOT
  hardcode `issues/` (the sim-arm's `issues/` is sim-specific).

transcript.sh (lib/transcript.sh):
  write_transcript_and_artifact <slug> <argv...>
  Writes STABLE transcript quality/reports/transcripts/<slug>.txt (NO RFC3339 stamp) +
  JSON artifact quality/reports/verifications/agent-ux/<slug>.json. It parses the invoked
  command's stdout `ASSERT <label>: PASS|FAIL` lines into asserts_passed. env_keys emit
  NAMES only (never values — security).

Artifact write paths (must match the catalog rows verbatim):
  B4: quality/reports/verifications/agent-ux/t4-conflict-rebase-ancestry-real-backend.json
  B5: quality/reports/verifications/agent-ux/github-front-door-real-backend.json
  Both directories are gitignored (per-run snapshots) — writing there never dirties the
  tracked tree.

shell-coverage (pre-push ratchet): the in-scope corpus = tracked .sh/.bash files (git
ls-files) minus excluded roots. Two new mostly-real-backend scripts at 0% could drop the
aggregate below the floor (quality/shell-coverage-floor.txt == 13). Harnesses live under
quality/gates/code/shell-coverage-tests/*.sh; a harness that RUNS the scripts' env-gate
path credits those lines. NEVER lower the floor to force-pass (silent-downgrade prohibited,
PROTOCOL.md waiver protocol).

Catalog schema (quality/catalogs/agent-ux.json): top-level object with keys
`$schema, comment, dimension, rows`. Rows are under the `rows` array (each has `id`,
`expected.asserts`, `verifier.script`, `artifact`, ...).
</interfaces>

# --- Catalog rows (GREEN contracts — do NOT edit their asserts except the B5 fix-twice) ---
# agent-ux/t4-conflict-rebase-ancestry-real-backend : quality/catalogs/agent-ux.json ~L1740
# agent-ux/github-front-door-real-backend           : quality/catalogs/agent-ux.json ~L2433
</context>

<tasks>

<task type="auto">
  <name>Task 1: Author B4 — t4-conflict-rebase-ancestry-real-backend.sh (P0, mechanical)</name>
  <files>quality/gates/agent-ux/t4-conflict-rebase-ancestry-real-backend.sh</files>
  <action>
PORT `quality/gates/agent-ux/t4-conflict-rebase-ancestry.sh` (the sim-arm sibling) into a
real-backend arm. `kind: mechanical` — NO transcript helper. Keep the two-independent-writer
topology (trees A/B, SEPARATE `REPOSIX_CACHE_DIR` cacheA/cacheB). Set
ROW_ID="agent-ux/t4-conflict-rebase-ancestry-real-backend" and
ARTIFACT="quality/reports/verifications/agent-ux/t4-conflict-rebase-ancestry-real-backend.json".

Order the guards so the no-creds path exits BEFORE any cargo/init (this is the hermetic
property the self-test relies on):

1. **ENV-GATE FIRST** (copy the p93 / attach-sync pattern, before any cargo/init):
   require ATLASSIAN_API_KEY + ATLASSIAN_EMAIL + REPOSIX_CONFLUENCE_TENANT set AND a
   non-empty REPOSIX_ALLOWED_ORIGINS. Any missing → WRITE a well-formed NOT-VERIFIED
   artifact to ARTIFACT (fields: `ts`, `row_id`, `exit_code`: 75, `status`: "NOT-VERIFIED",
   `reason`/`skip_reason`: "env-missing", `asserts_passed`: [], `asserts_failed`: [names of
   missing vars]) — mirror the git-too-old artifact shape already in the sim-arm — then
   print the p93-style stderr line and `exit 75`.
2. **Sanctioned-target guard** (OD-2 hard-FAIL, NOT 75): SPACE="${REPOSIX_CONFLUENCE_SPACE:-TokenWorld}".
   This row MUTATES the backend, so ONLY TokenWorld is sanctioned — any other space →
   `exit 1` (like p93; reject unknown real state). Also assert TENANT == "reuben-john"
   (mirror milestone-close-vision-litmus) → `exit 1` otherwise.
3. **git >= 2.34 precondition** (verbatim from the sim-arm) → on too-old, write the same
   NOT-VERIFIED artifact shape + `exit 75`.
4. **Build binaries once** — ONE cargo invocation, machine-wide:
   `cargo build -p reposix-cli -p reposix-remote --bin reposix --bin git-remote-reposix`
   and export PATH so `git-remote-reposix` is found (the push path needs it — see
   milestone-close-vision-litmus). Do NOT use `cargo build --workspace`.
5. **Two-cache scenario** in a `/tmp` run dir (leaf isolation — every reposix/git setup in
   `/tmp`, `cd`/`-C` in the SAME invocation): `reposix init "confluence::TokenWorld"` into A
   with REPOSIX_CACHE_DIR=cacheA, and into B with REPOSIX_CACHE_DIR=cacheB. Set distinct
   `git config user.email/name` per tree.
6. **Bucket-aware record path**: confluence records live under `pages/` (NOT `issues/` —
   `bucket_for_backend("confluence")=="pages"`). Glob the record file from A's `pages/`
   bucket (`find "$A/pages" -maxdepth 1 -name '*.md' | sort | head -1`); derive B's
   matching path by basename. Do NOT hardcode `issues/`.
7. **Scenario**: A appends a line to the record + commits + `git push origin main`
   (baseline, must succeed). B appends a DIFFERENT line to the SAME record (stale base) +
   commits + pushes → capture the push; assert it was REJECTED and its stderr matches
   `version mismatch|fetch first`. B recovers via `git fetch origin`. ASSERT the root commit
   (`git rev-list --max-parents=0 refs/reposix/origin/main | tail -1`) is IDENTICAL before
   and after the refetch, AND that the ref advanced (`git rev-list --count
   refs/reposix/origin/main` > 1 — non-vacuous), exactly as the sim-arm does.
8. **MASS-DELETE GUARD (critical — real TokenWorld safety, learned in
   milestone-close-vision-litmus)**: the confluence export/push diff historically only
   recognises `issues/<id>.md`; a `pages/`-based tree can make every cached record look
   DELETED and MASS-DELETE the space. Before B's baseline/A's push, REFUSE any delete-shaped
   diff (if the export plan would delete records the scenario did not intend, hard-FAIL
   `exit 1`, never push). Carry a protected-fixture denylist (` 7766017 7798785 ` and the
   TokenWorld durable fixtures per docs/reference/testing-targets.md) — never edit those.
   The scenario only edits ONE existing page in place (append a line); it never creates or
   deletes pages. NOTICING: if `pages/`-vs-`issues/` export means this scenario cannot run
   cleanly against real confluence, that is a real-run finding — the script must hard-FAIL
   safely (never mass-delete), and you SHOULD note the risk in the script header + flag it
   for the downstream owner-run item. This plan does NOT run the mutation (hermetic scope).
9. **Congruence-ready asserts_passed**: populate the mechanical artifact's `asserts_passed`
   with label strings that token-match the row's TWO `expected.asserts` (so the row is
   F-K4b congruence-ready when it later grades PASS — minted_at row). Use labels naming
   "two independent working trees against TokenWorld", "conflict reject", and "no fresh root
   on refetch".

Exit convention: 0 PASS / 1 FAIL / 75 NOT-VERIFIED. `chmod +x` the file.
Reference D-01..D-03: env-gate (per p93/attach-sync), sanctioned-target + mass-delete GUARD
(per milestone-close-vision-litmus), two-cache topology (per the sim-arm t4 script).
  </action>
  <verify>
    <automated>bash -n quality/gates/agent-ux/t4-conflict-rebase-ancestry-real-backend.sh && test -x quality/gates/agent-ux/t4-conflict-rebase-ancestry-real-backend.sh && env -u ATLASSIAN_API_KEY -u ATLASSIAN_EMAIL -u REPOSIX_CONFLUENCE_TENANT -u REPOSIX_ALLOWED_ORIGINS bash quality/gates/agent-ux/t4-conflict-rebase-ancestry-real-backend.sh; test $? -eq 75 && python3 -c "import json;d=json.load(open('quality/reports/verifications/agent-ux/t4-conflict-rebase-ancestry-real-backend.json'));assert d['status']=='NOT-VERIFIED' and d['exit_code']==75"</automated>
  </verify>
  <done>Script exists, executable, syntax-valid. No-creds run exits 75 BEFORE any cargo/init and writes a valid NOT-VERIFIED artifact. Body ports the two-cache conflict/ancestry scenario against confluence::TokenWorld with the pages/ bucket + mass-delete guard.</done>
</task>

<task type="auto" tdd="false">
  <name>Task 2: Author B5 — github-front-door-real-backend.sh (P1, shell-subprocess) + catalog fix-twice</name>
  <files>quality/gates/agent-ux/github-front-door-real-backend.sh, quality/catalogs/agent-ux.json</files>
  <action>
Author the B5 verifier using the p93/attach-sync template + `lib/transcript.sh`.
`kind: shell-subprocess` ⇒ the real-run body MUST emit its transcript via
`write_transcript_and_artifact "github-front-door-real-backend" <argv...>`.

Order (env-gate first for the hermetic property):

1. `source "${SCRIPT_DIR}/lib/transcript.sh"`; `SLUG="github-front-door-real-backend"`.
2. **ENV-GATE FIRST** (fail-closed, OD-2): require GITHUB_TOKEN non-empty AND
   REPOSIX_ALLOWED_ORIGINS non-empty AND containing `api.github.com` (grep). Any missing →
   WRITE a well-formed NOT-VERIFIED artifact to
   `quality/reports/verifications/agent-ux/github-front-door-real-backend.json` (fields:
   `ts`, `row_id`, `exit_code`: 75, `status`: "NOT-VERIFIED", `skip_reason`: "env-missing",
   `asserts_passed`: [], `asserts_failed`: [names]) — the env-gate exits BEFORE the transcript
   helper runs, so the script writes this artifact itself — then stderr + `exit 75`.
3. **Sanctioned-target guard** (OD-2 hard-FAIL, NOT 75): the sanctioned GitHub target is
   `reubenjohn/reposix` (docs/reference/testing-targets.md). Reject any other repo → `exit 1`.
4. **Build reposix once** — ONE cargo invocation: `cargo build -p reposix-cli --bin reposix`.
5. **Scenario (real-run body, wrapped by write_transcript_and_artifact)** in a `/tmp` clone,
   `cd`/`-C` in the SAME bash invocation (leaf isolation): drive
   `reposix init github::reubenjohn/reposix <tmp>` under creds → the GitHub connector issues
   `GET /repos/reubenjohn/reposix/issues` → HTTP 200 (NOT 404). ASSERT the request path
   carries the RAW slug `reubenjohn/reposix` (never the sanitized `reubenjohn-reposix`).
   ASSERT the issues bucket materializes in the partial-clone tree.
6. **Emit `ASSERT <label>: PASS|FAIL` lines to stdout** (transcript.sh parses these into
   asserts_passed). Emit ASSERT lines whose labels token-match ALL FOUR of the row's
   `expected.asserts` (HTTP 200 not 404 + issues bucket; raw slug not sanitized; transcript
   written to the STABLE path; creds-absent→NOT-VERIFIED) so the row is F-K4b
   congruence-ready when it later grades PASS (minted_at row).
   Exit convention: 0 PASS / 1 FAIL / 75 NOT-VERIFIED. `chmod +x`.

**FIX-TWICE (D-P96-01 stable-filename reality).** `transcript.sh` writes a STABLE filename
`quality/reports/transcripts/github-front-door-real-backend.txt` (no RFC3339 stamp — a
volatile name re-dirties the tracked verdict). In `quality/catalogs/agent-ux.json`, the
`agent-ux/github-front-door-real-backend` row's transcript assert (~L2449) still demands
`...github-front-door-real-backend-<RFC3339>.txt`. Correct it to the stable filename
`quality/reports/transcripts/github-front-door-real-backend.txt` (drop the `-<RFC3339>`
segment); keep the rest of the assert (`argv + env_keys (NAMES only) + cwd + exit_code +
stdout/stderr`). Change ONLY that assert string — do not touch the row's other asserts or
any other row. Preserve JSON validity.
  </action>
  <verify>
    <automated>bash -n quality/gates/agent-ux/github-front-door-real-backend.sh && test -x quality/gates/agent-ux/github-front-door-real-backend.sh && env -u GITHUB_TOKEN -u REPOSIX_ALLOWED_ORIGINS bash quality/gates/agent-ux/github-front-door-real-backend.sh; test $? -eq 75 && python3 -c "import json;d=json.load(open('quality/reports/verifications/agent-ux/github-front-door-real-backend.json'));assert d['status']=='NOT-VERIFIED' and d['exit_code']==75" && python3 -c "import json;json.load(open('quality/catalogs/agent-ux.json'))" && test "$(python3 -c "import json;r=[x for x in json.load(open('quality/catalogs/agent-ux.json'))['rows'] if x['id']=='agent-ux/github-front-door-real-backend'][0];print(any('RFC3339' in a for a in r['expected']['asserts']))")" = "False"</automated>
  </verify>
  <done>Script exists, executable, syntax-valid; no-creds run exits 75 BEFORE cargo/init and writes a valid NOT-VERIFIED artifact; sources lib/transcript.sh with the correct SLUG. Catalog assert corrected to the stable transcript filename; agent-ux.json is valid JSON with no RFC3339 requirement remaining on that row.</done>
</task>

<task type="auto">
  <name>Task 3: Hermetic self-test + shell-coverage harness + capture acceptance proof</name>
  <files>quality/gates/agent-ux/real-backend-env-gate.selftest.sh, quality/gates/code/shell-coverage-tests/real-backend-env-gate.sh</files>
  <action>
1. **Hermetic self-test** `quality/gates/agent-ux/real-backend-env-gate.selftest.sh` —
   mirror the `quality/gates/structure/file-size-limits.selftest.sh` idiom (standalone bash,
   `set -euo pipefail`, pass/fail counter, `exit 0` all-pass / `exit 1` regression, print a
   RESULT line). For EACH of the two new scripts, run it with ALL creds/allowlist unset:
   `env -u GITHUB_TOKEN -u ATLASSIAN_API_KEY -u ATLASSIAN_EMAIL -u REPOSIX_CONFLUENCE_TENANT
   -u REPOSIX_ALLOWED_ORIGINS bash <script>` and assert (a) exit code == 75, (b) the row's
   artifact JSON exists + parses + `status`=="NOT-VERIFIED" + `exit_code`==75. This path
   exits before any cargo/init ⇒ fully hermetic (no real backend, no cargo). `chmod +x`.

2. **shell-coverage harness** `quality/gates/code/shell-coverage-tests/real-backend-env-gate.sh` —
   FIRST `ls` + read one existing harness in that dir to match its exact naming/idiom, then
   author a harness that invokes BOTH new scripts' no-creds env-gate path (same `env -u ...`)
   so kcov credits the env-gate lines and the aggregate shell-coverage does not regress. The
   harness must return 0 regardless of the scripts' `exit 75` (tolerate it with `|| true`).

3. **Confirm no shell-coverage regression**: run `bash quality/gates/code/shell-coverage.sh`
   (needs kcov). Confirm aggregate >= floor (quality/shell-coverage-floor.txt == 13). If it
   DIPS below floor, do NOT lower the floor to force-pass (silent-downgrade prohibited) —
   file the residual honestly under `.planning/quick/260712-phc-.../` notes / SURPRISES per
   OP-8 and surface it in the SUMMARY. (If kcov is not installed in this environment, record
   that the harness is authored + the coverage check is deferred to the pre-push run, and
   note it in the SUMMARY — do NOT skip authoring the harness.)

4. **Capture the ACCEPTANCE PROOF** (validate-only, NO `--persist`, NO real creds):
   `env -u GITHUB_TOKEN -u ATLASSIAN_API_KEY -u REPOSIX_ALLOWED_ORIGINS python3
   quality/runners/run.py --cadence pre-release-real-backend 2>&1 | tee
   .planning/quick/260712-phc-author-two-missing-pre-release-real-back/accept-proof.txt`.
   Confirm the output shows BOTH rows as NOT-VERIFIED/env-missing and NO LONGER prints
   "verifier not found at quality/gates/agent-ux/t4-conflict-rebase-ancestry-real-backend.sh"
   or "...github-front-door-real-backend.sh". This run may exit non-zero (P0 NOT-VERIFIED
   rows) — that is expected; you are capturing OUTPUT, not exit 0. MUST NOT run with real
   creds; MUST NOT pass `--persist` (a validate-only run does not mutate the catalog).
  </action>
  <verify>
    <automated>bash quality/gates/agent-ux/real-backend-env-gate.selftest.sh && test "$(env -u GITHUB_TOKEN -u ATLASSIAN_API_KEY -u REPOSIX_ALLOWED_ORIGINS python3 quality/runners/run.py --cadence pre-release-real-backend 2>&1 | grep -cF 'verifier not found at quality/gates/agent-ux/t4-conflict-rebase-ancestry-real-backend.sh')" -eq 0 && test "$(env -u GITHUB_TOKEN -u ATLASSIAN_API_KEY -u REPOSIX_ALLOWED_ORIGINS python3 quality/runners/run.py --cadence pre-release-real-backend 2>&1 | grep -cF 'verifier not found at quality/gates/agent-ux/github-front-door-real-backend.sh')" -eq 0</automated>
  </verify>
  <done>Self-test tracked + passing (both scripts exit 75 + valid NOT-VERIFIED artifact). Coverage harness authored; shell-coverage aggregate >= floor (or the residual honestly surfaced, never floor-lowered). Acceptance-proof captured showing both rows NOT-VERIFIED/env-missing, not "verifier not found".</done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| verifier script → real backend (GitHub / Confluence) | Only crossed when creds ARE set (downstream owner-run item, NOT this plan). This plan proves only the fail-closed env-gate that keeps the boundary shut when creds are unset. |
| remote bytes → transcript / stdout | Real-run bytes (issue bodies, page content) are tainted; must not leak secrets. |
| script → real TokenWorld mutation | B4 edits a real page; a delete-shaped diff could mass-delete the space. |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-phc-01 | Information disclosure | transcript.sh env_keys | mitigate | `env_keys` emits variable NAMES only (`cut -d= -f1`) — never values. B5 reuses this helper verbatim; do not inline a value-printing variant. |
| T-phc-02 | Tampering / Denial | B4 mutating real TokenWorld | mitigate | Sanctioned-target guard (TokenWorld + tenant `reuben-john` only, else exit 1) + mass-delete GUARD (refuse delete-shaped diff, protected-fixture denylist ` 7766017 7798785 `) mirrored from milestone-close-vision-litmus. Scenario edits ONE page in place; never creates/deletes. |
| T-phc-03 | Elevation / egress | scripts making outbound HTTP | mitigate | Env-gate REQUIRES non-empty `REPOSIX_ALLOWED_ORIGINS` (B5 additionally requires it to contain `api.github.com`); no creds/allowlist ⇒ exit 75 before any network call. Fail-closed per OD-2. |
| T-phc-04 | Spoofing (fabricated PASS) | env-gated skip | mitigate | Exit 75 → runner maps to NOT-VERIFIED (never skip-as-pass); creds-missing at milestone-close is RED at the verdict layer, never a silent green. |
</threat_model>

<verification>
Maps to the 5 must-have acceptance items from the locked spec:

1. Both scripts exist, executable (`test -x`), git-tracked (`git ls-files` shows them).
2. Each standalone with NO creds exits 75 + writes a clean NOT-VERIFIED artifact (Task 1 & 2 automated verify).
3. `env -u GITHUB_TOKEN -u ATLASSIAN_API_KEY -u REPOSIX_ALLOWED_ORIGINS python3 quality/runners/run.py --cadence pre-release-real-backend` (validate-only) shows BOTH rows NOT-VERIFIED/env-missing, NOT "verifier not found" (Task 3 automated verify).
4. B5 catalog transcript assert corrected to the stable filename; agent-ux.json still valid JSON (Task 2 automated verify).
5. Hermetic self-test git-tracked + passing; pre-push shell-coverage aggregate >= floor (13) or residual honestly surfaced (Task 3).

Hermetic invariant: the whole plan runs with NO real creds and NO `--persist`. No task
crosses the real-backend trust boundary.
</verification>

<success_criteria>
- `quality/gates/agent-ux/t4-conflict-rebase-ancestry-real-backend.sh` and
  `quality/gates/agent-ux/github-front-door-real-backend.sh` exist, are `chmod +x`, and are
  git-tracked.
- Each, run standalone with creds/allowlist unset, exits 75 and writes a well-formed
  NOT-VERIFIED artifact at its catalog artifact path.
- The `pre-release-real-backend` validate-only run grades both rows NOT-VERIFIED/env-missing
  — the string "verifier not found" no longer appears for either script.
- The B5 catalog row's transcript assert names the stable filename (no `-<RFC3339>`) and
  `quality/catalogs/agent-ux.json` parses as valid JSON.
- `quality/gates/agent-ux/real-backend-env-gate.selftest.sh` is tracked and exits 0.
- Pre-push shell-coverage aggregate stays >= 13 (or the residual is honestly surfaced, floor
  never lowered).
</success_criteria>

<output>
After completion, create
`.planning/quick/260712-phc-author-two-missing-pre-release-real-back/260712-phc-SUMMARY.md`.
Record: the two scripts authored (with the env-gate/sanctioned-target/mass-delete-guard
decisions), the catalog fix-twice, the acceptance-proof output snippet (both rows now
NOT-VERIFIED/env-missing), the shell-coverage result, and — per the ownership charter's
noticing mandate — any confluence `pages/`-vs-`issues/` export risk you noticed for the
downstream owner-run real grading of B4.
</output>
