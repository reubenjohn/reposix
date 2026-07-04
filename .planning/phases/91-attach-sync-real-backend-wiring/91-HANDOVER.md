# 91-HANDOVER.md — P91 coordinator relief handover

Written 2026-07-04 by the outgoing P91 coordinator (relieved at the Wave-5.5/Wave-6
boundary, context capacity). Successor: read this + `91-OVERVIEW.md` + `91-DECISIONS.md`
first; per-wave detail lives in `91-0N-SUMMARY.md` files. Ground truth at handover:
HEAD `6d4fa7a`, tree CLEAN, origin/main still `32ba856` (NOTHING pushed yet — the
entire phase is local). Preflight was 3/3 PASS. Never touch PR #61. Do NOT start P92.

## 1. Wave state

| Wave | Plan | State | Commits |
|---|---|---|---|
| plan | plans+research+decisions+plan-check (2 MUST-FIX amended) | DONE | `1a64ab0`, `a7b6939` |
| 1 | 91-01 catalog-first mint (2 rows NOT-VERIFIED + ql-001 verifier skeleton) | DONE | `710211d` |
| 2 | 91-02 QL-001 fix (canonical path, peek-LF parser, issues/*.md filter, fixture re-key, refresh D91-10, waiver retire) | DONE | `1c03da0`, `4bebfa3`, `3bc10bf`, `c9e2b8f`, `a597152` |
| 3 | 91-03 attach/sync real dispatch ([lib] split, ForkAsNew=FREE, OP-3 kept, token scrub, real smokes, ci.yml JIRA_TEST_PROJECT) | DONE | `1f7fff3`, `cf7a37d`, `9af1d69`, `3f67c0e`, `bd5e115`, `e080117` |
| 4 | 91-04 dvcs-third-arm populated (counts: matched=3 no_id=1 backend_deleted=1 mirror_lag=2) + self-seeding hierarchy test + testing-targets protected-fixture doc | DONE | `1504425`, `587aee4`, `6ca3f6d`, `ac0fcdf` |
| 5 | 91-05 litmus rewrite (real 8-step D91-06 body + lib/litmus-flow.sh + GUARD B) + mirror repopulation | DONE | `5786784`, `85c8576` |
| 5.5 | REOPEN fix: bucket-aware canonical paths (unplanned — see §3) | DONE (see §1a) | `3090499`, `d6e1411`, `e2f8acd`, `6d4fa7a` |
| 6 | 91-06 docs + close prep (REQUIREMENTS DVCS-ATTACH-01..04 flip, CLAUDE.md attach example, architecture-sketch overstatement, comments/attachments doc check + intake→ROUTED-P95, swarm→P95) | **PENDING** | — |
| close | push + CI + litmus REOPEN loop + verifier + STATE (see §6) | **PENDING** | — |

### 1a. Wave-5.5 process incident (read before dispatching ANY executor)

Wave-5.5's first executor was a SendMessage-resume of the Wave-2 opus executor. It was
wrongly declared dead (orchestrator relay based on quiet disk); a fresh replacement
executor was dispatched, DETECTED the live sibling (PID 54111, `claude
--dangerously-skip-permissions`, cwd = this repo) mid-write, committed the core layer
(`3090499`, adopting the sibling's path.rs work) and halted per shared-state rules. The
sibling then finished the lane itself: `d6e1411` (bucket-aware `record_path(bucket,id)` +
`record_id_from_path` accepting {issues,pages} + id-keyed diff planner), `e2f8acd`
(credential-leak fix: strips embedded creds from mirror URLs in git config + helper
stderr — the Wave-5 MEDIUM intake), `6d4fa7a` (litmus GUARD B pages/-aware, ql-001
verifier bucket asserts, intake flips). At handover PID 54111 was STILL ALIVE with a
clean tree — verify it has exited (`ps -p 54111`) before dispatching any tree-writer;
a background watcher (`bs1fhsr7q`, this session) dies with the outgoing session — re-arm
your own if needed. Lesson (also in the halting executor's report): before granting any
executor "only tree-writer" status, check for live sibling `claude` processes sharing
the cwd.

**Wave-5.5 CONFIRMED COMPLETE** (report reached the orchestrator post-relief; recorded
here verbatim-in-substance): `3090499` core bucket layer (RECORD_BUCKETS {issues,pages},
`bucket_for_backend`, `record_path`/`record_id_from_path`); `d6e1411` id-keyed planner +
site wiring + regressions — planner matches prior↔tree by RECORD ID, mass-delete
signature reproduced RED pre-fix then fixed, duplicate-id refuses loud; `e2f8acd`
credential-leak fix (`strip_url_userinfo`, attach strips mirror-URL userinfo + teaching
warning, 4 helper stderr sites redacted); `6d4fa7a` GUARD B asserts pages/ substrate,
ql-001 verifier 16 asserts PASS, intakes RESOLVED, stale 91-05 handoff instruction
corrected. All suites green per-crate, clippy/fmt clean, GUARD A + SG-02 intact, litmus
env-unset still exits 75. Deliberate deviation from the intake's sketched resolution is
recorded inside the RESOLVED entry. Successor needs only a spot-check
(`git show --stat d6e1411 e2f8acd 6d4fa7a`), not a full gap-analysis.

## 2. Litmus / REOPEN state

- Mechanical litmus (`quality/gates/agent-ux/milestone-close-vision-litmus.sh`, rewritten
  in Wave 5, no longer a stub): run #1 env-unset → exit 75 (honest). Run #2 real vs
  TokenWorld → exit 1, honest FAIL at GUARD B (6/7 boxes) = the mass-delete BLOCKER
  (§3). Transcript in `quality/reports/transcripts/`. NOT yet re-run after the Wave-5.5
  fix — that re-run (expect exit 0) is the successor's first litmus step.
- Fresh-agent T2 friction run (D91-07 layer b): NOT YET RUN. REOPEN gate = mechanical
  litmus exit 0 AND fresh dark-factory-style subprocess agent completes T2-attach flow
  vs TokenWorld with 0 HIGH frictions (HIGH = documented happy path disagrees with
  binary). Loop fix→re-run until 0 HIGH. Friction counts per run are final-report data.
- Mirror substrate: `reubenjohn/reposix-tokenworld-mirror` repopulated by Wave 5 →
  `2a66596` (3 real records `pages/{2818063,7766017,7798785}.md`; pre-population base
  `09dda47`). **CRITICAL: pages/ STAYS canonical for Confluence — do NOT repopulate the
  mirror to issues/** (Wave-5.5 corrected the stale 91-05 instruction that said
  otherwise; the bucket fix made the planner pages/-aware instead).
- Real-backend etiquette: sanctioned = Confluence TokenWorld (space key REPOSIX, id
  360450 — NOTE "TokenWorld" and "REPOSIX" are the SAME space, doc ambiguity noticed),
  GitHub reubenjohn/reposix issues, JIRA KAN. kind=test label + sweep; NEVER touch
  Confluence pages 7766017/7798785 (now documented in testing-targets.md, Wave 4).

## 3. Confluence-push-mass-delete BLOCKER (disposition: FIXED IN-PHASE)

Wave-5's real litmus run found: Confluence trees use bucket `pages/<id>.md` but Wave-2's
canonicalization hardcoded `issues/` → a real Confluence push classified every record as
prior-Delete + tree-Create (mass-delete). During Wave-5 dev it actually deleted the
protected fixtures + Home page (executor fully restored all three, verified). Intake
BLOCKER entry filed 2026-07-04 (bottom of SURPRISES-INTAKE.md); disposition = fixed
in-phase (QL-001 bug class, adoption-critical) via `d6e1411` + GUARD B retained as
defense-in-depth; `6d4fa7a` flipped the intake. Successor: confirm the intake STATUS
reads RESOLVED with those SHAs and that the litmus re-run proves it against reality.

## 4. Mid-execution decisions not yet in 91-DECISIONS.md

- **D91-13 (de facto):** canonical path layer is bucket-aware — `record_path(bucket,id)`,
  sanctioned buckets {issues, pages}, `bucket_for_backend` mapping; extends D91-01
  (which said `issues/<id>.md` universally — WRONG for Confluence). Consider appending
  to 91-DECISIONS.md for the record.
- Sanctioned-space set widened to {TokenWorld, REPOSIX} in the attach-sync verifier
  (Wave-3 deviation, journaled in 91-03-SUMMARY; both = the one owner-owned space).
- Mass-delete BLOCKER absorbed in-phase (§3 rationale).
- Mirror repopulation is coordinator-owned litmus prep (executors were forbidden from
  touching the mirror/Confluence post-incident).
- Wave-2 runner side-effect handling: `run.py` cadence runs MUTATE catalogs in place;
  Waves 1/2 reverted unrelated row flips (pre-existing p87/p88 absorption-row FAILs)
  to keep commits scoped — those 4 on-demand FAIL rows are P96/P97's absorption work,
  not P91 breakage.

## 5. Noticed, not yet (or nonstandardly) filed

- `deferred-items.md` in this phase dir was used by Waves 2/4/5 for some findings
  (nonstandard vs SURPRISES-INTAKE/GOOD-TO-HAVES) — reconcile/route at Wave 6:
  p87/p88 rows' `claim_vs_assertion_audit` gap (hard-blocks catalog load when their
  verifiers re-run — time bomb), pre-existing contract.rs + .py file-size overages.
- `run.py` in-place catalog mutation with no `--dry-run` guard (PROTOCOL.md callout
  candidate; noticed by Waves 1+2).
- Helper internals still carry P79/P82 tokens (main/bus_handler/precheck) — intake
  filed → P97. Bare `P\d+\+` shape uncaught by banned-token gate — intake filed → P97.
- `crlf_blob_body_round_trips` masks BUG-1 (empty prior) — Wave-2 noticed; check
  whether the re-key wave already fixed it before re-filing.
- Stray `unknown command: feature` on helper stdin path; mirror push leg needs
  `github.com` (not just `api.github.com`) in REPOSIX_ALLOWED_ORIGINS guidance — LOW
  intakes filed by Wave 5.
- TokenWorld==REPOSIX one-space doc ambiguity in testing-targets.md (fold into Wave 6
  docs pass if cheap).
- Frontmatter-id vs path-id mismatch is unvalidated in `plan()` (pre-existing; noted by
  Wave-5.5, NOT filed — attach's duplicate-id abort covers the practical case; file it
  if Wave 6 finds a user-reachable path).

## 6. Precise next steps (successor's runbook)

1. Confirm PID 54111 exited + tree clean; gap-analyze Wave-5.5 (§1a checklist).
2. If mirror content predates the bucket fix, refresh it (coordinator-owned), then
   re-run the mechanical litmus vs TokenWorld → expect exit 0 + transcript + dual-table
   audit rows.
3. Dispatch Wave 6 executor (91-06-PLAN.md, sonnet): docs flips (REQUIREMENTS
   DVCS-ATTACH-01..04, CLAUDE.md attach example — same-PR rule, architecture-sketch
   overstatement, comments/attachments doc check + intake→ROUTED-P95, swarm→P95
   routing), mkdocs-strict + docs gates, reconcile `deferred-items.md` (§5).
4. Close ritual: full pre-push sweep green (`python3 quality/runners/run.py --cadence
   pre-push` via the hook), `git push origin main`, `gh run watch --exit-status` green —
   and VERIFY in the CI log that the pre-pr quality job actually executes
   `real-git-push-e2e` and it PASSES (D91-02: CI is the full-stack QL-001 proof; this
   box's git 2.25.1 can only reach exit-75 locally). **QL-001 waiver deadline
   2026-07-31** — waiver retirement already landed (`c9e2b8f`); the deadline pressure is
   CI-green-before-then.
5. Litmus REOPEN loop (D91-07 layer b): fresh dark-factory T2 subprocess agent vs
   TokenWorld; 0 HIGH frictions required; loop fix→re-run. Friction counts per run →
   final report.
6. Unbiased opus verifier per `quality/PROTOCOL.md` § "Verifier subagent prompt
   template" (zero session context, grades ROADMAP SC-1..9 + QL-001 criteria 1-6 +
   P91 catalog rows from artifacts) → verdict at `quality/reports/verdicts/p91/VERDICT.md`.
   RED ⇒ loop back, never waive (OD-2).
7. STATE.md advance (phases_completed 13→14, next_phase P92) + ROADMAP P91 checkboxes +
   final coordinator report per the original P91 brief (verdict+path, commits, QL-001
   evidence incl. real init→edit→push proof, litmus friction counts + REOPEN loops,
   real-backend coverage delta — which rows went honestly green, intake drained/filed,
   deviations, NOTICED, P92 handoff, SCOPE DECISION NEEDED: none outstanding — the
   comments/attachments call was decided D91-05/ROUTED-P95).
