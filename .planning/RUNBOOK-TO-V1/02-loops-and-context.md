# RUNBOOK ch.02 — loops, context budgets, rot prevention, reminders

Every recurring motion in the drive is one of five named loops. Each loop has an
entry condition, an exit condition, a corrective action on failure, and a named
tier/agent for every step. Honest corrective iteration (index Amendment 3) is
built into each: no loop exits on the author's word.

## §A — The five loops

### A-L1 — Per-phase loop (Operating Cadence A)

Owner of the loop: the L1 portion coordinator (L0 runs it directly only between
portions).

- **Entry:** prior phase verdict GREEN + pushed; zero undrained BLOCKER rows in
  `.planning/audits/QUALITY-LEDGER.md`; STATE.md cursor names the phase; steward
  window (A-L5) run since the last close.
- **Steps:**
  1. **Recon + charter** (L4 `Explore`/`reader-digester` → L1): digest the
     phase's ROADMAP section + intake entries; apply the 10x rule; pre-authorize
     splits; write the charter via `coordinator-dispatch` skill.
  2. **Dispatch L2 phase coordinator** (`phase-coordinator`, opus for
     security-judgment phases, sonnet otherwise). Phases marked `Execution mode:
     top-level` in ROADMAP (P96, P97, parts of P93/P94/P95) are still delegated —
     "top-level" means *not inside `gsd-execute-phase`*; the L2 coordinator runs
     the fan-out itself.
  3. **L2 runs waves** via L3 lanes; L3 makes case-by-case L4 calls (recon
     before edit, reviewer before report — HCI). One tree-writer at a time; ONE
     cargo invocation machine-wide. **Bottom-up triage:** every returning lane's
     NOTICED / RAISE LIST gets triaged on receipt — absorb into this wave (low
     charter-deviation + 10x capacity), re-delegate as a new lane, or file to
     intake; never drop it (ORCHESTRATION §2).
  4. **Phase close:** L2 ensures `git push origin main` lands BEFORE step 5
     (verifier grades RED without it), CI green (`gh run` — one watch, read
     failures, never declare green unseen).
  5. **Verifier dispatch** (fresh subagent, zero session context, PROTOCOL.md
     template; grades catalog rows from artifacts). Verdict committed under
     `quality/reports/verdicts/p<N>/VERDICT.md`.
  6. **Debt-drain window** (A-L3 drain half) + steward checks (A-L5).
  7. **Advance:** STATE.md cursor, ≤400-word report up-tier.
- **Exit:** verdict GREEN, pushed, CI green, drain clean, cursor advanced.
- **Corrective:** verdict RED → loop to step 3 with the verifier's findings as
  the new wave charter (fix lanes get the RED rows verbatim). Two RED cycles on
  the same row → valve E4. Coordinator rot at any point → DP-1 rotation (the
  loop survives rotation via the committed handover).

### A-L2 — Litmus REOPEN loop

- **Entry:** the phase's litmus gate runs at close (P92: T1+T4 sim+TokenWorld;
  P93: T1+T4; P94: T3 vs TokenWorld + GH mirror). Runner: an L3/L4 gate lane
  producing a committed transcript.
- **Exit:** zero HIGH findings on a full run.
- **Corrective:** ≥1 HIGH → phase REOPENS (never waive — OD-2 spirit: "on RED
  the orchestrator loops back, never waives"). Each REOPEN = targeted fix lane →
  FULL litmus re-run (not just the failed case) → fresh grade. Two REOPENs on
  the same finding → valve E4 with both transcripts.
- **Who:** L2 owns the loop; the grader is never the fixer (HCI).

### A-L3 — Audit fleet + debt drain (Operating Cadence B)

- **Entry (fleet half):** any phase in flight → read-only `audit-fleet-lane`
  agents (L4, parallel-safe, never write the tree) sweep assigned surfaces and
  append ledger-row findings to `.planning/audits/QUALITY-LEDGER.md`.
- **Entry (drain half):** every gap between a phase close and the next dispatch.
- **Steps (drain):** triage rows → `eager-window` rows get sonnet L3 fixers
  (one tree-writer at a time); `intake-P<N>` rows get routed into the named
  phase's intake; catalog-row candidates get minted per `quality/catalogs/`
  schema. Every fixed row: prove-before-fix applies if BLOCKER (DP-2).
- **Exit:** zero undrained BLOCKERs. **No dispatch over undrained BLOCKERs** —
  this is the hard gate between phases.
- **Corrective:** a BLOCKER that won't drain (fix failed / repro won't build) →
  DP-2 disposition; design-level → valve E2. A drain window that keeps growing
  (> a phase-slot of work) → DP-5 balloon test.

