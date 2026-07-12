# RELIEF-HANDOVER — C2 milestone-coordinator, v0.14.0 "Wave-2 hardening"

Relief point: clean wave boundary — no lanes running, P105 closed GREEN and pushed,
tracked tree clean. Written by an out-of-band handover-writer agent from content
supplied by the outgoing C2 coordinator; this agent did NOT do any milestone work
itself. Successor: read this file, then `.planning/ORCHESTRATION.md` §3 (relief
protocol) and §11-12 (tiering/HCI) if resuming without fable.

## 1. Ground truth (git)

- Branch `main`, HEAD `f944945`, **`origin/main == HEAD == f944945`** (verified via
  `git rev-parse HEAD origin/main`).
- `git config user.email` = `reubenvjohn@gmail.com` (verified correct).
- Tracked tree clean (`git status --porcelain` shows zero tracked-file diffs). Four
  UNTRACKED items present — see §5 ANOMALY, do not commit or delete without
  investigation:
  - `.planning/phases/21-op-7-hardening-bundle-contention-swarm-500-page-truncation-p/`
  - `.planning/phases/22-op-8-honest-tokenizer-benchmarks-replace-len-div-4-with-coun/`
  - `scripts/demos/`
  - `scripts/dev/`
- CI/gates status as of HEAD: honestly clean (57 PASS / 1 filed P2 FAIL / 1 WAIVED /
  0 NOT-VERIFIED, confirmed no masked-red repo-wide).
- Recent commit chain (most-recent-first) landing P105:
  - `f944945` chore(105): re-sync doc-alignment hashes for edited index.md:152
  - `104c689` docs(105): de-jargon RBF-LR-03 recovery prose (banned-words cold-reader)
  - `d877dd2` docs(105): file RBF-LR-03 lane intakes (resolve_import_parent MED + file-size LOW)
  - `fecad34` docs(105): RBF-LR-03 recovery resolved — correct known-limitation claims (fix-twice)
  - `0d3afe9` docs(verify): P105 rebase-recovery-reconciles — GREEN (unbiased phase-close)
  - `8afb52d` test(105): gate+unit cover deletion + no-op guard (WR-01/WR-02)
  - `140f4eb` fix(105): emit deleteall — deletions propagate (CR-01, BLOCKER fix)
  - `76ba06a` review(105): adversarial code review — BLOCKER deletion-propagation regression
  - `d210350`/`3766915`/`bf1ab5d` test(105): rebase-recovery gate drives full pull-rebase-push
  - `bd5b9cb` fix(105): helper import writes private ref ns — RBF-LR-03 ref-lock
  - `7211c92`/`44d6aec` test/docs(105): live gate exposes second bug (ref-lock)
  - `90ddaff` fix(105): fast-import chains onto tracking tip — RBF-LR-03 rebase-recovery
  - earlier: P104 gh404 fix chain, P103 early wins, P102 self-safe dark-factory.

## 2. Wave/cycle state

Milestone roadmap: `.planning/milestones/v0.14.0-phases/ROADMAP.md`, phases P102–P112.

| Phase | State | Commits / notes |
|---|---|---|
| P102 D2 self-safe dark-factory (HARD SERIALIZING GATE) | DONE | reject-`t@t`-identity hook, real per-leaf `/tmp`-clone isolation (no `--force` worktree removal), PreToolUse shared-`.git/config` write-block. Verdict `quality/reports/verdicts/p102/` |
| P103 early wins | DONE | doc-alignment grade/persist split; F-K4b false-demote robustness; OP-8 file-size split → `part-NN.md`+index, reusable `scripts/split_ledger.py` |
| P104 gh404 (`S-260707-gh404`) | DONE | `sanitize_project_for_cache` fix (raw slug to REST, sanitize on-disk only). `continue-on-error` at `ci.yml:263` intentionally RETAINED + KNOWN-LIMITATION kept (real-backend claim NOT promoted). Verdict `quality/reports/verdicts/p104/` |
| health-triage (interstitial) | DONE | confirmed no masked-red; `init.rs:159` clippy already fixed (`6efe293`) |
| P105 RBF-LR-03 rebase-recovery reconciliation | **DONE, CLOSED GREEN, pushed** | fixed parentless tracking-commit + helper/git-fetch ref collision; review-caught BLOCKER CR-01 (deletion-resurrection) fixed via `from <tip>`+`deleteall`; helper writes moved to private `refs/reposix-import/*`. Verdict `quality/reports/verdicts/p105/` |
| P106 (NEW lost-update phase, unscheduled) | NOT STARTED | see §5 — needs a phase slot minted |
| P107 RUSTSEC | NOT STARTED (assessment done, confirm pending) | |
| P108 prune_oid_map pagination-truncation | NOT STARTED | |
| P109 RBF-FW-11 + quality-convergence | NOT STARTED | |
| P110/P111 OP-8/OP-9 absorption slots | NOT STARTED | |
| P112 OD-4 launch-readiness | SCOPE-BUT-DO-NOT-START stub | begins only after wave-2 hardening closes |

No named-incident post-mortem outstanding beyond what's captured in §5.

