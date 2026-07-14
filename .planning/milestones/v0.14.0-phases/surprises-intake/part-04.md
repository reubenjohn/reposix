# v0.14.0 Surprises Intake (P110 source-of-truth) — Part 4 of 4

> Split from `SURPRISES-INTAKE.md` for the file-size gate (OP-8 drain). Index: `../SURPRISES-INTAKE.md`. Entries preserved verbatim.
>
> Filed by the successor #13 READY-TO-TAG mechanical (v0.14.0 tag-prep, 2026-07-13):
> authoring `tag-v0.14.0.sh` + draining this session's noticing backlog before the
> release manager runs the tag script. Items 1–4 are new full entries; the trailing
> "Carried forward" section lists prior-session findings that were never actually
> written into an intake entry (only referenced in passing) — filed here, one line
> each, rather than duplicated in full where an equivalent entry already exists.

## 2026-07-13 | discovered-by: successor #13 (v0.14.0 tag-prep mechanical, READY-TO-TAG authoring) | severity: HIGH

**Title:** Milestone-close 9th-probe cadence cannot reach honest exit-0 on a sub-2.34 VM.

**What:** `t4-conflict-rebase-ancestry-real-backend` (P0) git-floors at `git 2.25.1 <
2.34` — the two-writer conflict+refetch scenario needs `stateless-connect`
partial-clone, which needs git ≥2.34. On this VM only git 2.25.1 exists (no
passwordless sudo available to upgrade it in-session). This is the subject of
Manager Ruling #4 (Option B — tag proceeds under a recorded caveat; the git upgrade
(Option A) is RAISEd to the owner, explicitly NOT attempted autonomously per the
ruling's third binding condition). See
`quality/reports/verdicts/milestone-v0.14.0/VERDICT.md` § "The sole cadence caveat."

**Why out-of-scope for this mechanical:** fixing this needs either provisioning a
git ≥2.34 environment (infra change requiring privileged install, owner-gated by
Ruling #4 itself) or a doc change scoping the requirement — both are beyond a
tag-authoring mechanical's charter, and the manager has already ruled on the
disposition (tag proceeds, caveat recorded, no autonomous git upgrade attempt).

**Sketched resolution:** (a) document a git ≥2.34 requirement for the
`pre-release-real-backend` cadence environment specifically (distinct from the
already-documented "2.34+ recommended" general guidance in root `CLAUDE.md`, which
is a WARN not a hard requirement) in `quality/PROTOCOL.md` or
`docs/reference/testing-targets.md`, OR (b) provision a CI environment (dedicated
runner or container image) with git ≥2.34 so t4's destructive two-writer scenario
can actually execute at milestone-close instead of perpetually floor-gating on
whatever git ships on the interactive dev VM.

**STATUS:** OPEN — routed to v0.15.0. Cross-ref `.planning/CONSULT-DECISIONS.md`
"2026-07-13 [MANAGER] Ruling #4" + `.planning/RETROSPECTIVE.md` § "Tag-prep arc."

## 2026-07-13 | discovered-by: successor #13 (v0.14.0 tag-prep mechanical, READY-TO-TAG authoring) | severity: HIGH

**Title:** Transient verification-JSON clobber footgun bit hard this session (twice; confused 2 agents).

**What:** Creds-absent runs of `pre-release-real-backend` — including an ORPHANED
background shell that completed after its parent agent had already exited — overwrite
the gitignored per-run JSONs under `quality/reports/verifications/agent-ux/*.json` to
`env-missing`, silently destroying real-backend PASS / git-floor evidence a prior
credentialed run had captured. Two subagents this session misread the clobbered JSONs
before the committed catalog was confirmed as the actual source of truth — the
independent ratification subagent had to explicitly call this out and route around it:
`quality/reports/verdicts/milestone-v0.14.0/RATIFICATION.md` states it deliberately did
NOT use the transient JSONs as evidence because they were "clobbered after grading by an
unrelated background creds-absent run." The manager also flagged the mechanism in Ruling
#3: "Also filing the mechanism findings (run.py no-dotenv autoload; `--persist`
creds-absent clobber footgun) to intake" (`.planning/CONSULT-DECISIONS.md:134`) — that
filing intent was never actually executed as an intake entry until now; this entry (plus
item 4 below, the `.env`-autoload half) is that filing.

