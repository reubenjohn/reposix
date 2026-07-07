# RELIEF-HANDOVER — C2 milestone-coordinator, v0.13.1 "Front door actually works"

Relief point: Wave F, zero-shot human-sim gate returned RED. Written by an out-of-band
handover-writer agent from content supplied by the outgoing C2 coordinator — this agent
did NOT do any milestone work itself.

## 1. Ground truth (git)

- Branch `main`, HEAD `90dfc6b`, tree clean.
- `git config user.email` = `reubenvjohn@gmail.com` (verified correct).
- **The milestone IS pushed:** `origin/main == HEAD == 90dfc6b`. Pre-push cadence passed
  55 PASS / 0 FAIL / 1 pre-existing WAIVED (`file-size-limits`, expires 2026-08-08).
- Full commit list since pre-milestone `540fb1f`:
  - `cdc0bef`, `499ec67`, `8781c7f`, `ba49482`, `5ca79c5` — Waves A/B
  - `a94425b` — predecessor handover
  - `0578919`, `dd49109` — Wave D doc-truth
  - `2dd207d`, `320562f`, `4f1e0f0`, `30ede06` — Wave E1/E1b
  - `86ff771`, `5050a46` — E2 B6 catalog row + verifier
  - `5122ebb` — E3 phantom-green gate fix
  - `3051f2e`, `890414a`, `8a7f9c5` — F1 pre-push fixes
  - `0fbd5e7` — F1b doc-alignment refresh
  - `90dfc6b` — F1b badges-entry restore + push

## 2. Wave / cycle state

| Wave | State | Notes |
|---|---|---|
| D (doc-truth: B1/B2/B3-docs/checkout/B5/intake) | DONE | mkdocs+mermaid green |
| E (quality) | DONE | E1 README fix + bundled-seed GTH + binstall rows; E1b doc-alignment footprint rebind (8 rows); E2 (re-dispatched after an infra watchdog stall) added `agent-ux/zero-shot-onboarding` catalog row (catalog-first) + verifier `quality/gates/agent-ux/zero-shot-onboarding.sh` (PASS 3x); E3 fixed phantom-green `agent-ux/dark-factory-sim` gate (missing `REPOSIX_SIM_ORIGIN` export), re-minted honestly |
| F step 1 (push) | DONE | F1 cleared 3 pre-push FAILs (`3051f2e` test-name-honesty markers, `890414a` banned "fast-import" jargon, `8a7f9c5` banned-words marker placement), then STOPPED at 2 judgment blockers; F1b (opus) resolved both — doc-alignment refresh via INLINE Opus grading of 7 stale rows on first-run.md+index.md (also corrected 2 Haiku-backfill false-BONDS), and badges gate root cause = a prune commit had deleted the `badges-resolve` RESOLVED GOOD-TO-HAVES entry the gate requires (restored; verified 8/8 badge URLs HTTP 200 — NOT egress-blocked); then pushed |
| F step 2 (THE zero-shot human-sim gate) | **RED** | see §3/§4 |
| G (gsd-verifier) | NOT STARTED | |

## 3. Litmus / gate / REOPEN state — THE RED

A fresh general-purpose agent, zero repo context, ran README + `docs/tutorials/first-run.md`
verbatim. The flow COMPLETED read+write+push+audit with zero fixup COMMANDS, but
documented OUTPUTS mismatched reality — a doc-lie, which counts as FAIL. Per-finding
triage the successor MUST honor:

- **(a) `reposix init` "Next:" line — REAL doc fix.** Doc shows
  `Next: cd /tmp/repo && git checkout -B main refs/reposix/origin/main`; actual output
  appends ` (or git sparse-checkout set <pathspec> first)`. Transcribe the real output in
  `docs/tutorials/first-run.md`.
- **(b) `git push` ref-update shape — REAL doc fix.** Doc (`docs/tutorials/first-run.md`
  ~L157-160) shows `* [new branch]      main -> main`; reality is a fast-forward
  `bd848c1..be73654  main -> main` because `init` already created `main` on the remote.
  Fix the documented push output to the fast-forward form.