## 3. Binding constraints (unchanged)

- **ONE cargo invocation machine-wide** (`-p <crate>`, never `--workspace` in parallel);
  `.claude/hooks/cargo-mutex.sh` is the backstop, orchestration discipline is primary.
- **One tree-writer at a time.** The shared-worktree WRITE-RACE is REAL and was
  observed twice this wave (staged-file cross-contamination + push-tip races) — any
  parallel code-touching lane MUST run `isolation:"worktree"` (`git worktree add
  /tmp/<uniq> <branch>`), never share the coordinator's checkout.
- **No `--no-verify`.**
- **Push only at green** — `git push origin main` BEFORE the verifier-subagent dispatch;
  verifier grades RED if the phase shipped without the push landing.
- Commit-trailer format: `Co-Authored-By: Claude <tier> <noreply@anthropic.com>` (model
  tiering: fable → opus (security/complex) → sonnet (default) → haiku (mechanical)).
- Leaf isolation HARD-STOP: all `reposix init`/sim/`git config`/`git commit` FOR TESTING
  runs in a throwaway `/tmp/<uniq>` clone, `cd` in the SAME Bash invocation, never the
  shared repo.
- Verify `git config user.email == reubenvjohn@gmail.com` before every commit.

## 4. Litmus / gate / REOPEN state

- P102 self-safe dark-factory gate: PASS, verdict at `quality/reports/verdicts/p102/`.
- P104 gh404: sim/unit-level PASS; real-backend GitHub claim explicitly NOT promoted
  (KNOWN-LIMITATION doc + `continue-on-error` retained) — see owner ask #1 in §5.
- P105 rebase-recovery gate (`quality/gates/...rebase-recovery...`): PASS after CR-01
  fix + WR-01/WR-02 unit coverage added; verdict `quality/reports/verdicts/p105/`
  independently confirmed GREEN (`0d3afe9`, unbiased phase-close).
- Main-line pre-push gate suite at HEAD: 57 PASS / 1 filed P2 FAIL / 1 WAIVED / 0
  NOT-VERIFIED — no masked-red confirmed (P0/P1-blocks-vs-P2-nonblocking is documented
  doctrine, not a `|| true` mask).
- **Open waiver clock:** `code/shell-coverage` P2 FAIL at 12.54% < 13% floor — filed,
  not yet fixed, not a blocker for this relief.
- **ADR-010 duplicate-record/slug→id waiver** remains OPEN — P105's Lane 3 correctly
  refused to falsely mark it resolved; do not close it without doing the actual work.
- `quality/reports/verifications/agent-ux/fleet-safety-*.json` (3 files) regenerate
  DIRTY on any grading read (known persist-gate residual, see §5) — if they reappear
  dirty in `git status`, `git checkout --` them; never commit them.

## 5. Mid-execution decisions + noticed-not-filed

**Owner-named-target asks (forward to L0 — NOT self-authorizable):**
1. **gh404 real-backend verify.** Authorize ONE read-only GitHub call
   (`reposix init github::reubenjohn/reposix` → `GET /repos/reubenjohn/reposix/issues`,
   mutates nothing, needs `GITHUB_TOKEN` + `REPOSIX_ALLOWED_ORIGINS`) to retire the
   KNOWN-LIMITATION and promote `ci.yml:263` to load-bearing.
2. **RUSTSEC reframe.** Open dependabot PRs are tower-http #64 / gix #65 / rusqlite #66
   (NOT memmap2/quinn-proto as the original RUSTSEC HIGH implied); zero open dependabot
   alerts + green `cargo-audit` in CI → the original RUSTSEC HIGH looks
   already-resolved. Confirm whether separate memmap2/quinn-proto PRs exist/were merged
   before closing P107.
3. **Dependabot merges #64/#65/#66** — owner-gated, all currently RED from staleness;
   #64/#65 carry `feat!` breaking changes despite minor version labels; #66 (rusqlite
   0.40.1) has real SQL-injection hardening. Recommended: rebase + re-check each, then
   owner decides merge order.

**De-facto decisions made live this wave (not yet formalized elsewhere):**
- P104 deliberately did NOT promote the real-backend GitHub claim — kept
  `continue-on-error` + KNOWN-LIMITATION doc rather than declaring the fix complete
  without a real-backend run. This is a live judgment call, not an oversight; owner
  ask #1 above is the unblock.
- P107 RUSTSEC treated as "assessment done, confirm pending" rather than closed — the
  dependabot-PR evidence is suggestive but not yet independently re-verified via local
  `cargo audit`.

**Noticed, not yet filed as its own phase (HIGH severity):**
- **Silent lost-update bug** — `write_loop.rs:309`, stale `last_fetched_at` can cause a
  concurrent write to silently overwrite newer SoT state without conflict detection.
  Filed to `SURPRISES-INTAKE.md` by the P105 lane but has NOT been minted its own
  ROADMAP phase yet. This is the single highest-severity open item from this wave —
  schedule it before P108/P109 if severity holds up under a fresh read.