**Why out-of-scope for this mechanical:** fixing `--persist`'s overwrite semantics is a
runner code change (new guard logic comparing the existing committed grade against the
incoming grade before writing) — Rule-4 architectural, more than a documentation fix,
and risks masking a genuine future env-missing regrade if implemented carelessly. Not a
`<1h` in-place tweak for a tag-authoring mechanical that must not touch `run.py`.

**Sketched resolution:** (a) `--persist` must REFUSE to overwrite a real PASS/git-floor
grade with an env-missing NOT-VERIFIED (extends the existing `--persist` creds-absent
footgun the manager flagged in Ruling #3); (b) document explicitly (in
`quality/PROTOCOL.md` and/or `quality/CLAUDE.md`) that the COMMITTED
`quality/catalogs/agent-ux.json` is the source of truth, NOT the gitignored per-run
JSONs under `quality/reports/verifications/agent-ux/`; (c) consider a session-scoped
per-run JSON path (e.g. keyed by a run id or PID) so a stray orphaned background run
cannot clobber a concurrent session's artifact.

**STATUS:** OPEN — routed to v0.15.0. Cross-ref `RATIFICATION.md` (worked around this
footgun explicitly) and `.planning/CONSULT-DECISIONS.md` Ruling #3 (mechanism-findings
note that first flagged this).

## 2026-07-13 | discovered-by: successor #13 (v0.14.0 tag-prep mechanical, READY-TO-TAG authoring) | severity: MEDIUM

**Title:** Orphaned-background-shell dispatch hazard.

**What:** A verifier subagent launched a long real-backend cadence via
`run_in_background` and then ended its turn, orphaning the shell. The orphaned shell
later completed creds-absent (its parent's credential context was gone) and re-clobbered
the verification JSONs — the direct mechanism behind item 2 above.

**Why out-of-scope for this mechanical:** this is a dispatch-doctrine gap in
`.planning/ORCHESTRATION.md`, not a code fix — amending standing orchestration doctrine
is a decision the owner/manager should review, not something a tag-authoring mechanical
should silently edit in passing.

**Sketched resolution:** long real-backend runs must be FOREGROUND/blocking, never
background-and-exit — a subagent that dispatches `run.py --cadence
pre-release-real-backend` (or any credentialed real-backend gate) must wait for it to
complete before ending its turn. Candidate tightening for `.planning/ORCHESTRATION.md`
dispatch doctrine (near the existing "gh pr checkout" isolation rule in
`.planning/CLAUDE.md` § Subagent-dispatch specifics is a reasonable sibling location).

**STATUS:** OPEN — routed to v0.15.0 orchestration-doctrine review.

## 2026-07-13 | discovered-by: successor #13 (v0.14.0 tag-prep mechanical, READY-TO-TAG authoring) | severity: MEDIUM

**Title:** `run.py` has no `.env` autoload.

**What:** `quality/runners/run.py` reads `os.environ` directly; a fresh subagent shell
inherits no creds, so a naive 9th-probe dispatch silently env-gate-skips every
real-backend row (fails closed to NOT-VERIFIED, per OD-2 — not a silent pass, but still
an avoidable false-negative that costs a session a full diagnosis cycle). Also flagged
by the manager in Ruling #3: "filing the mechanism findings (run.py no-dotenv
autoload...)" (`.planning/CONSULT-DECISIONS.md:134`) — this entry executes that filing
intent.

**Why out-of-scope for this mechanical:** modifying `run.py` (already ~24k chars, over
the 15000-char anti-bloat cap tracked by `GOOD-TO-HAVES-06`) to add dotenv parsing is a
runner code change with test-suite implications (`quality/runners/test_run.py`) — not a
`<1h` tweak for a tag-authoring mechanical, and better landed together with the
already-filed anti-bloat split so the file isn't touched twice.

