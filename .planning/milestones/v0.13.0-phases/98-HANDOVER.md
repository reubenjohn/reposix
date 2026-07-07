# 98-HANDOVER.md — v0.13.0 release CI-red chase (C1 relief), 2026-07-07

Written by the outgoing C1 phase-coordinator (dispatched by L0 to clear CI reds on the
v0.13.0 release PR before L0 merges + publishes to crates.io). **This is a RELIEF
handover, not a phase close** — no verdict is being claimed; the successor is a fresh
coordinator identity picking up an in-flight, moving CI target. Context accumulated
chasing a red across 2 PR numbers and one genuinely hard-to-diagnose test failure;
relieving at a clean, fully-pushed boundary rather than past the ~100k line.

**Read order:** this file in full, then
`.planning/milestones/v0.13.0-phases/SURPRISES-INTAKE.md` entry `S-260707-rbf-01`
(the crlf-test investigation log, more detailed than §5 below), then
`.planning/STATE.md` § Workstream A for the broader release-runbook framing.

**Verdict for L0: NO-GO.** Do not merge/publish/tag v0.13.0 yet. One release-blocking
CI red (§4/§5) is still unexplained; its root cause has not been read from a CI log
because of a truncation bug that was only just fixed and not yet proven against a live
run.

## 1. Ground truth (git)

HEAD = `12334ac7823dc9e7e44c5ab4a541752bb143f94b` on `main`, `origin/main` matches
(pushed). Working tree has ONE modified-but-uncommitted file:
`quality/catalogs/doc-alignment.json` — a 1-line timestamp-only diff (`last_walked`
bumped by the pre-push walker hook; harmless, recurs on every pre-push run, already a
known GOOD-TO-HAVES-adjacent brittle-gate issue — see §5). **Leave it uncommitted or
let the next pre-push absorb it; do not hand-edit it.**

Commits made this session, all on `main`, all pushed (chronological):

| Commit | Summary |
|---|---|
| `f1e0e9b` | chore(quality): refresh doc-alignment catalog — walker hash re-extraction drift |
| `6959251` | chore(planning): file codecov upload-drop fragility to GOOD-TO-HAVES (MEDIUM) |
| `51ef6a8` | chore(planning): file recurring TokenWorld fixture trash-drift to GOOD-TO-HAVES (LOW-MEDIUM) |
| `aa2b33d` | fix(ci): stop swallowing p94 cargo-test failure diagnostics; harden CRLF-test timeouts |
| `fbe5bee` | fix(quality): stop dump_verifications truncating past the real panic message |
| `12334ac` | docs(planning): S-260707-rbf-01 — timeout fix proven ineffective, HIGH severity |

Plus a merge commit `deee8fd8a5925bf1f47c54cb657db20892c1857a` landed directly on the
old PR #61 branch (`release-plz-2026-07-04T05-03-11Z`) — now moot, that PR is closed
(superseded, see §2).

No `v0.13.0` tag exists (`git tag -l 'v0.13*'` — empty, verified live). No irreversible
action was taken this session.

**Deviation the successor must know:** none of this session's git history diverges
from the plan in a way that needs reverting — it is a straight-line sequence of
fixes/filings. The only open loose end is the still-unexplained test failure (§5).

## 2. Wave/cycle state

| Wave | Scope | State | Commits |
|---|---|---|---|
| 1 — doc-alignment catalog drift | refresh stale walker fields | DONE (recurs every pre-push, see §5) | `f1e0e9b` |
| 2 — Confluence TokenWorld fixture repair | restore `parentId` on page `7798785` | DONE (live repair, not a commit — see §5 item 2) | `51ef6a8` (filing only) |
| 3 — codecov/project phantom red | root-cause + file | DONE (explained, not a real regression; do NOT lower thresholds) | `6959251` |
| 4 — CI log truncation bug | fix `dump_verifications.py` tail window | DONE (fix landed, NOT yet verified against a live CI run) | `fbe5bee` |
| 5 — crlf test timeout theory | bump 15s→30s, re-test | DONE, and DISPROVEN — not a timeout | `aa2b33d`, `12334ac` |
| 6 — crlf test real root cause | read the actual panic text, fix it | **NOT STARTED / BLOCKED** — this is the live blocker | none |
| 7 — PR #61 → #68 supersession | release-plz auto-closed #61, opened #68 | happened mid-session, outside this coordinator's control | n/a |