### A-L4 — Re-audit to convergence (fresh eyes, cap 3)

- **Entry:** after any drain window that fixed ≥5 rows, after any surface-wide
  change (row migration, doc overhaul, root-CLAUDE slim), and at every portion
  boundary.
- **Steps:** dispatch a FRESH audit lane (new agent, zero prior context, not the
  agent whose fixes are being audited — HCI) over the same surface. It grades
  from artifacts only.
- **Exit:** an audit pass returning zero new BLOCKER/HIGH = converged.
- **Cap:** 3 iterations. At the cap with HIGHs remaining → the residue goes to
  intake with rationale; any remaining BLOCKER → valve E4.
- **Corrective:** if each pass finds a NEW class of issue (not residue of the
  last), the surface is under-specified — stop auditing, write the missing spec
  (catalog row / doc contract), then resume.

### A-L5 — Timers + steward window (standing holds)

Run by a `steward` agent (owner-named targets only) at EVERY phase close, and by
L0 at session start. Checklist — current values as of 2026-07-05, verify live:

| Item | State now | Action when it fires |
|---|---|---|
| PR #61 "chore: release v0.13.0" | OPEN, HELD until P97 | Do not touch until P97 GREEN; then close/merge per P97 frame (ch.03 §E) |
| PR #62 (codecov dependabot) | no action per steward.md | leave |
| `structure/file-size-limits` waiver | expires **2026-08-08**, 10 violations | Before expiry: split the files or take an explicit owner re-waiver; silent lapse = pre-push starts blocking |
| security waivers ×2 | until **2026-08-15**, P0 | flip WAIVED→PASS after first real CI run (P92) |
| docs-repro ×5 + subjective ×3 waivers | until **2026-09-15** | drain in P95 |
| Weekly chronic-yellow rows | `benchmark-claim/8ms-cached-read`, `89.1-percent-token-reduction` — yellow BY DESIGN (D-CONV-2) | do NOT “fix” the yellow; mechanization lands in launch-readiness (GOOD-TO-HAVES-04). Weekly verdict yellow-not-green is correct until then |
| Subjective rubrics 30-day TTL | `quality/catalogs/subjective-rubrics.json` | `/reposix-quality-review --all-stale` when stale (P97 does a full pass) |
| docs-alignment | 17 rows STALE_TEST_DRIFT @ `00ddc53` (ratio 0.818→0.768, floor 0.5) | top-level `/reposix-quality-refresh <doc>`; if pre-push BLOCKs mid-phase: checkpoint executor, run the slash command from a fresh top-level session |
| `JIRA_TEST_PROJECT` secret | NOT set in repo; CI defaults `TEST`, owner's live project is `KAN` | needs OWNER to set the secret (E1 — credential handling); until then JIRA CI rows run against `TEST` |
| Orphan processes | e.g. stale `claude` PIDs, leftover cargo | `ps aux | grep -E "cargo|rustc|claude"`; kill only OWN session's orphans; foreign long-lived PIDs → note to owner |
| ~19 stale remote branches | blocked on owner-named approval | list to owner once per portion; never delete unnamed |
| `tag-v0.13.0.sh` | `.disabled` | stays disabled until P97 milestone verdict GREEN |
| Scheduled-workflow health (weekly quality verdict, `security-audit`/RUSTSEC, bench-cron) | chronic-yellow + bench rows above are the known-benign set | Read each scheduled run at the phase-close steward window (`gh run list --workflow=<name>`); a NEWLY-red scheduled run is a regression to fix BEFORE the next dispatch, not alerting-only noise. Chronic-yellow rows (above) stay yellow by design — do not "fix" them |
| Code-coverage trend (`cargo-llvm-cov` → Codecov, `ci.yml` `coverage` job) | generated in CI; NO ratchet floor gated today; watch-only | Check the phase's Codecov delta; an unexplained line-coverage DROP is a regression to explain or restore before advancing. Ratchet-floor catalog row proposed in `GOOD-TO-HAVES.md` (2026-07-05, `code/coverage-ratchet`) — NOT yet an enforcement gate, do not treat watch-only as blocking |