**Sketched resolution:** add `--env-file`/dotenv autoload to `run.py`, OR codify the
in-shell `set -a && . ./.env && set +a` dispatch pattern as the ONLY sanctioned
invocation in `quality/PROTOCOL.md` (so every dispatcher copies one documented idiom
instead of rediscovering the env-gate gap independently each session).

**STATUS:** OPEN — routed to v0.15.0. Cross-ref `GOOD-TO-HAVES-06` (`run.py`
anti-bloat-cap split — land together) and `.planning/CONSULT-DECISIONS.md` Ruling #3.

## Carried forward, still-unfiled from prior handovers (one-liners, LOW)

Findings noticed in prior sessions but never written into a dedicated intake entry —
filed here as pointers rather than duplicated where an equivalent entry already exists.

- **Dead `PROTECTED_IDS` var in `scripts/refresh-tokenworld-mirror.sh`.** Line 66 sets
  `PROTECTED_IDS=" 7766017 7798785 "` but the actual guard loop at line 132 hardcodes
  `for pid in 7766017 7798785; do` instead of iterating `$PROTECTED_IDS` — the variable
  is assigned and never read. Cosmetic (the hardcoded ids are correct today), but a
  future edit to one list and not the other silently desyncs the guard from its own
  documented protected-id source. **STATUS:** OPEN — routed to v0.15.0, mechanical fix
  (`for pid in $PROTECTED_IDS; do` or drop the unused var).
- **Split-candidate tests over the file-size soft-warn band.**
  `crates/reposix-cli/tests/agent_flow_real.rs` (47364 B) and
  `crates/reposix-confluence/src/translate.rs` (26597 B) are both real (non-`crates/*.rs`
  bare-name-exempt) `.rs` files past the 20000 B `structure/file-size-limits.sh` ceiling.
  Not yet in `GOOD-TO-HAVES-02`'s residual list. **STATUS:** OPEN — routed to v0.15.0,
  fold into the `GOOD-TO-HAVES-02` file-size-drain residual sweep.
- **Stale STATUS language at `surprises-intake/part-03.md:59-61`.** The B2 p93 entry's
  STATUS block still reads "active p93 blocker for the v0.14.0 tag; owner decision
  (fix-before-tag vs documented waiver) pending" — overtaken by commit `1c424d7` ("test
  (agent-ux): rewrite p93 real-Confluence smoke to honest UPDATE-recovery") after which
  `p93-partial-failure-recovery-real-confluence` now PASSES (see
  `quality/reports/verdicts/milestone-v0.14.0/VERDICT.md`). **STATUS:** OPEN — routed to
  v0.15.0 (or the next intake-hygiene pass), re-STATUS that block to
  `RESOLVED-in-1c424d7` with a pointer to the VERDICT's PASS row.
- **`run_helper_export_real` discards helper stderr.** Already filed —
  `surprises-intake/part-03.md:270-277` (item-5 DIAGNOSTIC lane entry, STATUS: OPEN,
  tangent 2 of 3) — `agent_flow_real.rs:720-724` swallows stderr so real push failures
  surface as bare `some-actions-failed` with zero per-action detail. Not re-filed here;
  listed only so this carry-forward cluster is a complete pointer set.
- **`.planning/CONSULT-DECISIONS.md` is ~50053 B**, over the 20000 B `*.md`
  soft-warn ceiling (`structure/file-size-limits.sh`, currently in the print-only
  EARLY-WARNING band relative to its own ceiling class — this file is long-lived planning
  ledger prose, not a split-ledger candidate per `GOOD-TO-HAVES-02`'s existing
  exclusion list). **STATUS:** OPEN — routed to v0.15.0, prune CLOSED/RULED entries at a
  clean milestone boundary (git history is the archive; no information is lost by
  trimming the live file).