No named-incident post-mortem beyond the SURPRISES-INTAKE entry referenced above —
read that entry before dispatching anyone at the crlf test.

## 3. Binding constraints (unchanged)

- One tree-writer at a time; **one cargo invocation machine-wide** (this repo's VM has
  OOM-crashed on parallel builds).
- No `--no-verify`, ever.
- Push only at green (per-phase push cadence in `.planning/CLAUDE.md`); this session's
  commits all pushed to `main` directly (they were doc/quality-config filings, not a
  phase deliverable — consistent with the debt-drain / OP-8 filing pattern, not a phase
  push).
- Commit-trailer format: `Co-Authored-By` + Claude-Session trailers, no exceptions.
- Model tiering: fable → opus/sonnet/haiku per `.planning/ORCHESTRATION.md` §1; this
  coordinator ran as a sonnet-tier `phase-coordinator` (per its dispatch).
- **Do NOT lower any codecov threshold** to route around item 3 above — the fix is
  upload robustness, not a gate relaxation (see GOOD-TO-HAVES entry, §5).
- **Do NOT run `tag-v0.13.0.sh.disabled`** — the tag push is L0/owner's, gated on GREEN
  CI on the live release PR, which does not exist yet.
- External/security-sensitive mutations (PR close/reopen, live Confluence page edits)
  were performed this session under the owner's prior release-decision delegation
  (`.planning/STATE.md` § Workstream A, "RELEASE DECISION DELEGATED TO L0") — the
  successor inherits that same delegation scope, not a fresh grant. If the successor is
  a different coordinator identity than the one holding that delegation, confirm with
  L0 before repeating external mutations at the next level up (E1/E3 discipline
  unchanged).

## 4. Litmus / gate / REOPEN state

- **Gate:** `quality/gates/agent-ux/p94-git243-fallback-sentinel.sh` (catalog row
  `agent-ux/p94-git243-fallback-sentinel`), exercised inside the `quality gates
  (pre-pr)` CI job.
- **Run history (on the now-closed PR #61, both against the same test):**
  - CI run `28819166220` — RED, 2 failures (`quality gates (pre-pr)` +
    `codecov/project`; the latter explained as phantom, §5 item 3).
  - CI run `28837407948` (job `85523987205`), head `deee8fd` — RED again on `quality
    gates (pre-pr)` AFTER the 15s→30s timeout bump (`aa2b33d`) landed, proving the
    timeout theory dead (`timed_out: False`, finished in 0.14s).
- **REOPEN state:** OPEN, HIGH severity (raised from MEDIUM at `12334ac`). Per OD-3,
  "litmus REOPEN gates remain in force UNCHANGED — on RED the orchestrator loops back,
  never waives." No waiver has been requested or granted for this gate.
- **No waiver-expiry clocks** are running against this specific item.
- **CodeQL only, on PR #68 so far** (`Analyze (actions/python/rust)`, `CodeQL` — all
  PASS, checked live this session). `CI` and `Security audit` on PR #68 are stuck at
  `action_required` (see §6 step 1) — the `quality gates (pre-pr)` job has **not yet
  run** on PR #68 at all, so there is no fresh evidence yet either way for this PR.

## 5. Mid-execution decisions not yet formalized + "noticed, not yet filed"

All of the below are formalized in commits/intake already; listed here so the
successor doesn't have to re-derive them from scratch.