## §B — Context budgets (quantified, per tier)

| Tier | Budget | Mechanism |
|---|---|---|
| L0 | ~10% of context for the WHOLE drive (all portions together, not per-portion); remaining ~90% is correction margin, never planned workload | Report-only diet: consumes ≤400-word reports and ≤300-word digests ONLY. Never opens source files, plans, or transcripts — an L0 that reads a 500-line file has spent a phase's budget on one turn |
| L1/L2 coordinators | end-state by ~50% of own context; relief at any wave boundary past 50% | `relief-handover-writer` (L4) writes+commits the handover; PRE-NOTIFIED rotation (DP-1); `gsd-context-monitor.js` warns at ≤35%/≤25% remaining but do not wait for it |
| L3 lanes | ≤100 tool calls | If a plan implies more → split via L2 BEFORE dispatch (10x rule). L3 may push mechanical halves down to L4 haiku |
| L4 leaves | single errand; digest ≤300 words, report ≤400 words | Terminal — no spawning |

## §C — Rot-signal checklist + rotation

Watch every child coordinator for (any 2 → rotate; details DP-1):
stop/watch cycles · <5-tool-call bookkeeping turns · watcher-arming/sleeps ·
re-asking answered questions · re-reading already-digested files · report
latency up while commit rate down · self-contradiction on wave state.

Rotation is ALWAYS pre-notified: announce → atomic unit completes →
`relief-handover-writer` commits the handover (SHA confirmed) → successor spawned
with handover as charter → predecessor stands down. Top-level rotation uses the
same template into `.planning/SESSION-HANDOVER.md`.

## §D — Mis-routed-reply relay

When a sub-agent's reply cannot be delivered (cross-session addressing failure),
L0/L1 relays the FULL report inline to the intended parent. **Before re-running
any lane, check the agent tree and `git log --oneline` first** — the work may
already be committed. Duplicate lanes cost context AND can double-write the tree.
(`post-dispatch-relay.sh` injects this reminder after dispatches.)

## §E — Periodic reminders: hooks vs procedure

**Hooks already inject (do not duplicate, do rely on):**

| Hook | Fires | Injects/enforces |
|---|---|---|
| `cargo-mutex.sh` | PreToolUse | BLOCKS second concurrent cargo (exit 2) |
| `dispatch-doctrine.sh` | PreToolUse (Agent) | JIT tier check, charter shape, lane-size |
| `post-dispatch-relay.sh` | PostToolUse | check-tree-before-rerun reminder |
| `session-start-brief.sh` | SessionStart | orientation brief |
| `stop-uncommitted.sh` | Stop | advisory: uncommitted = didn't happen |
| `precompact-persist.sh` | PreCompact | persist state before compaction |
| `gsd-context-monitor.js` (user-level) | context thresholds | WARNING ≤35% / CRITICAL ≤25% remaining |

**Procedural (nothing injects these — the runbook is the reminder):**

- **Every phase close:** re-read this chapter's §A loop table; run A-L5 steward
  checklist; confirm push-before-verifier happened; ledger BLOCKER scan.
- **Every drain window:** A-L4 convergence check due? intake files routed?
- **Every portion boundary:** re-read ch.03's next portion frame; A-L4 fresh
  audit; report to owner (one ≤400-word portion summary).
- **Every owner message:** treat casual asides as rule seeds — encode into the
  relevant artifact same-session (fix-it-twice; this very runbook's Amendments
  1–3 came from asides).
- **Every CLAUDE.md edit:** root file was slimmed to ~16k (a74f9b3, ahead of the
  P95 plan) via the ORCHESTRATION.md pointerization pattern (~6-line summary +
  pointer); the ≤40k discipline holds with headroom. Additions still pointerize to
  a scoped `CLAUDE.md` or long-form home rather than inlining — grow the pointer,
  not the root. (Note: ch.03 §D bucket 2 still frames the slim as pending P95 work;
  that framing is now stale — the slim already landed.)
- **Weekly (or first session after):** chronic-row check — yellow rows listed in
  A-L5 stay yellow by design; anything NEWLY yellow gets a ledger row.