- **(c) UNDOCUMENTED secret-scanner output on push — AMBIGUOUS, INVESTIGATE BEFORE
  EDITING.** On `git push` in the /tmp tree, gitleaks-like lines appeared ("Unknown SCM
  platform…", "2 commits scanned.", "scanned ~3732 bytes…", "no leaks found") plus a
  `Secret scan: clean ✓` wrapper line — none documented. MUST DETERMINE: is this a
  GLOBAL/ENVIRONMENTAL git hook on THIS test box (check for a global `core.hooksPath` /
  a global pre-push hook / whether the repo's installed hooks leak into a fresh
  `reposix init` tree) — in which case a real external user would NOT see it, so it is a
  FALSE-FAIL artifact and NO doc edit is warranted (instead note it + filter it in the
  re-run gate) — OR is it a genuine `git-remote-reposix` push-flow secret-scan FEATURE
  (the `✓` wrapper suggests a deliberate reposix message) — in which case DOCUMENT it in
  the tutorial's push step and consider suppressing raw gitleaks tool-noise for UX. Do
  NOT edit docs for this until the environmental-vs-feature question is answered
  empirically.
- **(d) audit timestamp format — minor doc fix.** Doc shows `2026-07-07T15:38:27Z`
  (second precision, `Z`); reality `2026-07-07T22:57:00.885355337+00:00` (nanosecond,
  `+00:00`). Make the example match real format or mark it illustrative. Row CONTENT +
  ORDER matched exactly (`helper_push_accepted` / `helper_push_sanitized_field|version` /
  `helper_push_started`).
- **(e) `git >= 2.34` floor — REAL accuracy issue, do NOT blindly lower.** Confirmed: the
  entire sim flow incl. `extensions.partialClone=origin` worked on the box's
  `git 2.25.1`. The doc claims 2.34+ required (README + tutorial). INVESTIGATE whether
  2.34 is genuinely needed for real-backend / `stateless-connect` paths the sim flow does
  NOT exercise; likely soften to "sim flow works on 2.25+; 2.34+ required/recommended for
  <specific reason>" rather than delete the floor. Record the empirical finding for L0
  either way (predecessor RAISE-LIST item).
- Everything else matched docs EXACTLY: banner line, `ls issues/` = `1.md`..`6.md`,
  `cat issues/1.md` full contents (id/title="database connection drops under
  load"/status/labels/version), the two-hunk diff. README's 4 install blocks eyeballed
  clean (not executed).

## 4. Mid-execution decisions + noticed-not-filed

- doc-alignment ~180-row backlog likely harbors more Haiku-backfill false-BONDS of the
  shape F1b corrected → filed GTH; needs a systematic re-grade pass (v0.14.0). Wave G
  should note it.
- `doc-alignment walk` mutates the committed catalog in place with no `--persist` gate →
  filed GTH; bitten 3+ lanes (E1, E1b, F1b); prioritize the fix.
- The `S-260707` claim that `.git/hooks/pre-push` is a DEAD symlink is STALE/WRONG — a
  pre-push hook demonstrably ran (enforced gates + gitleaks) during F1b's push. Wave G /
  a fix lane should correct that surprise entry.
- `git >= 2.34` floor empirical finding (works on 2.25.1) — record for L0.
- `SURPRISES-INTAKE.md` is ~86k chars (4.3x soft limit) — pre-existing bloat, flag to L0
  at milestone close (v0.14.0 ingest/split).
- Phase-number collision v0.13.1 (P98-P101) vs v0.13.2 (P98-P107) — deferred to v0.13.2
  planning; do NOT touch v0.13.2 files.

## 5. Successor runbook (Wave D-prime → re-push → re-gate → Wave G → L0)

1. **Wave D-prime (opus lane).** FIRST investigate the (c) secret-scanner env-vs-feature
   question and the (e) git-floor question. THEN apply the confirmed-real doc-truth fixes
   [(a), (b), (d), (e), and (c) only if it's a real feature]. Run
   `bash quality/gates/docs-build/mkdocs-strict.sh` +
   `bash quality/gates/docs-build/mermaid-renders.sh`. Commit.
2. **CRITICAL:** any `docs/**` edit RE-STALES doc-alignment rows → the pre-push
   `docs-alignment/walk` gate will block again. Before re-pushing, rebind via INLINE
   Opus grading (the approach F1b used):
   `reposix-quality doc-alignment plan-refresh <doc>` → judge each stale row against
   `.claude/skills/reposix-quality-doc-alignment/prompts/grader.md` → `bind` with fresh
   grade → `walk` exits 0 → commit ONLY the in-scope rows. The walk MUTATES the whole
   catalog as a side effect with no `--persist` gate; `git checkout --` any
   out-of-scope flips. Do NOT spawn Task graders (depth-unreachable) — the opus executor
   IS the grader.
3. **Re-push `origin main`** (badges + previously-refreshed rows should pass; only the
   newly-edited docs need re-refresh).
4. **Re-run the zero-shot gate.** Fresh general-purpose agent, model sonnet, ZERO repo
   context, work from a fresh `git clone` of the pushed repo + the pre-built
   `target/debug` binary on PATH, copy-paste verbatim, STOP at first friction. Must
   return PASS with zero fixups AND all documented outputs matching reality. NEW
   findings = loop again. (Prior gate prompt is reusable — see the F2 dispatch shape: it
   must be told to disregard any injected CLAUDE.md and behave as an external
   first-timer; note the fidelity caveat that the harness injects repo CLAUDE.md.)
5. **Wave G.** Dispatch `gsd-verifier` (OP-7) to grade catalog rows PASS (incl
   `agent-ux/zero-shot-onboarding` + re-minted `dark-factory-sim`) + DoD met. RED loops
   back to the owning wave.
6. **Report tag-ready to L0.** STOP BOUNDARY: do NOT `git tag v0.13.1`, trigger
   `release.yml`, or publish crates.io — that is L0's irreversible action.

## Binding constraints (unchanged, load-bearing)

- ONE cargo invocation machine-wide (`-p <crate> -j2`, never `--workspace`; mutex hook).
- Leaf isolation HARD-STOP: all `reposix init`/sim/`git config`/`git commit` FOR TESTING
  runs in `/tmp/<uniq>`, same-invocation `cd`, never the shared repo.
- Verify `git config user.email` == `reubenvjohn@gmail.com` before EVERY commit.
- No `--no-verify`.
- Push `origin main` BEFORE the verifier.
- Embed the ownership charter in every code-touching dispatch; do NOT embed it in the
  zero-shot gate dispatch (naivety is the point).

## Release runbook state (relay to L0 at close)

crates.io publishes on MERGE-to-main via `release-plz.yml` (NOT the tag); tag `v*`
triggers `.github/workflows/release.yml`. Gotchas: `git_release_enable=false`
(per-package zero-asset releases previously stole `releases/latest` and 404'd installer
URLs — do not re-enable without reading that file's header comment); a bot-authored
release-plz push leaves `pull_request`-triggered workflows stuck at `action_required` (a
real-actor close/reopen unblocks); release-plz regenerates its PR on every `main` push so
the PR number moves.