1. **Doc-alignment.json will drift again on the very next pre-push.** The walker
   rewrites `last_walked` (and sometimes `coverage_ratio`/`total_eligible_lines`) even
   in validate-only mode. Confirmed live again this session (current dirty diff is a
   1-line timestamp bump). This is a known, already-flagged recurring brittle-gate
   issue — no new filing needed, just don't be surprised by it.

2. **Confluence TokenWorld fixture `7798785`** had its parent link to `7766017`
   repaired live via the Confluence REST API this session (no code change, no commit —
   an external-system mutation under the release-decision delegation). Verified
   `parentId == 7766017` post-repair. **Root cause of the repeat-trashing (2+
   occurrences) is still unknown** — filed to GOOD-TO-HAVES (`51ef6a8`, LOW-MEDIUM,
   non-blocking for v0.13.0). If it happens a third time before the tag ships, that's a
   stronger signal something structural (a test teardown, a stale script, an owner
   habit in the TokenWorld space) is mistargeting this page — worth a dedicated
   investigation lane at that point rather than another silent repair.

3. **codecov/project phantom -16% is explained, not fixed.** Root cause: `ci.yml`'s
   coverage job uploads Rust `lcov.info` with `fail_ci_if_error: false`; a silently
   dropped upload leaves BASE (Rust+shell) compared against HEAD (shell-only),
   manufacturing a fake regression. Confirmed `codecov/patch` was green throughout —
   this was never a real coverage regression. Filed to GOOD-TO-HAVES (`6959251`,
   MEDIUM) recommending upload robustness (retry / explicit flags / `after_n_builds`).
   **Do not treat a recurrence of this specific symptom shape (project red + patch
   green + upload-count mismatch in the codecov banner) as a real blocker** — diagnose
   it the same way before escalating.