- **`§5 stateless-connect` (git ≥2.34) transport claim is UNVERIFIED** on this VM (local
  git is 2.25). Filed by P105; needs a modern-git CI run (or a container with git 2.34+)
  to actually exercise the partial-clone `stateless-connect` path before the doc claim
  can be called confirmed rather than asserted.
- **`structure/verifier-script-exists.sh` gate missing.** TWO independent lanes this
  wave recommended it — the P103 OP-8 file-splits broke two hardcoded-path verifiers
  that a "does the verifier script this catalog row points to actually exist"
  structural gate would have caught immediately. Non-cargo, cheap, high-leverage; not
  yet built.
- **agent-ux-walk persist-gate residual** — the 3 `fleet-safety-*.json` reports
  regenerate dirty on every grading read (same shape of bug P103 already fixed once for
  doc-alignment via the grade/persist split). Apply the identical fix pattern here.

## ANOMALY — coordination hazard (flag prominently to the incoming C2/L0)

A concurrent **herdr-manager** process appears to be live on this SAME working tree —
it owns `.planning/MANAGER-HANDOVER.md` / `.planning/STATE.md` (this handover
deliberately did NOT touch either file; STATE cursor advancement is deferred to that
process). Four untracked directories with a foreign phase-numbering scheme appeared
during this wave with unknown provenance:
- `.planning/phases/21-op-7-hardening-bundle-contention-swarm-500-page-truncation-p/`
- `.planning/phases/22-op-8-honest-tokenizer-benchmarks-replace-len-div-4-with-coun/`
- `scripts/demos/`
- `scripts/dev/`

These do NOT match this milestone's `P102–P112` numbering (they look like a different
phase-numbering scheme entirely) and were never committed or examined by this C2. The
incoming C2/L0 MUST investigate their origin (diff their contents, check for another
concurrent agent process, check timestamps) BEFORE committing or deleting anything in
them. Do not assume they are garbage; do not assume they are safe to merge in.

The observed shared-worktree write-race (staged-file cross-contamination + push-tip
races, twice this wave) is consistent with more than one coordinating process writing
to this tree concurrently — treat ALL future parallel lanes as worktree-isolated by
default until this is resolved, not just as a performance optimization.

## 6. Precise next steps (successor runbook)

1. **Ground-truth first.** Run `git rev-parse HEAD origin/main` and `git status`
   yourself — do not trust this document's snapshot if time has passed. Confirm still
   clean / still `f944945`.
2. **Investigate the 4 untracked dirs** (§5 ANOMALY) before touching anything else in
   `.planning/phases/` or `scripts/`. Determine provenance; do not commit or delete
   until understood. If they belong to the concurrent herdr-manager process, leave them
   alone and note that in your own ground-truth pass.
3. **Escalate the 3 owner-named-target asks (§5)** to L0 — do not self-authorize the
   real-backend GitHub call or any dependabot merge.
4. **Mint a ROADMAP phase for the lost-update HIGH** (`write_loop.rs:309`) — this is
   the single most severe open finding from this wave; schedule it ahead of or
   alongside P108/P109 depending on fresh-read severity.
5. **Dispatch `structure/verifier-script-exists.sh` gate** as a cheap, non-cargo,
   high-leverage lane (two lanes already recommended it independently this wave).
6. **Confirm RUSTSEC (P107).** Run local `cargo audit`; if memmap2/quinn-proto are
   confirmed clear (per the dependabot-PR evidence in §5), close P107 as a near-no-op
   with a one-line verdict citing the confirming run.
7. **Fix the agent-ux-walk persist-gate residual** using the same grade/persist split
   pattern P103 already established for doc-alignment (`quality/CLAUDE.md` /
   `quality/gates/agent-ux/` — locate P103's diff as the template).
8. **Then work down the remaining queue** — P108 (prune_oid_map pagination-truncation),
   P109 (RBF-FW-11 + quality-convergence), the `code/shell-coverage` 12.54%<13% P2 FAIL,
   the open ADR-010 duplicate-record waiver (do the real work, don't false-close it) —
   respecting ONE cargo invocation machine-wide and worktree isolation for any parallel
   code-touching lane.
9. **P110/P111 (OP-8/OP-9 absorption slots)** drain `SURPRISES-INTAKE.md` /
   `GOOD-TO-HAVES.md` per OP-8/OP-9 doctrine — do this only after the hardening queue
   above is substantially drained, per the milestone's own phase order.
10. **P112 (OD-4 launch-readiness) stays a SCOPE-BUT-DO-NOT-START stub** until wave-2
    hardening formally closes — do not pull it forward without an explicit executive
    resequencing decision (DP-4 / ORCHESTRATION §10).
11. **At milestone close:** run the non-skippable 9th probe
    (`python3 quality/runners/run.py --cadence pre-release-real-backend`), distill
    `SURPRISES-INTAKE.md`/`GOOD-TO-HAVES.md`/run findings into `.planning/RETROSPECTIVE.md`
    (OP-9) BEFORE archiving, advance `.planning/STATE.md` cursor (coordinate with the
    herdr-manager process rather than hand-editing it directly), and report tag-readiness
    to L0. Do NOT `git tag`, trigger `release.yml`, or publish crates.io — that is L0's
    irreversible action.