4. **`action_required` CI-gap recurs on every release-plz bot push, by design of
   GitHub's `pull_request` trigger security model** (bot-authored `GITHUB_TOKEN`
   pushes don't fire `pull_request`-triggered workflows). Fix each time: `gh pr close
   <n>` then `gh pr reopen <n>` by a real actor. Needed 2-3 times this session on PR
   #61 as release-plz kept regenerating it in response to this session's own pushes to
   `main`. **Currently needed again on PR #68** (see §6 step 1) — this is not a new
   finding, just confirming the pattern held true a fourth time.

5. **PR #61 → #68 supersession was NOT this coordinator's action.** Release-plz
   auto-closed PR #61 (github-actions[bot], not merged) and opened #68 partway through
   the session, triggered by this session's own pushes to `main` (which changed crate
   versions/changelogs needing regen). Confirmed live: PR #61 is `state: CLOSED`
   (head `deee8fd`, still `mergeable: MERGEABLE` — moot). **PR #68 is the live release
   target** (`state: OPEN`, head `a38caeb943d41ef78252810ff23d77b5a18ca1a6`,
   `mergeable: MERGEABLE`, `mergeStateStatus: UNSTABLE` because required checks
   haven't run yet). **Expect this to happen again** if `main` gets more commits before
   release-plz settles — don't be alarmed by a PR number changing underneath the
   release runbook.

6. **Confirmed live this session: PR #68's branch does NOT yet contain the log-fix
   commit `fbe5bee`.** `git merge-base main origin/release-plz-2026-07-07T02-37-20Z` =
   `aa2b33d` — i.e. PR #68 was cut from `main` at `aa2b33d`, one commit before
   `fbe5bee` landed. **The log-truncation fix is NOT live on PR #68 yet.** Until a
   fresh `origin/main` merge/rebase lands `fbe5bee` (and `12334ac`) onto PR #68's
   branch, a `quality gates (pre-pr)` run on #68 will STILL truncate the panic text
   even if the crlf test fails again. This is the single most important unresolved
   fact for the successor — do not waste a CI run without checking this first.

**Nothing else noticed-not-filed this session** beyond what's already in
GOOD-TO-HAVES/SURPRISES-INTAKE above.

## 6. Precise next steps (successor runbook)

1. **Unblock PR #68's stuck CI.** `CI` and `Security audit` on PR #68 are at
   `action_required` (confirmed live: `gh run list --branch
   release-plz-2026-07-07T02-37-20Z` shows both `conclusion: action_required`). Before
   doing anything else, check whether PR #68's branch has been superseded again (`gh pr
   view 68 --json state,headRefOid`) — if release-plz regenerated it since this
   handover was written, re-verify head SHA and the merge-base-vs-`fbe5bee` check in
   step 2 against the NEW head.
2. **Land `fbe5bee` (and `12334ac`) onto whatever PR-#68-equivalent branch is live**,
   if not already present: `git fetch origin && git checkout
   release-plz-<branch> && git merge origin/main && git push` (same pattern as this
   session's `deee8fd` merge onto the old PR #61 branch). Confirm with `git log
   --oneline -3` that `fbe5bee` is now an ancestor of the branch tip before proceeding.
3. **Real-actor close/reopen** the PR (`gh pr close <n>` then `gh pr reopen <n>`) to
   force the `pull_request`-triggered `CI` + `Security audit` workflows to actually
   run (see §5 item 4). Wait for the run to complete (`gh run watch <run-id>` or poll
   `gh pr checks <n>`).
4. **Read the `quality gates (pre-pr)` job log directly** (`gh run view <id> --log` or
   the job's own artifact/log page) — specifically look for `dump_verifications.py`'s
   now-uncapped (200-line) tail output around the `p94-git243-fallback-sentinel`
   verifier. The `grep -B2 -A15 'panicked at'` context block should now survive into
   the printed log. Extract the exact assertion/panic text.
5. **Diagnose from the real text:**
   - If it names `tests/protocol.rs:219` (the CRLF-preservation assert) — inspect the
     actual `body=...` diff printed; compare against local values for the same input,
     looking specifically at `serde_json` / `wiremock` / `http` crate version
     resolution differences between CI's `Swatinem/rust-cache`-restored lockfile build
     and a fresh local build (leading hypothesis (i) in the SURPRISES-INTAKE entry).
   - If it names a different assertion (e.g. the `stdout.contains("ok
     refs/heads/main")` check around line 203-206) — that reframes the whole
     investigation; update SURPRISES-INTAKE `S-260707-rbf-01` with the corrected
     target before further work.
   - Either way, update the SURPRISES-INTAKE entry with the finding BEFORE attempting a
     fix (per DP-2 prove-before-fix on BLOCKERs, `.planning/ORCHESTRATION.md` §11).
6. **Fix the real root cause** (or, if it proves to be a CI-environment-only artifact
   with no code path to fix, document that explicitly and bring it to L0 as an E2/E4
   escalation candidate — two failed self-attempts at the same gate is the standing
   trigger for a fable consult, per ORCHESTRATION §11).
7. **Re-run `quality gates (pre-pr)`** on the branch after the fix lands; confirm GREEN
   before calling this gate closed. Update the SURPRISES-INTAKE entry's STATUS from
   OPEN to CLOSED with the resolving commit.
8. **Once `quality gates (pre-pr)` + all other required checks are GREEN** on the live
   release PR: re-verify the diff is still release-churn-only (version bumps +
   changelogs, no stray source — same steward-review standard applied to PR #61
   earlier in the runbook), then report GO back to L0. **Do not merge, publish to
   crates.io, or cut the `v0.13.0` tag yourself** — those remain L0/owner-gated
   irreversible actions per the standing release-decision delegation
   (`.planning/STATE.md` § Workstream A).
9. **Update `.planning/STATE.md`'s release-runbook status block** with the new PR
   number, CI run ID, and GO/NO-GO verdict once step 8 resolves, mirroring the pattern
   already used for the #61 → run `28819166220` NO-GO entry (commit `c5fd461`).
